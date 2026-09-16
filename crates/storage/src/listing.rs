use super::*;
use vocab_domain::WordListItem;

/// A single snapshot for list display and CSV, including items without an Encounter.
impl SqliteStore {
    pub fn vocabulary_listing(&self) -> Result<Vec<(WordListItem, String)>, RepositoryError> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(repo_error)?;
        let settings: Option<String> = tx
            .query_row(
                "SELECT payload FROM user_settings WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(repo_error)?;
        let settings: UserSettings = settings
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(repo_error)?
            .unwrap_or_default();
        let mut stmt = tx.prepare("WITH encounter_counts AS (
            SELECT word_id,COUNT(*) AS n,MAX(captured_at) AS last_at FROM encounters WHERE deleted_at IS NULL GROUP BY word_id
        ) SELECT w.id,w.owner_scope,w.lemma,w.display_form,w.source_language,w.target_language,
            w.translation,w.part_of_speech,w.status,w.created_at,w.updated_at,w.deleted_at,
            w.mastered_at,w.achieved_at,w.delete_after,w.review_state_json,
            COALESCE(e.n,0),e.last_at,t.text,t.target_language
        FROM words w LEFT JOIN encounter_counts e ON e.word_id=w.id
        LEFT JOIN translation_history t ON t.rowid=COALESCE(
            (SELECT MAX(p.rowid) FROM translation_history p WHERE p.word_id=w.id AND p.target_language=?1),
            (SELECT MAX(p.rowid) FROM translation_history p WHERE p.word_id=w.id))
        WHERE w.deleted_at IS NULL AND w.achieved_at IS NULL").map_err(repo_error)?;
        let mut items = stmt
            .query_map([settings.target_language.to_lowercase()], |row| {
                let word = map_word(row)?;
                let last: Option<String> = row.get(17)?;
                Ok((
                    WordListItem {
                        id: word.id,
                        display_form: word.display_form,
                        translation: row.get(18)?,
                        translation_language: row.get(19)?,
                        status: word.status,
                        encounter_count: usize::try_from(row.get::<_, i64>(16)?)
                            .map_err(sql_conversion_error)?,
                        next_review_at: word.review_state.map(|s| s.due_at),
                        last_seen_at: last.map(parse_time).transpose()?.unwrap_or(word.updated_at),
                        achieved_at: word.achieved_at,
                        delete_after: word.delete_after,
                    },
                    word.source_language,
                ))
            })
            .map_err(repo_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(repo_error)?;
        items.sort_by(|a, b| {
            b.0.last_seen_at
                .cmp(&a.0.last_seen_at)
                .then_with(|| a.0.id.cmp(&b.0.id))
        });
        Ok(items)
    }
}
