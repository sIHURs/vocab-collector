use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vocab_domain::{RepositoryError, ReviewState, WordStatus, dedupe_key};

use crate::{SqliteStore, enqueue, get_word_by_id, insert_word, repo_error};

pub(super) fn migrate(connection: &Connection) -> Result<(), RepositoryError> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS learning_status_undo (
        token TEXT PRIMARY KEY, word_id TEXT NOT NULL UNIQUE REFERENCES words(id) ON DELETE CASCADE,
        revision INTEGER NOT NULL, before_json TEXT NOT NULL
    ); PRAGMA user_version = 9;",
        )
        .map_err(repo_error)
}

#[derive(Serialize, Deserialize)]
struct LearningSnapshot {
    status: WordStatus,
    review_state: Option<ReviewState>,
    mastered_at: Option<DateTime<Utc>>,
}

impl SqliteStore {
    pub fn change_learning_status(
        &self,
        word_id: Uuid,
        status: WordStatus,
        now: DateTime<Utc>,
    ) -> Result<Option<Uuid>, RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        let mut word = get_word_by_id(&tx, word_id)?.ok_or(RepositoryError::NotFound)?;
        if word.deleted_at.is_some() {
            return Err(RepositoryError::NotFound);
        }
        if word.is_achieved() {
            return Err(RepositoryError::Achieved);
        }
        let before = LearningSnapshot {
            status: word.status,
            review_state: word.review_state.clone(),
            mastered_at: word.mastered_at,
        };
        let token = if word.change_learning_status(status, now) {
            insert_word(&tx, &dedupe_key(&word.lemma, &word.source_language), &word)?;
            enqueue(&tx, "word", word.id, "upsert", &word, now)?;
            let token = Uuid::now_v7();
            tx.execute("INSERT INTO learning_status_undo(token, word_id, revision, before_json)
                SELECT ?1, ?2, revision, ?3 FROM word_revisions WHERE word_id = ?2
                ON CONFLICT(word_id) DO UPDATE SET token=excluded.token, revision=excluded.revision, before_json=excluded.before_json",
                params![token.to_string(), word.id.to_string(), serde_json::to_string(&before).map_err(repo_error)?]).map_err(repo_error)?;
            Some(token)
        } else {
            None
        };
        tx.commit().map_err(repo_error)?;
        Ok(token)
    }

    pub fn undo_learning_status(
        &self,
        token: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Uuid, RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        let snapshot = tx.query_row("SELECT u.word_id, u.before_json, u.revision, r.revision
            FROM learning_status_undo u JOIN word_revisions r ON r.word_id=u.word_id WHERE u.token=?1",
            [token.to_string()], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?)))
            .optional().map_err(repo_error)?.ok_or(RepositoryError::StaleLearningStatusUndo)?;
        if snapshot.2 != snapshot.3 {
            return Err(RepositoryError::StaleLearningStatusUndo);
        }
        let word_id = Uuid::parse_str(&snapshot.0).map_err(repo_error)?;
        let mut word =
            get_word_by_id(&tx, word_id)?.ok_or(RepositoryError::StaleLearningStatusUndo)?;
        if word.deleted_at.is_some() || word.is_achieved() {
            return Err(RepositoryError::StaleLearningStatusUndo);
        }
        let before: LearningSnapshot = serde_json::from_str(&snapshot.1).map_err(repo_error)?;
        word.status = before.status;
        word.review_state = before.review_state;
        word.mastered_at = before.mastered_at;
        word.updated_at = now;
        insert_word(&tx, &dedupe_key(&word.lemma, &word.source_language), &word)?;
        enqueue(&tx, "word", word.id, "upsert", &word, now)?;
        tx.execute(
            "DELETE FROM learning_status_undo WHERE token=?1",
            [token.to_string()],
        )
        .map_err(repo_error)?;
        tx.commit().map_err(repo_error)?;
        Ok(word_id)
    }
}
