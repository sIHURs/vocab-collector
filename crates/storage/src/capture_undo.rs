use super::*;
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub(super) struct BeforeCapture {
    words: Vec<Word>,
    encounters: Vec<(String, String)>,
    reviews: Vec<(String, String)>,
    translations: Vec<(String, String)>,
}

pub(super) fn migrate(connection: &Connection) -> Result<(), RepositoryError> {
    connection.execute_batch("CREATE TABLE IF NOT EXISTS word_revisions(word_id TEXT PRIMARY KEY, revision INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS capture_undo(encounter_id TEXT PRIMARY KEY REFERENCES encounters(id) ON DELETE CASCADE,
        word_id TEXT NOT NULL, revision INTEGER NOT NULL, before_json TEXT NOT NULL);").map_err(repo_error)?;
    for (table, key) in [
        ("words", "id"),
        ("encounters", "word_id"),
        ("review_logs", "word_id"),
        ("translation_history", "word_id"),
    ] {
        for (operation, alias) in [("INSERT", "NEW"), ("UPDATE", "NEW"), ("DELETE", "OLD")] {
            connection.execute_batch(&format!("CREATE TRIGGER IF NOT EXISTS revision_{table}_{operation} AFTER {operation} ON {table}
                BEGIN INSERT INTO word_revisions(word_id,revision) VALUES({alias}.{key},1)
                ON CONFLICT(word_id) DO UPDATE SET revision=revision+1; END;")).map_err(repo_error)?;
        }
    }
    Ok(())
}

pub(super) fn before(
    connection: &Connection,
    input: &CaptureRecord,
) -> Result<BeforeCapture, RepositoryError> {
    let words = identity::candidates(connection, input)?;
    let references = |table: &str, key: &str| -> Result<Vec<(String, String)>, RepositoryError> {
        let mut result = Vec::new();
        for word in &words {
            let mut statement = connection
                .prepare(&format!(
                    "SELECT {key},word_id FROM {table} WHERE word_id=?1"
                ))
                .map_err(repo_error)?;
            result.extend(
                statement
                    .query_map([word.id.to_string()], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map_err(repo_error)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(repo_error)?,
            );
        }
        Ok(result)
    };
    Ok(BeforeCapture {
        encounters: references("encounters", "id")?,
        reviews: references("review_logs", "id")?,
        translations: references("translation_history", "event_id")?,
        words,
    })
}

pub(super) fn record(
    connection: &Connection,
    encounter: &Encounter,
    before: &BeforeCapture,
) -> Result<(), RepositoryError> {
    connection
        .execute(
            "INSERT INTO capture_undo(encounter_id,word_id,revision,before_json)
        SELECT ?1,?2,revision,?3 FROM word_revisions WHERE word_id=?2",
            params![
                encounter.id.to_string(),
                encounter.word_id.to_string(),
                serde_json::to_string(before).map_err(repo_error)?
            ],
        )
        .map_err(repo_error)?;
    Ok(())
}

pub(super) fn check(
    connection: &Connection,
    encounter_id: Uuid,
) -> Result<Option<BeforeCapture>, RepositoryError> {
    let saved: Option<(String, i64, i64)> = connection
        .query_row(
            "SELECT u.before_json,u.revision,r.revision
        FROM capture_undo u JOIN word_revisions r ON r.word_id=u.word_id WHERE u.encounter_id=?1",
            [encounter_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(repo_error)?;
    match saved {
        Some((_, expected, current)) if expected != current => Err(RepositoryError::Persistence(
            "Cannot undo: this Vocabulary Item changed after the capture. No changes were undone."
                .into(),
        )),
        Some((json, _, _)) => Ok(Some(serde_json::from_str(&json).map_err(repo_error)?)),
        None => Ok(None),
    }
}

pub(super) fn restore(
    connection: &Connection,
    encounter_id: Uuid,
    before: BeforeCapture,
    now: DateTime<Utc>,
) -> Result<(), RepositoryError> {
    let current: String = connection
        .query_row(
            "SELECT word_id FROM encounters WHERE id=?1",
            [encounter_id.to_string()],
            |r| r.get(0),
        )
        .map_err(repo_error)?;
    connection
        .execute(
            "DELETE FROM translation_history WHERE event_id=?1",
            [encounter_id.to_string()],
        )
        .map_err(repo_error)?;
    if before.words.is_empty() {
        let mut word = get_word_by_id(connection, Uuid::parse_str(&current).map_err(repo_error)?)?
            .ok_or(RepositoryError::NotFound)?;
        word.deleted_at = Some(now);
        word.updated_at = now;
        insert_word(connection, "", &word)?;
        enqueue(connection, "word", word.id, "delete", &word.id, now)?;
    } else {
        // Free the post-reconciliation key before restoring the original identities.
        connection
            .execute(
                "UPDATE words SET dedupe_key='undo:' || id WHERE id=?1",
                [&current],
            )
            .map_err(repo_error)?;
        for word in &before.words {
            insert_word(
                connection,
                &dedupe_key(&word.lemma, &word.source_language),
                word,
            )?;
        }
        for (table, key, refs) in [
            ("encounters", "id", before.encounters),
            ("review_logs", "id", before.reviews),
            ("translation_history", "event_id", before.translations),
        ] {
            for (id, word_id) in refs {
                connection
                    .execute(
                        &format!("UPDATE {table} SET word_id=?1 WHERE {key}=?2"),
                        params![word_id, id],
                    )
                    .map_err(repo_error)?;
                if table == "review_logs" {
                    connection.execute("UPDATE review_logs SET result_payload=json_set(result_payload,'$.wordId',?1) WHERE id=?2 AND result_payload IS NOT NULL",params![word_id,id]).map_err(repo_error)?;
                }
                if table != "translation_history" {
                    connection.execute("UPDATE outbox SET payload=json_set(payload,'$.wordId',?1) WHERE entity_id=?2 AND operation='upsert'",params![word_id,id]).map_err(repo_error)?;
                }
            }
        }
        for word in &before.words {
            enqueue(connection, "word", word.id, "upsert", word, now)?;
        }
    }
    connection
        .execute(
            "DELETE FROM capture_undo WHERE encounter_id=?1",
            [encounter_id.to_string()],
        )
        .map_err(repo_error)?;
    Ok(())
}
