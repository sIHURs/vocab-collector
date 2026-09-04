#![forbid(unsafe_code)]

//! SQLite persistence and transactional outbox support.

use std::{path::Path, sync::Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
use uuid::Uuid;
use vocab_domain::{
    CaptureOrigin, Encounter, EncounterRepository, OwnerScope, RepositoryError, ReviewLog,
    ReviewRepository, ReviewState, SettingsRepository, UserSettings, Word, WordRepository,
    WordStatus, dedupe_key, normalize_lemma,
};

const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS words (
  id TEXT PRIMARY KEY, dedupe_key TEXT NOT NULL UNIQUE, owner_scope TEXT NOT NULL,
  lemma TEXT NOT NULL, display_form TEXT NOT NULL, source_language TEXT NOT NULL,
  target_language TEXT NOT NULL, translation TEXT, part_of_speech TEXT, status TEXT NOT NULL,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT, review_state_json TEXT
);
CREATE TABLE IF NOT EXISTS encounters (
  id TEXT PRIMARY KEY, word_id TEXT NOT NULL REFERENCES words(id), selected_text TEXT NOT NULL,
  sentence TEXT NOT NULL, source_app TEXT, source_title TEXT, source_url TEXT,
  capture_origin TEXT NOT NULL DEFAULT '"manual"',
  captured_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_encounters_word_captured ON encounters(word_id, captured_at DESC);
CREATE TABLE IF NOT EXISTS review_logs (
  id TEXT PRIMARY KEY, word_id TEXT NOT NULL REFERENCES words(id), rating TEXT NOT NULL,
  reviewed_at TEXT NOT NULL, received_at TEXT NOT NULL, device_id TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS user_settings (
  singleton INTEGER PRIMARY KEY CHECK(singleton = 1), payload TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS outbox (
  mutation_id TEXT PRIMARY KEY, entity_type TEXT NOT NULL, entity_id TEXT NOT NULL,
  operation TEXT NOT NULL, payload TEXT NOT NULL, created_at TEXT NOT NULL,
  attempt_count INTEGER NOT NULL DEFAULT 0
);
PRAGMA user_version = 2;
"#;

#[derive(Clone, Debug)]
pub struct CaptureRecord {
    pub selected_text: String,
    pub lemma: String,
    pub sentence: String,
    pub source_language: String,
    pub target_language: String,
    pub translation: Option<String>,
    pub part_of_speech: Option<String>,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub source_url: Option<String>,
    pub capture_origin: CaptureOrigin,
    pub captured_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct StoredCapture {
    pub word: Word,
    pub encounter: Encounter,
    pub is_existing_word: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSummary {
    pub schema_version: u32,
    pub foreign_keys_enabled: bool,
    pub word_count: usize,
    pub active_encounter_count: usize,
    pub review_log_count: usize,
    pub pending_outbox_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboxDebugEntry {
    pub mutation_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub attempt_count: u32,
}

pub struct SqliteStore {
    connection: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RepositoryError> {
        Self::from_connection(Connection::open(path).map_err(repo_error)?)
    }

    pub fn open_in_memory() -> Result<Self, RepositoryError> {
        Self::from_connection(Connection::open_in_memory().map_err(repo_error)?)
    }

    fn from_connection(connection: Connection) -> Result<Self, RepositoryError> {
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(repo_error)?;
        connection
            .execute_batch(MIGRATION_001)
            .map_err(repo_error)?;
        let has_origin = {
            let mut statement = connection
                .prepare("PRAGMA table_info(encounters)")
                .map_err(repo_error)?;
            statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(repo_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(repo_error)?
                .iter()
                .any(|column| column == "capture_origin")
        };
        if !has_origin {
            connection
                .execute_batch(
                    "ALTER TABLE encounters ADD COLUMN capture_origin TEXT NOT NULL DEFAULT '\"manual\"';
                     PRAGMA user_version = 2;",
                )
                .map_err(repo_error)?;
        }
        let has_review_result = {
            let mut statement = connection
                .prepare("PRAGMA table_info(review_logs)")
                .map_err(repo_error)?;
            statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(repo_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(repo_error)?
                .iter()
                .any(|column| column == "result_payload")
        };
        if !has_review_result {
            connection
                .execute_batch(
                    "ALTER TABLE review_logs ADD COLUMN result_payload TEXT;
                     PRAGMA user_version = 3;",
                )
                .map_err(repo_error)?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn capture(&self, input: &CaptureRecord) -> Result<StoredCapture, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(repo_error)?;
        let key = dedupe_key(&input.lemma, &input.source_language, &input.target_language);
        let existing = find_word(&transaction, &key)?;
        let is_existing_word = existing.is_some();
        let word = existing.unwrap_or_else(|| Word {
            id: Uuid::now_v7(),
            owner_scope: OwnerScope::Guest,
            lemma: normalize_lemma(&input.lemma),
            display_form: input.selected_text.trim().to_string(),
            source_language: input.source_language.trim().to_lowercase(),
            target_language: input.target_language.trim().to_lowercase(),
            translation: input.translation.clone(),
            part_of_speech: input.part_of_speech.clone(),
            status: WordStatus::Learning,
            created_at: input.captured_at,
            updated_at: input.captured_at,
            deleted_at: None,
            review_state: Some(ReviewState {
                difficulty: 5.0,
                stability: 1.0,
                due_at: input.captured_at,
                last_reviewed_at: None,
                lapse_count: 0,
            }),
        });
        if !is_existing_word {
            insert_word(&transaction, &key, &word)?;
            enqueue(
                &transaction,
                "word",
                word.id,
                "upsert",
                &word,
                input.captured_at,
            )?;
        }

        let mut encounter = Encounter::new(
            word.id,
            input.selected_text.trim().to_string(),
            input.sentence.clone(),
            input.source_app.clone(),
            input.captured_at,
        );
        encounter.source_title.clone_from(&input.source_title);
        encounter.source_url.clone_from(&input.source_url);
        encounter.capture_origin = input.capture_origin;
        insert_encounter(&transaction, &encounter)?;
        enqueue(
            &transaction,
            "encounter",
            encounter.id,
            "upsert",
            &encounter,
            input.captured_at,
        )?;
        transaction.commit().map_err(repo_error)?;
        Ok(StoredCapture {
            word,
            encounter,
            is_existing_word,
        })
    }

    pub fn pending_outbox_count(&self) -> Result<usize, RepositoryError> {
        let count: i64 = self
            .lock()?
            .query_row("SELECT COUNT(*) FROM outbox", [], |row| row.get(0))
            .map_err(repo_error)?;
        usize::try_from(count).map_err(repo_error)
    }

    pub fn foreign_keys_enabled(&self) -> Result<bool, RepositoryError> {
        self.lock()?
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, bool>(0))
            .map_err(repo_error)
    }

    pub fn schema_version(&self) -> Result<u32, RepositoryError> {
        self.lock()?
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(repo_error)
    }

    pub fn database_summary(&self) -> Result<DatabaseSummary, RepositoryError> {
        let connection = self.lock()?;
        let count = |sql: &str| -> Result<usize, RepositoryError> {
            let value: i64 = connection
                .query_row(sql, [], |row| row.get(0))
                .map_err(repo_error)?;
            usize::try_from(value).map_err(repo_error)
        };
        let schema_version = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(repo_error)?;
        let foreign_keys_enabled = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, bool>(0))
            .map_err(repo_error)?;

        Ok(DatabaseSummary {
            schema_version,
            foreign_keys_enabled,
            word_count: count("SELECT COUNT(*) FROM words WHERE deleted_at IS NULL")?,
            active_encounter_count: count(
                "SELECT COUNT(*) FROM encounters WHERE deleted_at IS NULL",
            )?,
            review_log_count: count("SELECT COUNT(*) FROM review_logs")?,
            pending_outbox_count: count("SELECT COUNT(*) FROM outbox")?,
        })
    }

    pub fn outbox_debug_entries(&self) -> Result<Vec<OutboxDebugEntry>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT mutation_id, entity_type, entity_id, operation, payload, created_at,
                 attempt_count FROM outbox ORDER BY created_at, mutation_id",
            )
            .map_err(repo_error)?;
        statement
            .query_map([], |row| {
                let attempt_count =
                    u32::try_from(row.get::<_, i64>(6)?).map_err(sql_conversion_error)?;
                Ok(OutboxDebugEntry {
                    mutation_id: parse_uuid(row.get(0)?)?,
                    entity_type: row.get(1)?,
                    entity_id: parse_uuid(row.get(2)?)?,
                    operation: row.get(3)?,
                    payload: parse_json(&row.get::<_, String>(4)?)?,
                    created_at: parse_time(row.get(5)?)?,
                    attempt_count,
                })
            })
            .map_err(repo_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(repo_error)
    }

    pub fn record_review(&self, word: &Word, review: &ReviewLog) -> Result<(), RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        let key = dedupe_key(&word.lemma, &word.source_language, &word.target_language);
        insert_word(&tx, &key, word)?;
        tx.execute(
            "INSERT INTO review_logs(id, word_id, rating, reviewed_at, received_at, device_id, result_payload)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                review.id.to_string(),
                review.word_id.to_string(),
                enum_json(&review.rating)?,
                review.reviewed_at.to_rfc3339(),
                review.received_at.to_rfc3339(),
                review.device_id.to_string(),
                review.result.as_ref().map(serde_json::to_string).transpose().map_err(repo_error)?,
            ],
        )
        .map_err(repo_error)?;
        enqueue(&tx, "word", word.id, "upsert", word, word.updated_at)?;
        enqueue(
            &tx,
            "review_log",
            review.id,
            "upsert",
            review,
            review.reviewed_at,
        )?;
        tx.commit().map_err(repo_error)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, RepositoryError> {
        self.connection
            .lock()
            .map_err(|_| RepositoryError::Persistence("database lock poisoned".into()))
    }
}

impl WordRepository for SqliteStore {
    fn find_by_dedupe_key(&self, key: &str) -> Result<Option<Word>, RepositoryError> {
        let connection = self.lock()?;
        find_word(&connection, key)
    }

    fn get(&self, id: Uuid) -> Result<Option<Word>, RepositoryError> {
        self.lock()?
            .query_row(
                "SELECT id, owner_scope, lemma, display_form, source_language, target_language,
                 translation, part_of_speech, status, created_at, updated_at, deleted_at,
                 review_state_json FROM words WHERE id = ?1",
                [id.to_string()],
                map_word,
            )
            .optional()
            .map_err(repo_error)
    }

    fn list(&self) -> Result<Vec<Word>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, owner_scope, lemma, display_form, source_language, target_language,
                 translation, part_of_speech, status, created_at, updated_at, deleted_at,
                 review_state_json FROM words WHERE deleted_at IS NULL ORDER BY updated_at DESC",
            )
            .map_err(repo_error)?;
        statement
            .query_map([], map_word)
            .map_err(repo_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(repo_error)
    }

    fn save(&self, word: &Word) -> Result<(), RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        let key = dedupe_key(&word.lemma, &word.source_language, &word.target_language);
        insert_word(&tx, &key, word)?;
        enqueue(&tx, "word", word.id, "upsert", word, word.updated_at)?;
        tx.commit().map_err(repo_error)
    }
}

impl EncounterRepository for SqliteStore {
    fn save(&self, encounter: &Encounter) -> Result<(), RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        insert_encounter(&tx, encounter)?;
        enqueue(
            &tx,
            "encounter",
            encounter.id,
            "upsert",
            encounter,
            encounter.updated_at,
        )?;
        tx.commit().map_err(repo_error)
    }

    fn list_for_word(&self, word_id: Uuid) -> Result<Vec<Encounter>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, word_id, selected_text, sentence, source_app, source_title,
                 source_url, capture_origin, captured_at, updated_at, deleted_at FROM encounters
                 WHERE word_id = ?1 AND deleted_at IS NULL ORDER BY captured_at DESC",
            )
            .map_err(repo_error)?;
        statement
            .query_map([word_id.to_string()], map_encounter)
            .map_err(repo_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(repo_error)
    }

    fn soft_delete(&self, id: Uuid, deleted_at: DateTime<Utc>) -> Result<(), RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        let changed = tx
            .execute(
                "UPDATE encounters SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
                params![id.to_string(), deleted_at.to_rfc3339()],
            )
            .map_err(repo_error)?;
        if changed == 0 {
            return Err(RepositoryError::NotFound);
        }
        enqueue(&tx, "encounter", id, "delete", &id, deleted_at)?;
        tx.commit().map_err(repo_error)
    }
}

impl SettingsRepository for SqliteStore {
    fn get(&self) -> Result<UserSettings, RepositoryError> {
        let payload: Option<String> = self
            .lock()?
            .query_row(
                "SELECT payload FROM user_settings WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(repo_error)?;
        payload.map_or_else(
            || Ok(UserSettings::default()),
            |json| serde_json::from_str(&json).map_err(repo_error),
        )
    }

    fn save(&self, settings: &UserSettings) -> Result<(), RepositoryError> {
        self.lock()?
            .execute(
                "INSERT INTO user_settings(singleton, payload) VALUES(1, ?1)
                 ON CONFLICT(singleton) DO UPDATE SET payload = excluded.payload",
                [serde_json::to_string(settings).map_err(repo_error)?],
            )
            .map_err(repo_error)?;
        Ok(())
    }
}

impl ReviewRepository for SqliteStore {
    fn append(&self, review: &ReviewLog) -> Result<(), RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        tx.execute(
            "INSERT INTO review_logs(id, word_id, rating, reviewed_at, received_at, device_id, result_payload)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                review.id.to_string(),
                review.word_id.to_string(),
                enum_json(&review.rating)?,
                review.reviewed_at.to_rfc3339(),
                review.received_at.to_rfc3339(),
                review.device_id.to_string(),
                review.result.as_ref().map(serde_json::to_string).transpose().map_err(repo_error)?,
            ],
        )
        .map_err(repo_error)?;
        enqueue(
            &tx,
            "review_log",
            review.id,
            "upsert",
            review,
            review.reviewed_at,
        )?;
        tx.commit().map_err(repo_error)
    }

    fn list_for_word(&self, word_id: Uuid) -> Result<Vec<ReviewLog>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, word_id, rating, reviewed_at, received_at, device_id, result_payload
                 FROM review_logs WHERE word_id = ?1 ORDER BY reviewed_at",
            )
            .map_err(repo_error)?;
        statement
            .query_map([word_id.to_string()], |row| {
                Ok(ReviewLog {
                    id: parse_uuid(row.get::<_, String>(0)?)?,
                    word_id: parse_uuid(row.get::<_, String>(1)?)?,
                    rating: parse_json(&row.get::<_, String>(2)?)?,
                    reviewed_at: parse_time(row.get::<_, String>(3)?)?,
                    received_at: parse_time(row.get::<_, String>(4)?)?,
                    device_id: parse_uuid(row.get::<_, String>(5)?)?,
                    result: row
                        .get::<_, Option<String>>(6)?
                        .map(|payload| parse_json(&payload))
                        .transpose()?,
                })
            })
            .map_err(repo_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(repo_error)
    }
}

fn insert_word(tx: &Transaction<'_>, key: &str, word: &Word) -> Result<(), RepositoryError> {
    tx.execute(
        "INSERT INTO words(id, dedupe_key, owner_scope, lemma, display_form, source_language,
         target_language, translation, part_of_speech, status, created_at, updated_at, deleted_at,
         review_state_json) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(id) DO UPDATE SET display_form=excluded.display_form,
         translation=excluded.translation, part_of_speech=excluded.part_of_speech,
         status=excluded.status, updated_at=excluded.updated_at, deleted_at=excluded.deleted_at,
         review_state_json=excluded.review_state_json",
        params![
            word.id.to_string(),
            key,
            enum_json(&word.owner_scope)?,
            word.lemma,
            word.display_form,
            word.source_language,
            word.target_language,
            word.translation,
            word.part_of_speech,
            enum_json(&word.status)?,
            word.created_at.to_rfc3339(),
            word.updated_at.to_rfc3339(),
            word.deleted_at.map(|time| time.to_rfc3339()),
            serde_json::to_string(&word.review_state).map_err(repo_error)?,
        ],
    )
    .map_err(repo_error)?;
    Ok(())
}

fn find_word(connection: &Connection, key: &str) -> Result<Option<Word>, RepositoryError> {
    connection
        .query_row(
            "SELECT id, owner_scope, lemma, display_form, source_language, target_language,
             translation, part_of_speech, status, created_at, updated_at, deleted_at,
             review_state_json FROM words WHERE dedupe_key = ?1 AND deleted_at IS NULL",
            [key],
            map_word,
        )
        .optional()
        .map_err(repo_error)
}

fn map_word(row: &rusqlite::Row<'_>) -> rusqlite::Result<Word> {
    Ok(Word {
        id: parse_uuid(row.get::<_, String>(0)?)?,
        owner_scope: parse_json(&row.get::<_, String>(1)?)?,
        lemma: row.get(2)?,
        display_form: row.get(3)?,
        source_language: row.get(4)?,
        target_language: row.get(5)?,
        translation: row.get(6)?,
        part_of_speech: row.get(7)?,
        status: parse_json(&row.get::<_, String>(8)?)?,
        created_at: parse_time(row.get::<_, String>(9)?)?,
        updated_at: parse_time(row.get::<_, String>(10)?)?,
        deleted_at: row
            .get::<_, Option<String>>(11)?
            .map(parse_time)
            .transpose()?,
        review_state: parse_json(&row.get::<_, String>(12)?)?,
    })
}

fn insert_encounter(tx: &Transaction<'_>, encounter: &Encounter) -> Result<(), RepositoryError> {
    tx.execute(
        "INSERT INTO encounters(id, word_id, selected_text, sentence, source_app, source_title,
         source_url, capture_origin, captured_at, updated_at, deleted_at) VALUES(?1, ?2, ?3, ?4,
         ?5, ?6, ?7, ?8, ?9, ?10, ?11) ON CONFLICT(id) DO UPDATE SET sentence=excluded.sentence,
         updated_at=excluded.updated_at, deleted_at=excluded.deleted_at",
        params![
            encounter.id.to_string(),
            encounter.word_id.to_string(),
            encounter.selected_text,
            encounter.sentence,
            encounter.source_app,
            encounter.source_title,
            encounter.source_url,
            enum_json(&encounter.capture_origin)?,
            encounter.captured_at.to_rfc3339(),
            encounter.updated_at.to_rfc3339(),
            encounter.deleted_at.map(|time| time.to_rfc3339()),
        ],
    )
    .map_err(repo_error)?;
    Ok(())
}

fn map_encounter(row: &rusqlite::Row<'_>) -> rusqlite::Result<Encounter> {
    Ok(Encounter {
        id: parse_uuid(row.get::<_, String>(0)?)?,
        word_id: parse_uuid(row.get::<_, String>(1)?)?,
        selected_text: row.get(2)?,
        sentence: row.get(3)?,
        source_app: row.get(4)?,
        source_title: row.get(5)?,
        source_url: row.get(6)?,
        capture_origin: parse_json(&row.get::<_, String>(7)?)?,
        captured_at: parse_time(row.get::<_, String>(8)?)?,
        updated_at: parse_time(row.get::<_, String>(9)?)?,
        deleted_at: row
            .get::<_, Option<String>>(10)?
            .map(parse_time)
            .transpose()?,
    })
}

fn enqueue(
    tx: &Transaction<'_>,
    entity_type: &str,
    entity_id: Uuid,
    operation: &str,
    payload: &impl Serialize,
    created_at: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    tx.execute(
        "INSERT INTO outbox(mutation_id, entity_type, entity_id, operation, payload, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            Uuid::now_v7().to_string(),
            entity_type,
            entity_id.to_string(),
            operation,
            serde_json::to_string(payload).map_err(repo_error)?,
            created_at.to_rfc3339(),
        ],
    )
    .map_err(repo_error)?;
    Ok(())
}

fn enum_json(value: &impl Serialize) -> Result<String, RepositoryError> {
    serde_json::to_string(value).map_err(repo_error)
}

fn parse_json<T: serde::de::DeserializeOwned>(value: &str) -> rusqlite::Result<T> {
    serde_json::from_str(value).map_err(sql_conversion_error)
}

fn parse_uuid(value: String) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&value).map_err(sql_conversion_error)
}

fn parse_time(value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(sql_conversion_error)
}

fn sql_conversion_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn repo_error(error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Persistence(error.to_string())
}
