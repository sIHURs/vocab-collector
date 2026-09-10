use super::*;
use std::collections::BTreeMap;

pub(super) fn candidates(
    connection: &Connection,
    input: &CaptureRecord,
) -> Result<Vec<Word>, RepositoryError> {
    let lemma = normalize_lemma(&input.lemma);
    let source = input.source_language.trim().to_lowercase();
    let words: Vec<_> = all(connection)?
        .into_iter()
        .filter(|w| {
            w.deleted_at.is_none() && w.owner_scope == OwnerScope::Guest && w.lemma == lemma
        })
        .collect();
    let known: std::collections::HashSet<_> = words
        .iter()
        .filter(|w| w.source_language != "auto")
        .map(|w| w.source_language.as_str())
        .collect();
    let resolved = if source == "auto" && known.len() == 1 {
        *known.iter().next().unwrap()
    } else {
        &source
    };
    let allow_unknown = known.is_empty() || (known.len() == 1 && known.contains(resolved));
    Ok(words
        .iter()
        .filter(|w| w.source_language == resolved || (allow_unknown && w.source_language == "auto"))
        .cloned()
        .collect())
}

pub(super) fn resolve(
    connection: &Connection,
    input: &CaptureRecord,
) -> Result<Option<Word>, RepositoryError> {
    let words = candidates(connection, input)?;
    if words.is_empty() {
        return Ok(None);
    }
    let resolved = words
        .iter()
        .find(|w| w.source_language != "auto")
        .map(|w| w.source_language.clone())
        .unwrap_or_else(|| input.source_language.trim().to_lowercase());
    let mut word = merge(connection, words)?;
    word.source_language = resolved;
    insert_word(
        connection,
        &dedupe_key(&word.lemma, &word.source_language),
        &word,
    )?;
    Ok(Some(word))
}

impl SqliteStore {
    pub fn capture_match(&self, input: &CaptureRecord) -> Result<Option<Word>, RepositoryError> {
        let words = candidates(&*self.lock()?, input)?;
        // The notice describes matched history, not the state chosen for merging.
        // A Learning auto alias must not conceal a known-language Achieved item.
        Ok(words
            .iter()
            .find(|word| word.is_achieved())
            .or(words.first())
            .cloned())
    }
}

pub(super) fn scoped_key(word: &Word, key: &str) -> Result<String, RepositoryError> {
    Ok(if word.deleted_at.is_some() {
        format!("deleted:{}", word.id)
    } else {
        format!("{}|{key}", enum_json(&word.owner_scope)?)
    })
}

pub(super) fn all(connection: &Connection) -> Result<Vec<Word>, RepositoryError> {
    connection
        .prepare(
            "SELECT id, owner_scope, lemma, display_form, source_language,
        target_language, translation, part_of_speech, status, created_at, updated_at, deleted_at,
        mastered_at, achieved_at, delete_after, review_state_json FROM words ORDER BY id",
        )
        .map_err(repo_error)?
        .query_map([], map_word)
        .map_err(repo_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(repo_error)
}

pub(super) fn migrate(connection: &Connection) -> Result<(), RepositoryError> {
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='identity_migration')",
            [],
            |r| r.get(0),
        )
        .map_err(repo_error)?;
    if !exists {
        // The caller owns the upgrade savepoint. Temporary keys prevent uniqueness collisions.
        connection
            .execute("UPDATE words SET dedupe_key='migrating:' || id", [])
            .map_err(repo_error)?;
        let mut groups: BTreeMap<String, Vec<Word>> = BTreeMap::new();
        for mut word in all(connection)? {
            // These are explicitly named old QA fixtures, not a general display-form fallback.
            if word.lemma.starts_with("qa-fixture-") {
                let tail = word.lemma.strip_prefix("qa-fixture-").unwrap();
                if tail.split_once('-').is_some_and(|(n, text)| {
                    n.parse::<u32>().is_ok() && text == normalize_lemma(&word.display_form)
                }) {
                    word.lemma = normalize_lemma(&word.display_form);
                }
            }
            word.lemma = normalize_lemma(&word.lemma);
            word.source_language = word.source_language.trim().to_lowercase();
            let key = scoped_key(&word, &dedupe_key(&word.lemma, &word.source_language))?;
            groups.entry(key).or_default().push(word);
        }
        for words in groups.into_values() {
            let word = merge(connection, words)?;
            insert_word(
                connection,
                &dedupe_key(&word.lemma, &word.source_language),
                &word,
            )?;
            enqueue(
                connection,
                "word",
                word.id,
                "upsert",
                &word,
                word.updated_at,
            )?;
        }
        connection.execute_batch("CREATE TABLE identity_migration (version INTEGER NOT NULL); INSERT INTO identity_migration VALUES(8);").map_err(repo_error)?;
    }
    connection
        .execute_batch("PRAGMA user_version = 8;")
        .map_err(repo_error)
}

pub(super) fn merge(tx: &Connection, mut words: Vec<Word>) -> Result<Word, RepositoryError> {
    words.sort_by_key(|w| w.id);
    let mut survivor = words[0].clone();
    apply_state(&mut survivor, &words);
    for removed in words.iter().skip(1) {
        for (table, key) in [
            ("encounters", "id"),
            ("review_logs", "id"),
            ("translation_history", "event_id"),
        ] {
            for mut reference in references::collect(tx, table, key, removed.id)? {
                reference.word_id = survivor.id.to_string();
                references::remap(tx, table, key, &reference)?;
            }
        }
        tx.execute(
            "DELETE FROM outbox WHERE entity_type='word' AND entity_id=?1",
            [removed.id.to_string()],
        )
        .map_err(repo_error)?;
        tx.execute("DELETE FROM words WHERE id=?1", [removed.id.to_string()])
            .map_err(repo_error)?;
    }
    if let Some(value) = translations::list(tx, survivor.id)?.first() {
        survivor.target_language.clone_from(&value.target_language);
        survivor.translation = Some(value.text.clone());
    }
    Ok(survivor)
}

fn apply_state(survivor: &mut Word, words: &[Word]) {
    let state = words
        .iter()
        .min_by_key(|w| {
            (
                if w.is_achieved() {
                    3
                } else {
                    match w.status {
                        WordStatus::Learning => 0,
                        WordStatus::Paused => 1,
                        WordStatus::Mastered => 2,
                    }
                },
                w.review_state.as_ref().map(|s| s.due_at),
                w.id,
            )
        })
        .unwrap();
    survivor.status = state.status;
    survivor.review_state = state.review_state.clone();
    survivor.mastered_at = state.mastered_at;
    survivor.achieved_at = state.achieved_at;
    survivor.delete_after = state.delete_after;
    if words.iter().all(Word::is_achieved) {
        let latest = words.iter().max_by_key(|w| w.delete_after).unwrap();
        survivor.achieved_at = latest.achieved_at;
        survivor.delete_after = latest.delete_after;
    }
}
