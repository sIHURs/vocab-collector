use super::*;

fn spreadsheet_text(value: &str) -> String {
    if value.trim_start().starts_with(['=', '+', '-', '@']) || value.starts_with(['\t', '\r', '\n'])
    {
        format!("'{value}")
    } else {
        value.into()
    }
}

impl SqliteStore {
    /// Human-readable export; intentionally not a reversible backup format.
    pub fn vocabulary_csv(&self) -> Result<(Vec<u8>, usize), RepositoryError> {
        let rows = self.vocabulary_listing()?;
        let mut writer = csv::WriterBuilder::new()
            .terminator(csv::Terminator::CRLF)
            .from_writer(vec![0xef, 0xbb, 0xbf]);
        writer
            .write_record([
                "vocabulary",
                "source_language",
                "translation",
                "translation_language",
                "learning_status",
                "encounter_count",
                "last_encounter_at",
            ])
            .map_err(repo_error)?;
        for (word, language) in &rows {
            let status = match word.status {
                WordStatus::Learning => "learning",
                WordStatus::Mastered => "mastered",
                WordStatus::Paused => "paused",
            };
            writer
                .write_record([
                    spreadsheet_text(&word.display_form),
                    spreadsheet_text(language),
                    spreadsheet_text(word.translation.as_deref().unwrap_or("")),
                    spreadsheet_text(word.translation_language.as_deref().unwrap_or("")),
                    status.into(),
                    word.encounter_count.to_string(),
                    if word.encounter_count == 0 {
                        String::new()
                    } else {
                        word.last_seen_at.to_rfc3339()
                    },
                ])
                .map_err(repo_error)?;
        }
        Ok((writer.into_inner().map_err(repo_error)?, rows.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vocab_domain::{OwnerScope, Word};
    #[test]
    fn csv_handles_starters_unicode_and_formula_text() {
        let store = SqliteStore::open_in_memory().unwrap();
        let now = Utc::now();
        let mut word = Word {
            id: Uuid::now_v7(),
            owner_scope: OwnerScope::Guest,
            lemma: "test".into(),
            display_form: "=秘密,\"x\"\nnext".into(),
            source_language: "en".into(),
            target_language: "zh".into(),
            translation: None,
            part_of_speech: None,
            status: WordStatus::Learning,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            mastered_at: None,
            achieved_at: None,
            delete_after: None,
            review_state: None,
        };
        WordRepository::save(&store, &word).unwrap();
        let (bytes, count) = store.vocabulary_csv().unwrap();
        assert_eq!(count, 1);
        let mut reader = csv::Reader::from_reader(&bytes[3..]);
        let row = reader.records().next().unwrap().unwrap();
        assert_eq!(&row[0], "'=秘密,\"x\"\nnext");
        assert_eq!(&row[5], "0");
        assert_eq!(&row[6], "");
        word.achieved_at = Some(now);
        WordRepository::save(&store, &word).unwrap();
        assert_eq!(store.vocabulary_csv().unwrap().1, 0);
    }
}
