use std::{path::Path, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use vocab_domain::{
    CaptureCard, CaptureOrigin, ReviewLog, ReviewRating, ReviewRepository, TodayView, UserSettings,
    WordDetail, WordListItem, dedupe_key, normalize_context, normalize_lemma,
};
use vocab_storage::{DatabaseSummary, OutboxDebugEntry, SqliteStore};

use crate::{AppService, ApplicationError, CaptureRequest};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugCaptureInput {
    pub selected_text: String,
    pub lemma: Option<String>,
    pub sentence: String,
    pub source_language: String,
    pub target_language: String,
    pub translation: Option<String>,
    pub part_of_speech: Option<String>,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub source_url: Option<String>,
    #[serde(default)]
    pub capture_origin: CaptureOrigin,
    pub captured_at: DateTime<Utc>,
}

impl From<DebugCaptureInput> for CaptureRequest {
    fn from(value: DebugCaptureInput) -> Self {
        Self {
            selected_text: value.selected_text,
            lemma: value.lemma,
            sentence: value.sentence,
            source_language: value.source_language,
            target_language: value.target_language,
            translation: value.translation,
            part_of_speech: value.part_of_speech,
            source_app: value.source_app,
            source_title: value.source_title,
            source_url: value.source_url,
            capture_origin: value.capture_origin,
            captured_at: value.captured_at,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugStage {
    Input,
    Domain,
    Application,
    Persistence,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugObservation {
    pub stage: DebugStage,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WordDebugView {
    pub detail: WordDetail,
    pub reviews: Vec<ReviewLog>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTrace {
    pub input: DebugCaptureInput,
    pub observations: Vec<DebugObservation>,
    pub result: CaptureCard,
    pub persisted: WordDebugView,
}

pub struct CoreDebugSession {
    application: AppService,
    store: Arc<SqliteStore>,
}

impl CoreDebugSession {
    pub fn in_memory(device_id: Uuid) -> Result<Self, ApplicationError> {
        let store = Arc::new(SqliteStore::open_in_memory()?);
        Ok(Self::from_store(store, device_id))
    }

    pub fn open(path: impl AsRef<Path>, device_id: Uuid) -> Result<Self, ApplicationError> {
        let store = Arc::new(SqliteStore::open(path)?);
        Ok(Self::from_store(store, device_id))
    }

    fn from_store(store: Arc<SqliteStore>, device_id: Uuid) -> Self {
        let application = AppService::new(Arc::clone(&store), device_id);
        Self { application, store }
    }

    pub fn capture(&self, input: DebugCaptureInput) -> Result<CaptureTrace, ApplicationError> {
        let lemma = input
            .lemma
            .as_deref()
            .unwrap_or(input.selected_text.as_str());
        let normalized_lemma = normalize_lemma(lemma);
        let normalized_sentence = normalize_context(&input.sentence);
        let dedupe_key = dedupe_key(lemma, &input.source_language, &input.target_language);
        let outbox_before = self.store.pending_outbox_count()?;
        let result = self.application.capture(input.clone().into())?;
        let persisted = self.inspect_word(result.word_id)?;
        let outbox_after = self.store.pending_outbox_count()?;
        let observations = vec![
            DebugObservation {
                stage: DebugStage::Input,
                data: json!({ "capture": input }),
            },
            DebugObservation {
                stage: DebugStage::Domain,
                data: json!({
                    "normalizedLemma": normalized_lemma,
                    "normalizedSentence": normalized_sentence,
                    "dedupeKey": dedupe_key,
                }),
            },
            DebugObservation {
                stage: DebugStage::Application,
                data: json!({ "captureCard": result }),
            },
            DebugObservation {
                stage: DebugStage::Persistence,
                data: json!({
                    "outboxBefore": outbox_before,
                    "outboxAfter": outbox_after,
                    "word": persisted,
                }),
            },
        ];

        Ok(CaptureTrace {
            input,
            observations,
            result,
            persisted,
        })
    }

    pub fn inspect_word(&self, word_id: Uuid) -> Result<WordDebugView, ApplicationError> {
        Ok(WordDebugView {
            detail: self.application.get_word(word_id)?,
            reviews: ReviewRepository::list_for_word(self.store.as_ref(), word_id)?,
        })
    }

    pub fn list_words(&self) -> Result<Vec<WordListItem>, ApplicationError> {
        self.application.list_words()
    }

    pub fn today(&self, now: DateTime<Utc>) -> Result<TodayView, ApplicationError> {
        self.application.get_today(now)
    }

    pub fn review(
        &self,
        word_id: Uuid,
        rating: ReviewRating,
        at: DateTime<Utc>,
    ) -> Result<WordDebugView, ApplicationError> {
        self.application.submit_review(word_id, rating, at)?;
        self.inspect_word(word_id)
    }

    pub fn undo(&self, encounter_id: Uuid) -> Result<DatabaseSummary, ApplicationError> {
        self.application.undo_capture(encounter_id)?;
        self.database_summary()
    }

    pub fn settings(&self) -> Result<UserSettings, ApplicationError> {
        self.application.get_settings()
    }

    pub fn update_settings(
        &self,
        settings: UserSettings,
    ) -> Result<UserSettings, ApplicationError> {
        self.application.update_settings(settings)?;
        self.settings()
    }

    pub fn database_summary(&self) -> Result<DatabaseSummary, ApplicationError> {
        Ok(self.store.database_summary()?)
    }

    pub fn outbox(&self) -> Result<Vec<OutboxDebugEntry>, ApplicationError> {
        Ok(self.store.outbox_debug_entries()?)
    }
}
