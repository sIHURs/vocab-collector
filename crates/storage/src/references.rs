use super::*;
use serde::Deserialize;

// Tuple serialization preserves the persisted Undo snapshot format.
#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "(String, String)", into = "(String, String)")]
pub(super) struct WordReference {
    pub entity_id: String,
    pub word_id: String,
}
impl From<(String, String)> for WordReference {
    fn from((entity_id, word_id): (String, String)) -> Self {
        Self { entity_id, word_id }
    }
}
impl From<WordReference> for (String, String) {
    fn from(value: WordReference) -> Self {
        (value.entity_id, value.word_id)
    }
}

pub(super) fn collect(
    connection: &Connection,
    table: &str,
    key: &str,
    word_id: Uuid,
) -> Result<Vec<WordReference>, RepositoryError> {
    connection
        .prepare(&format!(
            "SELECT {key},word_id FROM {table} WHERE word_id=?1"
        ))
        .map_err(repo_error)?
        .query_map([word_id.to_string()], |row| {
            Ok(WordReference {
                entity_id: row.get(0)?,
                word_id: row.get(1)?,
            })
        })
        .map_err(repo_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(repo_error)
}

pub(super) fn remap(
    connection: &Connection,
    table: &str,
    key: &str,
    reference: &WordReference,
) -> Result<(), RepositoryError> {
    let WordReference { entity_id, word_id } = reference;
    connection
        .execute(
            &format!("UPDATE {table} SET word_id=?1 WHERE {key}=?2"),
            params![word_id, entity_id],
        )
        .map_err(repo_error)?;
    if table == "review_logs" {
        connection.execute("UPDATE review_logs SET result_payload=json_set(result_payload,'$.wordId',?1) WHERE id=?2 AND result_payload IS NOT NULL", params![word_id, entity_id]).map_err(repo_error)?;
    }
    let entity_type = match table {
        "encounters" => Some("encounter"),
        "review_logs" => Some("review_log"),
        _ => None,
    };
    if let Some(entity_type) = entity_type {
        connection.execute("UPDATE outbox SET payload=json_set(payload,'$.wordId',?1) WHERE entity_id=?2 AND entity_type=?3 AND operation='upsert'", params![word_id, entity_id, entity_type]).map_err(repo_error)?;
    }
    Ok(())
}
