#![forbid(unsafe_code)]

//! Application use cases exposed to desktop IPC.

pub mod debug;
mod platform_capture;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vocab_domain::{
    CaptureCard, CaptureOrigin, EncounterRepository, RepositoryError, ReviewCard, ReviewLog,
    ReviewRating, SettingsRepository, TodayView, UserSettings, WordDetail, WordListItem,
    WordRepository, apply_review, build_review_queue,
};
use vocab_storage::{CaptureRecord, SqliteStore};

pub use platform_capture::{PlatformCaptureError, PlatformCaptureWorkflow, PreparedCapture};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRequest {
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

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("{0}")]
    Repository(#[from] RepositoryError),
    #[error("word not found")]
    WordNotFound,
}

pub struct AppService {
    store: Arc<SqliteStore>,
    device_id: Uuid,
}

impl AppService {
    pub fn new(store: Arc<SqliteStore>, device_id: Uuid) -> Self {
        Self { store, device_id }
    }

    pub fn capture(&self, request: CaptureRequest) -> Result<CaptureCard, ApplicationError> {
        let stored = self.store.capture(&CaptureRecord {
            selected_text: request.selected_text.clone(),
            lemma: request.lemma.unwrap_or(request.selected_text),
            sentence: request.sentence,
            source_language: request.source_language,
            target_language: request.target_language,
            translation: request.translation,
            part_of_speech: request.part_of_speech,
            source_app: request.source_app,
            source_title: request.source_title,
            source_url: request.source_url,
            capture_origin: request.capture_origin,
            captured_at: request.captured_at,
        })?;
        let encounter_count = self.store.list_for_word(stored.word.id)?.len();
        Ok(CaptureCard {
            word_id: stored.word.id,
            encounter_id: stored.encounter.id,
            display_form: stored.word.display_form,
            translation: stored.word.translation,
            context: stored.encounter.sentence,
            encounter_count,
            is_existing_word: stored.is_existing_word,
        })
    }

    pub fn undo_capture(&self, encounter_id: Uuid) -> Result<(), ApplicationError> {
        self.store.soft_delete(encounter_id, Utc::now())?;
        Ok(())
    }

    pub fn get_today(&self, now: DateTime<Utc>) -> Result<TodayView, ApplicationError> {
        let settings = SettingsRepository::get(self.store.as_ref())?;
        let words = self.store.list()?;
        let queue = build_review_queue(words.iter(), now, settings.daily_limit);
        let review_queue = queue
            .iter()
            .map(|word| {
                let context = self
                    .store
                    .list_for_word(word.id)?
                    .into_iter()
                    .next()
                    .map(|encounter| encounter.sentence);
                Ok(ReviewCard {
                    word_id: word.id,
                    display_form: word.display_form.clone(),
                    translation: word.translation.clone(),
                    context,
                })
            })
            .collect::<Result<Vec<_>, RepositoryError>>()?;
        let recent_captures = self
            .list_words()?
            .into_iter()
            .take(settings.recent_captures_limit)
            .collect();
        let due_count = review_queue.len();
        Ok(TodayView {
            due_count,
            estimated_minutes: usize::from(due_count > 0).max(due_count.div_ceil(3)),
            review_queue,
            recent_captures,
            settings,
        })
    }

    pub fn list_words(&self) -> Result<Vec<WordListItem>, ApplicationError> {
        self.store
            .list()?
            .into_iter()
            .map(|word| {
                let encounters = self.store.list_for_word(word.id)?;
                let last_seen_at = encounters
                    .first()
                    .map_or(word.updated_at, |encounter| encounter.captured_at);
                Ok(WordListItem {
                    id: word.id,
                    display_form: word.display_form,
                    translation: word.translation,
                    status: word.status,
                    encounter_count: encounters.len(),
                    next_review_at: word.review_state.map(|state| state.due_at),
                    last_seen_at,
                })
            })
            .collect::<Result<Vec<_>, RepositoryError>>()
            .map_err(Into::into)
    }

    pub fn get_word(&self, word_id: Uuid) -> Result<WordDetail, ApplicationError> {
        let word = WordRepository::get(self.store.as_ref(), word_id)?
            .ok_or(ApplicationError::WordNotFound)?;
        let encounters = self.store.list_for_word(word_id)?;
        let last_seen_at = encounters
            .first()
            .map_or(word.updated_at, |encounter| encounter.captured_at);
        Ok(WordDetail {
            item: WordListItem {
                id: word.id,
                display_form: word.display_form,
                translation: word.translation,
                status: word.status,
                encounter_count: encounters.len(),
                next_review_at: word.review_state.as_ref().map(|state| state.due_at),
                last_seen_at,
            },
            lemma: word.lemma,
            part_of_speech: word.part_of_speech,
            encounters,
        })
    }

    pub fn submit_review(
        &self,
        word_id: Uuid,
        rating: ReviewRating,
        reviewed_at: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        let mut word = WordRepository::get(self.store.as_ref(), word_id)?
            .ok_or(ApplicationError::WordNotFound)?;
        word.review_state = Some(apply_review(
            word.review_state.as_ref(),
            rating,
            reviewed_at,
        ));
        word.updated_at = reviewed_at;
        let review = ReviewLog {
            id: Uuid::now_v7(),
            word_id,
            rating,
            reviewed_at,
            received_at: reviewed_at,
            device_id: self.device_id,
        };
        self.store.record_review(&word, &review)?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<UserSettings, ApplicationError> {
        Ok(SettingsRepository::get(self.store.as_ref())?)
    }

    pub fn update_settings(&self, settings: UserSettings) -> Result<(), ApplicationError> {
        SettingsRepository::save(self.store.as_ref(), &settings)?;
        Ok(())
    }
}
