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
    ReviewRating, ReviewResult, SettingsRepository, TodayView, UserSettings, WordDetail,
    WordListItem, WordRepository, apply_review, build_review_queue,
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
    #[error("target language must be explicit")]
    InvalidTargetLanguage,
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
        let mut settings = SettingsRepository::get(self.store.as_ref())?;
        settings.normalize_languages();
        let words = self.store.list()?;
        let due = build_review_queue(words.iter(), now, usize::MAX);
        let total_due_count = due.len();
        let planned = due
            .into_iter()
            .take(settings.daily_limit)
            .collect::<Vec<_>>();
        let review_queue = planned
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
        let planned_review_count = review_queue.len();
        Ok(TodayView {
            total_due_count,
            planned_review_count,
            estimated_minutes: usize::from(planned_review_count > 0)
                .max(planned_review_count.div_ceil(3)),
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
    ) -> Result<ReviewResult, ApplicationError> {
        let mut word = WordRepository::get(self.store.as_ref(), word_id)?
            .ok_or(ApplicationError::WordNotFound)?;
        let prior = word
            .review_state
            .clone()
            .unwrap_or(vocab_domain::ReviewState {
                difficulty: 5.0,
                stability: 1.0,
                due_at: reviewed_at,
                last_reviewed_at: None,
                lapse_count: 0,
            });
        word.review_state = Some(apply_review(Some(&prior), rating, reviewed_at));
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
        let encounter_count = self.store.list_for_word(word_id)?.len();
        let review_history =
            vocab_domain::ReviewRepository::list_for_word(self.store.as_ref(), word_id)?;
        let consecutive_forgotten = review_history
            .iter()
            .rev()
            .take_while(|review| review.rating == ReviewRating::Forgot)
            .count();
        let result = word
            .review_state
            .as_ref()
            .expect("review state was assigned");
        Ok(ReviewResult {
            word_id,
            rating,
            reviewed_at,
            previous_due_at: prior.due_at,
            next_due_at: result.due_at,
            previous_stability: prior.stability,
            stability: result.stability,
            difficulty: result.difficulty,
            lapse_count: result.lapse_count,
            encounter_count,
            repeated_forgetting: consecutive_forgotten >= 3,
        })
    }

    pub fn get_settings(&self) -> Result<UserSettings, ApplicationError> {
        let mut settings = SettingsRepository::get(self.store.as_ref())?;
        settings.normalize_languages();
        Ok(settings)
    }

    pub fn update_settings(&self, mut settings: UserSettings) -> Result<(), ApplicationError> {
        settings.normalize_languages();
        if !settings.languages_are_valid() {
            return Err(ApplicationError::InvalidTargetLanguage);
        }
        SettingsRepository::save(self.store.as_ref(), &settings)?;
        Ok(())
    }
}
