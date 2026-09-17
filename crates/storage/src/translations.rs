use super::*;
use vocab_domain::SavedTranslation;

pub(super) fn migrate(connection: &Connection) -> Result<(), RepositoryError> {
    let columns = connection
        .prepare("PRAGMA table_info(encounters)")
        .map_err(repo_error)?
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(repo_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(repo_error)?;
    if !columns.iter().any(|name| name == "saved_translation_json") {
        let tx = connection;
        tx.execute_batch("ALTER TABLE encounters ADD COLUMN saved_translation_json TEXT;
            CREATE TABLE translation_history (
                event_id TEXT PRIMARY KEY, word_id TEXT NOT NULL REFERENCES words(id) ON DELETE CASCADE,
                target_language TEXT NOT NULL, text TEXT NOT NULL, saved_at TEXT NOT NULL
            );
            CREATE INDEX translation_word ON translation_history(word_id);
            INSERT INTO translation_history(event_id,word_id,target_language,text,saved_at)
                SELECT 'legacy:' || id,id,lower(target_language),translation,updated_at FROM words
                WHERE translation IS NOT NULL AND trim(translation) <> '' ORDER BY updated_at,id;").map_err(repo_error)?;
    }
    connection
        .execute_batch("PRAGMA user_version = 7;")
        .map_err(repo_error)
}

impl SqliteStore {
    pub fn translations(&self, word_id: Uuid) -> Result<Vec<SavedTranslation>, RepositoryError> {
        list(&*self.lock()?, word_id)
    }
}

pub(super) fn list(
    connection: &Connection,
    word_id: Uuid,
) -> Result<Vec<SavedTranslation>, RepositoryError> {
    let mut statement = connection
        .prepare(
            "SELECT target_language,text,saved_at FROM translation_history
        WHERE word_id=?1 ORDER BY rowid DESC",
        )
        .map_err(repo_error)?;
    let all = statement
        .query_map([word_id.to_string()], |row| {
            Ok(SavedTranslation {
                target_language: row.get(0)?,
                text: row.get(1)?,
                saved_at: parse_time(row.get(2)?)?,
            })
        })
        .map_err(repo_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(repo_error)?;
    let mut seen = std::collections::HashSet::new();
    Ok(all
        .into_iter()
        .filter(|value| seen.insert(value.target_language.clone()))
        .collect())
}

pub(super) fn save(
    tx: &Connection,
    word: &mut Word,
    encounter: &mut Encounter,
    input: &CaptureRecord,
) -> Result<(), RepositoryError> {
    let Some(text) = input
        .translation
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    else {
        return Ok(());
    };
    let value = SavedTranslation {
        target_language: input.target_language.trim().to_lowercase(),
        text: text.into(),
        saved_at: Utc::now(),
    };
    tx.execute("INSERT INTO translation_history(event_id,word_id,target_language,text,saved_at) VALUES(?1,?2,?3,?4,?5)",
        params![encounter.id.to_string(),word.id.to_string(),value.target_language,value.text,value.saved_at.to_rfc3339()]).map_err(repo_error)?;
    word.target_language.clone_from(&value.target_language);
    word.translation = Some(value.text.clone());
    word.updated_at = input.captured_at;
    encounter.saved_translation = Some(value);
    Ok(())
}

pub(super) fn undo(
    tx: &Connection,
    encounter_id: Uuid,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    let word_id: String = tx
        .query_row(
            "SELECT word_id FROM encounters WHERE id=?1",
            [encounter_id.to_string()],
            |row| row.get(0),
        )
        .map_err(repo_error)?;
    let word_id = Uuid::parse_str(&word_id).map_err(repo_error)?;
    tx.execute(
        "DELETE FROM translation_history WHERE event_id=?1",
        [encounter_id.to_string()],
    )
    .map_err(repo_error)?;
    let mut word = get_word_by_id(tx, word_id)?.ok_or(RepositoryError::NotFound)?;
    let latest = list(tx, word_id)?.into_iter().next();
    word.translation = latest.as_ref().map(|v| v.text.clone());
    if let Some(value) = latest {
        word.target_language = value.target_language;
    }
    word.updated_at = now;
    insert_word(tx, &dedupe_key(&word.lemma, &word.source_language), &word)?;
    enqueue(tx, "word", word.id, "upsert", &word, now)
}
