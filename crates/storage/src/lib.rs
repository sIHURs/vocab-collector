#![forbid(unsafe_code)]

//! SQLite persistence and transactional outbox support.

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Days, Local, Months, NaiveDate, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use uuid::Uuid;
use vocab_domain::{
    CaptureOrigin, Encounter, EncounterRepository, OwnerScope, RepositoryError, ReviewLog,
    ReviewRepository, ReviewState, SettingsRepository, UserSettings, Word, WordRepository,
    WordStatus, dedupe_key, normalize_lemma,
};

mod capture_undo;
mod identity;
mod references;
mod translations;

const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS words (
  id TEXT PRIMARY KEY, dedupe_key TEXT NOT NULL UNIQUE, owner_scope TEXT NOT NULL,
  lemma TEXT NOT NULL, display_form TEXT NOT NULL, source_language TEXT NOT NULL,
  target_language TEXT NOT NULL, translation TEXT, part_of_speech TEXT, status TEXT NOT NULL,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT,
  mastered_at TEXT, achieved_at TEXT, delete_after TEXT, review_state_json TEXT
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
CREATE TABLE IF NOT EXISTS lifetime_archive (
  singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
  vocabulary_count INTEGER NOT NULL DEFAULT 0,
  encounter_count INTEGER NOT NULL DEFAULT 0,
  review_count INTEGER NOT NULL DEFAULT 0,
  remembered_count INTEGER NOT NULL DEFAULT 0,
  forgotten_count INTEGER NOT NULL DEFAULT 0,
  rating_breakdown_complete INTEGER NOT NULL DEFAULT 1
);
PRAGMA user_version = 5;
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LifetimeStatistics {
    pub current_vocabulary_count: usize,
    pub current_achieved_count: usize,
    pub active_encounter_count: usize,
    pub review_count: usize,
    pub remembered_count: usize,
    pub forgotten_count: usize,
    pub archived_vocabulary_count: usize,
    pub archived_encounter_count: usize,
    pub archived_review_count: usize,
    pub archived_remembered_count: usize,
    pub archived_forgotten_count: usize,
    pub rating_breakdown_complete: bool,
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
    pub archived_vocabulary_count: usize,
    pub archived_encounter_count: usize,
    pub archived_review_count: usize,
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
    local_date: Arc<dyn Fn() -> NaiveDate + Send + Sync>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RepositoryError> {
        Self::from_connection(Connection::open(path).map_err(repo_error)?)
    }

    pub fn open_in_memory() -> Result<Self, RepositoryError> {
        Self::from_connection(Connection::open_in_memory().map_err(repo_error)?)
    }

    /// Inject the system-local calendar date at the persistence boundary.
    /// Capture timestamps may come from callers and are not the save date.
    pub fn open_with_local_date(
        path: impl AsRef<Path>,
        local_date: Arc<dyn Fn() -> NaiveDate + Send + Sync>,
    ) -> Result<Self, RepositoryError> {
        Self::from_connection_with_date(Connection::open(path).map_err(repo_error)?, local_date)
    }

    fn from_connection(connection: Connection) -> Result<Self, RepositoryError> {
        Self::from_connection_with_date(connection, Arc::new(|| Local::now().date_naive()))
    }

    fn from_connection_with_date(
        connection: Connection,
        local_date: Arc<dyn Fn() -> NaiveDate + Send + Sync>,
    ) -> Result<Self, RepositoryError> {
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(repo_error)?;
        if version > 8 {
            return Err(RepositoryError::Persistence(
                "Database was created by a newer app version".into(),
            ));
        }
        connection
            .execute_batch("PRAGMA foreign_keys = ON; SAVEPOINT schema_upgrade;")
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
        let lifecycle_columns = {
            let mut statement = connection
                .prepare("PRAGMA table_info(words)")
                .map_err(repo_error)?;
            statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(repo_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(repo_error)?
        };
        if !lifecycle_columns
            .iter()
            .any(|column| column == "mastered_at")
        {
            connection
                .execute_batch(
                    "ALTER TABLE words ADD COLUMN mastered_at TEXT;
                     ALTER TABLE words ADD COLUMN achieved_at TEXT;
                     ALTER TABLE words ADD COLUMN delete_after TEXT;
                     CREATE INDEX IF NOT EXISTS idx_words_achieved_deadline ON words(delete_after);
                     CREATE INDEX IF NOT EXISTS idx_words_mastered_at ON words(mastered_at);
                     PRAGMA user_version = 4;",
                )
                .map_err(repo_error)?;
            let migrated_at = Utc::now().to_rfc3339();
            connection
                .execute(
                    "UPDATE words SET mastered_at = ?1 WHERE status = '\"mastered\"' AND deleted_at IS NULL",
                    [migrated_at],
                )
                .map_err(repo_error)?;
        }
        connection
            .execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_words_achieved_deadline ON words(delete_after);
             CREATE INDEX IF NOT EXISTS idx_words_mastered_at ON words(mastered_at);
             CREATE TABLE IF NOT EXISTS lifetime_archive (
               singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
               vocabulary_count INTEGER NOT NULL DEFAULT 0,
               encounter_count INTEGER NOT NULL DEFAULT 0,
               review_count INTEGER NOT NULL DEFAULT 0
             );",
            )
            .map_err(repo_error)?;
        let archive_columns = {
            let mut statement = connection
                .prepare("PRAGMA table_info(lifetime_archive)")
                .map_err(repo_error)?;
            statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(repo_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(repo_error)?
        };
        if !archive_columns
            .iter()
            .any(|column| column == "remembered_count")
        {
            connection
                .execute_batch(
                    "ALTER TABLE lifetime_archive ADD COLUMN remembered_count INTEGER NOT NULL DEFAULT 0;
                     ALTER TABLE lifetime_archive ADD COLUMN forgotten_count INTEGER NOT NULL DEFAULT 0;
                     ALTER TABLE lifetime_archive ADD COLUMN rating_breakdown_complete INTEGER NOT NULL DEFAULT 1;
                     UPDATE lifetime_archive SET rating_breakdown_complete = CASE WHEN review_count = 0 THEN 1 ELSE 0 END;",
                )
                .map_err(repo_error)?;
        }
        connection
            .execute_batch("PRAGMA user_version = 5;")
            .map_err(repo_error)?;
        migrate_vocabulary_log(&connection, local_date())?;
        translations::migrate(&connection)?;
        identity::migrate(&connection)?;
        capture_undo::migrate(&connection)?;
        connection
            .execute_batch("RELEASE schema_upgrade;")
            .map_err(repo_error)?;
        Ok(Self {
            connection: Mutex::new(connection),
            local_date,
        })
    }

    pub fn capture(&self, input: &CaptureRecord) -> Result<StoredCapture, RepositoryError> {
        self.capture_with_expected(input, None)
    }

    fn capture_with_expected(
        &self,
        input: &CaptureRecord,
        expected: Option<Uuid>,
    ) -> Result<StoredCapture, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(repo_error)?;
        let key = dedupe_key(&input.lemma, &input.source_language);
        if let Some(id) = expected
            && identity::preview(&transaction, input)?.is_none_or(|w| w.id != id)
        {
            return Err(RepositoryError::Persistence(
                "Vocabulary Item changed; recheck the capture before saving".into(),
            ));
        }
        let before = capture_undo::before(&transaction, input)?;
        let existing = identity::resolve(&transaction, input)?;
        if expected.is_none() && existing.as_ref().is_some_and(Word::is_achieved) {
            return Err(RepositoryError::Achieved);
        }
        let is_existing_word = existing.is_some();
        let mut word = existing.unwrap_or_else(|| Word {
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
            mastered_at: None,
            achieved_at: None,
            delete_after: None,
            review_state: Some(ReviewState::initial(input.captured_at)),
        });
        if word.is_achieved() {
            word.unmaster(input.captured_at, WordStatus::Learning);
            word.review_state = Some(ReviewState::initial(input.captured_at));
        }
        if !is_existing_word {
            insert_word(&transaction, &key, &word)?;
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
        translations::save(&transaction, &mut word, &mut encounter, input)?;
        insert_word(
            &transaction,
            &dedupe_key(&word.lemma, &word.source_language),
            &word,
        )?;
        enqueue(
            &transaction,
            "word",
            word.id,
            "upsert",
            &word,
            input.captured_at,
        )?;
        insert_encounter(&transaction, &encounter)?;
        record_daily_capture(&transaction, encounter.id, (self.local_date)())?;
        enqueue(
            &transaction,
            "encounter",
            encounter.id,
            "upsert",
            &encounter,
            input.captured_at,
        )?;
        capture_undo::record(&transaction, &encounter, &before)?;
        transaction.commit().map_err(repo_error)?;
        Ok(StoredCapture {
            word,
            encounter,
            is_existing_word,
        })
    }

    pub fn restore_achieved_and_capture(
        &self,
        expected_word_id: Uuid,
        input: &CaptureRecord,
    ) -> Result<StoredCapture, RepositoryError> {
        self.capture_with_expected(input, Some(expected_word_id))
    }

    pub fn lifetime_statistics(&self) -> Result<LifetimeStatistics, RepositoryError> {
        let connection = self.lock()?;
        let count = |sql: &str| -> Result<usize, RepositoryError> {
            let value: i64 = connection
                .query_row(sql, [], |row| row.get(0))
                .map_err(repo_error)?;
            usize::try_from(value).map_err(repo_error)
        };
        let archived = connection
            .query_row(
                "SELECT vocabulary_count, encounter_count, review_count, remembered_count, forgotten_count, rating_breakdown_complete
                 FROM lifetime_archive WHERE singleton = 1",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?, row.get::<_, i64>(4)?, row.get::<_, bool>(5)?)),
            )
            .optional()
            .map_err(repo_error)?
            .unwrap_or((0, 0, 0, 0, 0, true));
        Ok(LifetimeStatistics {
            current_vocabulary_count: count("SELECT COUNT(*) FROM words WHERE deleted_at IS NULL")?,
            current_achieved_count: count(
                "SELECT COUNT(*) FROM words WHERE deleted_at IS NULL AND achieved_at IS NOT NULL AND delete_after IS NOT NULL",
            )?,
            active_encounter_count: count(
                "SELECT COUNT(*) FROM encounters WHERE deleted_at IS NULL",
            )?,
            review_count: count("SELECT COUNT(*) FROM review_logs")?,
            remembered_count: count(
                "SELECT COUNT(*) FROM review_logs WHERE rating = '\"remembered\"'",
            )?,
            forgotten_count: count("SELECT COUNT(*) FROM review_logs WHERE rating = '\"forgot\"'")?,
            archived_vocabulary_count: usize::try_from(archived.0).map_err(repo_error)?,
            archived_encounter_count: usize::try_from(archived.1).map_err(repo_error)?,
            archived_review_count: usize::try_from(archived.2).map_err(repo_error)?,
            archived_remembered_count: usize::try_from(archived.3).map_err(repo_error)?,
            archived_forgotten_count: usize::try_from(archived.4).map_err(repo_error)?,
            rating_breakdown_complete: archived.5,
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
        let archived = connection.query_row(
            "SELECT vocabulary_count, encounter_count, review_count FROM lifetime_archive WHERE singleton = 1",
            [], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)),
        ).optional().map_err(repo_error)?.unwrap_or((0, 0, 0));

        Ok(DatabaseSummary {
            schema_version,
            foreign_keys_enabled,
            word_count: count("SELECT COUNT(*) FROM words WHERE deleted_at IS NULL")?,
            active_encounter_count: count(
                "SELECT COUNT(*) FROM encounters WHERE deleted_at IS NULL",
            )?,
            review_log_count: count("SELECT COUNT(*) FROM review_logs")?,
            pending_outbox_count: count("SELECT COUNT(*) FROM outbox")?,
            archived_vocabulary_count: usize::try_from(archived.0).map_err(repo_error)?,
            archived_encounter_count: usize::try_from(archived.1).map_err(repo_error)?,
            archived_review_count: usize::try_from(archived.2).map_err(repo_error)?,
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
        let key = dedupe_key(&word.lemma, &word.source_language);
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

    pub fn unachieve_words(
        &self,
        ids: &[Uuid],
        now: DateTime<Utc>,
    ) -> Result<usize, RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        let mut words = Vec::with_capacity(ids.len());
        for id in ids {
            let mut word = get_word_by_id(&tx, *id)?.ok_or(RepositoryError::NotFound)?;
            word.unachieve(now)
                .map_err(|error| RepositoryError::Persistence(error.to_string()))?;
            words.push(word);
        }
        for word in &words {
            let key = dedupe_key(&word.lemma, &word.source_language);
            insert_word(&tx, &key, word)?;
            enqueue(&tx, "word", word.id, "upsert", word, now)?;
        }
        tx.commit().map_err(repo_error)?;
        Ok(words.len())
    }

    pub fn delete_achieved_words(
        &self,
        ids: &[Uuid],
        now: DateTime<Utc>,
    ) -> Result<usize, RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        for id in ids {
            let word = get_word_by_id(&tx, *id)?.ok_or(RepositoryError::NotFound)?;
            if !word.is_achieved() {
                return Err(RepositoryError::Persistence(
                    "only Achieved Vocabulary Items can be permanently deleted".into(),
                ));
            }
        }
        for id in ids {
            let encounter_count: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM encounters WHERE word_id = ?1",
                    [id.to_string()],
                    |row| row.get(0),
                )
                .map_err(repo_error)?;
            let review_count: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM review_logs WHERE word_id = ?1",
                    [id.to_string()],
                    |row| row.get(0),
                )
                .map_err(repo_error)?;
            let remembered_count: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM review_logs WHERE word_id = ?1 AND rating = '\"remembered\"'",
                    [id.to_string()],
                    |row| row.get(0),
                )
                .map_err(repo_error)?;
            let forgotten_count: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM review_logs WHERE word_id = ?1 AND rating = '\"forgot\"'",
                    [id.to_string()],
                    |row| row.get(0),
                )
                .map_err(repo_error)?;
            tx.execute(
                "INSERT INTO lifetime_archive(singleton, vocabulary_count, encounter_count, review_count, remembered_count, forgotten_count) VALUES(1,1,?1,?2,?3,?4)
                 ON CONFLICT(singleton) DO UPDATE SET vocabulary_count=vocabulary_count+1,
                 encounter_count=encounter_count+excluded.encounter_count, review_count=review_count+excluded.review_count,
                 remembered_count=remembered_count+excluded.remembered_count, forgotten_count=forgotten_count+excluded.forgotten_count",
                params![encounter_count, review_count, remembered_count, forgotten_count],
            ).map_err(repo_error)?;
            tx.execute(
                "DELETE FROM review_logs WHERE word_id = ?1",
                [id.to_string()],
            )
            .map_err(repo_error)?;
            tx.execute(
                "DELETE FROM encounters WHERE word_id = ?1",
                [id.to_string()],
            )
            .map_err(repo_error)?;
            tx.execute("DELETE FROM words WHERE id = ?1", [id.to_string()])
                .map_err(repo_error)?;
            enqueue(&tx, "word", *id, "delete", id, now)?;
        }
        tx.commit().map_err(repo_error)?;
        Ok(ids.len())
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
                 mastered_at, achieved_at, delete_after, review_state_json FROM words WHERE id = ?1",
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
                 mastered_at, achieved_at, delete_after, review_state_json FROM words WHERE deleted_at IS NULL ORDER BY updated_at DESC",
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
        let key = dedupe_key(&word.lemma, &word.source_language);
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
                 source_url, capture_origin, captured_at, updated_at, deleted_at, saved_translation_json FROM encounters
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
        let before = capture_undo::check(&tx, id)?;
        let changed = tx
            .execute(
                "UPDATE encounters SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1 AND deleted_at IS NULL",
                params![id.to_string(), deleted_at.to_rfc3339()],
            )
            .map_err(repo_error)?;
        if changed == 0 {
            return Err(RepositoryError::NotFound);
        }
        let saved_date: Option<String> = tx
            .query_row(
                "SELECT saved_local_date FROM encounters WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .map_err(repo_error)?;
        if let Some(date) = saved_date {
            let changed = tx.execute("UPDATE vocabulary_daily_counts SET count = count - 1 WHERE date = ?1 AND count > 0", [date]).map_err(repo_error)?;
            if changed != 1 {
                return Err(RepositoryError::Persistence(
                    "daily capture count is inconsistent".into(),
                ));
            }
        }
        if let Some(before) = before {
            capture_undo::restore(&tx, id, before, deleted_at)?;
        } else {
            translations::undo(&tx, id, deleted_at)?;
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

fn insert_word(tx: &Connection, key: &str, word: &Word) -> Result<(), RepositoryError> {
    let key = identity::scoped_key(word, key)?;
    tx.execute(
        "INSERT INTO words(id, dedupe_key, owner_scope, lemma, display_form, source_language,
         target_language, translation, part_of_speech, status, created_at, updated_at, deleted_at,
         mastered_at, achieved_at, delete_after, review_state_json) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
         ON CONFLICT(id) DO UPDATE SET dedupe_key=excluded.dedupe_key, lemma=excluded.lemma, source_language=excluded.source_language, display_form=excluded.display_form,
         target_language=excluded.target_language, translation=excluded.translation, part_of_speech=excluded.part_of_speech,
         status=excluded.status, updated_at=excluded.updated_at, deleted_at=excluded.deleted_at,
         mastered_at=excluded.mastered_at, achieved_at=excluded.achieved_at,
         delete_after=excluded.delete_after, review_state_json=excluded.review_state_json",
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
            word.mastered_at.map(|time| time.to_rfc3339()),
            word.achieved_at.map(|time| time.to_rfc3339()),
            word.delete_after.map(|time| time.to_rfc3339()),
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
             mastered_at, achieved_at, delete_after, review_state_json FROM words WHERE dedupe_key = ?1 AND deleted_at IS NULL",
            [format!("{}|{key}", enum_json(&OwnerScope::Guest)?)],
            map_word,
        )
        .optional()
        .map_err(repo_error)
}

fn get_word_by_id(connection: &Connection, id: Uuid) -> Result<Option<Word>, RepositoryError> {
    connection
        .query_row(
            "SELECT id, owner_scope, lemma, display_form, source_language, target_language,
         translation, part_of_speech, status, created_at, updated_at, deleted_at,
         mastered_at, achieved_at, delete_after, review_state_json FROM words WHERE id = ?1",
            [id.to_string()],
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
        mastered_at: row
            .get::<_, Option<String>>(12)?
            .map(parse_time)
            .transpose()?,
        achieved_at: row
            .get::<_, Option<String>>(13)?
            .map(parse_time)
            .transpose()?,
        delete_after: row
            .get::<_, Option<String>>(14)?
            .map(parse_time)
            .transpose()?,
        review_state: parse_json(&row.get::<_, String>(15)?)?,
    })
}

fn insert_encounter(tx: &Connection, encounter: &Encounter) -> Result<(), RepositoryError> {
    tx.execute(
        "INSERT INTO encounters(id, word_id, selected_text, sentence, source_app, source_title,
         source_url, capture_origin, captured_at, updated_at, deleted_at, saved_translation_json) VALUES(?1, ?2, ?3, ?4,
         ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) ON CONFLICT(id) DO UPDATE SET sentence=excluded.sentence,
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
            encounter.saved_translation.as_ref().map(serde_json::to_string).transpose().map_err(repo_error)?,
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
        saved_translation: row
            .get::<_, Option<String>>(11)?
            .map(|v| parse_json(&v))
            .transpose()?,
        captured_at: parse_time(row.get::<_, String>(8)?)?,
        updated_at: parse_time(row.get::<_, String>(9)?)?,
        deleted_at: row
            .get::<_, Option<String>>(10)?
            .map(parse_time)
            .transpose()?,
    })
}

fn enqueue(
    tx: &Connection,
    entity_type: &str,
    entity_id: Uuid,
    operation: &str,
    payload: &impl Serialize,
    created_at: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    let mut serialized = serde_json::to_value(payload).map_err(repo_error)?;
    if entity_type == "word" && operation == "upsert" {
        serialized["translations"] =
            serde_json::to_value(translations::list(tx, entity_id)?).map_err(repo_error)?;
    }
    tx.execute(
        "INSERT INTO outbox(mutation_id, entity_type, entity_id, operation, payload, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            Uuid::now_v7().to_string(),
            entity_type,
            entity_id.to_string(),
            operation,
            serde_json::to_string(&serialized).map_err(repo_error)?,
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

fn migrate_vocabulary_log(
    connection: &Connection,
    today: NaiveDate,
) -> Result<(), RepositoryError> {
    // Metadata and the nullable original save date are installed atomically.
    // Do not backfill from surviving encounters: deleted history is unknowable.
    let tx = connection;
    let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM pragma_table_info('encounters') WHERE name = 'saved_local_date')", [], |row| row.get(0)).map_err(repo_error)?;
    if !exists {
        tx.execute_batch("ALTER TABLE encounters ADD COLUMN saved_local_date TEXT;")
            .map_err(repo_error)?;
    }
    tx.execute_batch("CREATE TABLE IF NOT EXISTS vocabulary_daily_counts (date TEXT PRIMARY KEY, count INTEGER NOT NULL CHECK(count >= 0));
        CREATE TABLE IF NOT EXISTS vocabulary_log_coverage (singleton INTEGER PRIMARY KEY CHECK(singleton = 1), started_on TEXT NOT NULL);").map_err(repo_error)?;
    tx.execute(
        "INSERT OR IGNORE INTO vocabulary_log_coverage VALUES (1, ?1)",
        [today.to_string()],
    )
    .map_err(repo_error)?;
    tx.execute_batch("PRAGMA user_version = 6;")
        .map_err(repo_error)?;
    Ok(())
}

fn record_daily_capture(
    tx: &Connection,
    encounter_id: Uuid,
    date: NaiveDate,
) -> Result<(), RepositoryError> {
    tx.execute(
        "UPDATE encounters SET saved_local_date = ?2 WHERE id = ?1",
        params![encounter_id.to_string(), date.to_string()],
    )
    .map_err(repo_error)?;
    tx.execute("INSERT INTO vocabulary_daily_counts(date, count) VALUES (?1, 1) ON CONFLICT(date) DO UPDATE SET count = count + 1", [date.to_string()]).map_err(repo_error)?;
    Ok(())
}

impl vocab_domain::VocabularyLogRepository for SqliteStore {
    fn vocabulary_log(&self) -> Result<vocab_domain::VocabularyLog, RepositoryError> {
        use vocab_domain::{LogCoverage, VocabularyLog, VocabularyLogDay};
        let end_date = (self.local_date)();
        let start_date = end_date
            .checked_sub_months(Months::new(12))
            .and_then(|d| d.checked_add_days(Days::new(1)))
            .ok_or_else(|| RepositoryError::Persistence("invalid calendar range".into()))?;
        let connection = self.lock()?;
        let started: String = connection
            .query_row(
                "SELECT started_on FROM vocabulary_log_coverage WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .map_err(repo_error)?;
        let started: NaiveDate = started.parse().map_err(repo_error)?;
        let mut statement = connection
            .prepare("SELECT date, count FROM vocabulary_daily_counts WHERE date BETWEEN ?1 AND ?2")
            .map_err(repo_error)?;
        let counts = statement
            .query_map(
                params![start_date.to_string(), end_date.to_string()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?)),
            )
            .map_err(repo_error)?
            .collect::<Result<std::collections::HashMap<_, _>, _>>()
            .map_err(repo_error)?;
        let days = start_date
            .iter_days()
            .take_while(|date| *date <= end_date)
            .map(|date| {
                let recorded = counts.get(&date.to_string()).copied();
                let coverage = if date < started {
                    if recorded.is_some() {
                        LogCoverage::Partial
                    } else {
                        LogCoverage::Unknown
                    }
                } else if date == started {
                    LogCoverage::Partial
                } else {
                    LogCoverage::Complete
                };
                let count = if coverage == LogCoverage::Unknown {
                    None
                } else {
                    Some(recorded.unwrap_or(0))
                };
                VocabularyLogDay {
                    date,
                    count,
                    coverage,
                }
            })
            .collect();
        Ok(VocabularyLog {
            start_date,
            end_date,
            days,
        })
    }
}
