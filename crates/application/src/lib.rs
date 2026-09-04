#![forbid(unsafe_code)]

//! Application use cases exposed to desktop IPC.

pub mod debug;
mod platform_capture;

use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vocab_domain::{
    CaptureCard, CaptureOrigin, EncounterRepository, RepositoryError, ReviewCard, ReviewLog,
    ReviewRating, ReviewResult, ReviewSessionInsight, SettingsRepository, TodayView, UserSettings,
    WordDetail, WordListItem, WordRepository, WordStatus, apply_review, build_review_queue,
    summarize_review_session,
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
    #[error("review submission identity does not match the original request")]
    ReviewSubmissionConflict,
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
        self.submit_review_once(Uuid::now_v7(), word_id, rating, reviewed_at)
    }

    pub fn submit_review_once(
        &self,
        submission_id: Uuid,
        word_id: Uuid,
        rating: ReviewRating,
        reviewed_at: DateTime<Utc>,
    ) -> Result<ReviewResult, ApplicationError> {
        let review_history =
            vocab_domain::ReviewRepository::list_for_word(self.store.as_ref(), word_id)?;
        if let Some(existing) = review_history
            .iter()
            .find(|review| review.id == submission_id)
        {
            if existing.word_id != word_id || existing.rating != rating {
                return Err(ApplicationError::ReviewSubmissionConflict);
            }
            return existing
                .result
                .clone()
                .ok_or(ApplicationError::ReviewSubmissionConflict);
        }
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
        let encounter_count = self.store.list_for_word(word_id)?.len();
        let consecutive_forgotten = if rating == ReviewRating::Forgot {
            review_history
                .iter()
                .rev()
                .take_while(|review| review.rating == ReviewRating::Forgot)
                .count()
                + 1
        } else {
            0
        };
        let state = word
            .review_state
            .as_ref()
            .expect("review state was assigned");
        let result = ReviewResult {
            submission_id,
            word_id,
            rating,
            reviewed_at,
            previous_due_at: prior.due_at,
            next_due_at: state.due_at,
            previous_stability: prior.stability,
            stability: state.stability,
            difficulty: state.difficulty,
            lapse_count: state.lapse_count,
            encounter_count,
            repeated_forgetting: consecutive_forgotten >= 3,
        };
        let review = ReviewLog {
            id: submission_id,
            word_id,
            rating,
            reviewed_at,
            received_at: reviewed_at,
            device_id: self.device_id,
            result: Some(result.clone()),
        };
        self.store.record_review(&word, &review)?;
        Ok(result)
    }

    pub fn get_settings(&self) -> Result<UserSettings, ApplicationError> {
        let mut settings = SettingsRepository::get(self.store.as_ref())?;
        settings.normalize_languages();
        Ok(settings)
    }

    pub fn get_review_session_insight(
        &self,
        submission_ids: &[Uuid],
        next_day_end: DateTime<Utc>,
    ) -> Result<ReviewSessionInsight, ApplicationError> {
        let words = self.store.list()?;
        let requested: HashSet<_> = submission_ids.iter().copied().collect();
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        for word in &words {
            for review in
                vocab_domain::ReviewRepository::list_for_word(self.store.as_ref(), word.id)?
            {
                if requested.contains(&review.id)
                    && seen.insert(review.id)
                    && let Some(result) = review.result
                {
                    results.push(result);
                }
            }
        }
        let next_day_due_count = words
            .iter()
            .filter(|word| {
                word.deleted_at.is_none()
                    && word.status == WordStatus::Learning
                    && word
                        .review_state
                        .as_ref()
                        .is_some_and(|state| state.due_at < next_day_end)
            })
            .count();
        Ok(summarize_review_session(&results, next_day_due_count))
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
