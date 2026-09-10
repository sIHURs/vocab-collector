#![forbid(unsafe_code)]

//! Application use cases exposed to desktop IPC.

pub mod debug;
mod platform_capture;

use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vocab_domain::{
    AchievedCaptureConflict, AchievedWordListItem, CaptureCard, CaptureOrigin, DeletionUrgency,
    EncounterRepository, GlobalInsight, LifecycleError, LifecycleSweepResult, RepositoryError,
    ReviewCard, ReviewLog, ReviewRating, ReviewResult, ReviewSessionInsight, SettingsRepository,
    TodayView, UserSettings, WordDetail, WordListItem, WordRepository, WordStatus, apply_review,
    build_review_queue, summarize_review_session,
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
    #[error("{0}")]
    Lifecycle(#[from] LifecycleError),
    #[error("retention must be 10, 20, 30, or 60 days")]
    InvalidAchievedRetention,
    #[error("batch must contain between 1 and 500 unique Vocabulary Items")]
    InvalidLifecycleBatch,
}

pub struct AppService {
    store: Arc<SqliteStore>,
    device_id: Uuid,
}

impl AppService {
    pub fn change_learning_status(
        &self,
        word_id: Uuid,
        status: WordStatus,
        now: DateTime<Utc>,
    ) -> Result<Option<Uuid>, ApplicationError> {
        Ok(self.store.change_learning_status(word_id, status, now)?)
    }

    pub fn undo_learning_status(
        &self,
        token: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Uuid, ApplicationError> {
        Ok(self.store.undo_learning_status(token, now)?)
    }

    pub fn new(store: Arc<SqliteStore>, device_id: Uuid) -> Self {
        Self { store, device_id }
    }

    pub fn capture(&self, request: CaptureRequest) -> Result<CaptureCard, ApplicationError> {
        let stored = self.store.capture(&capture_record(request))?;
        self.capture_card(stored)
    }

    pub fn find_achieved_capture(
        &self,
        request: &CaptureRequest,
    ) -> Result<Option<AchievedCaptureConflict>, ApplicationError> {
        let word = self
            .store
            .capture_match(&capture_record(request.clone()))?
            .filter(|word| word.is_achieved());
        Ok(word.map(|word| AchievedCaptureConflict {
            word_id: word.id,
            display_form: word.display_form,
            achieved_at: word.achieved_at.expect("Achieved invariant checked"),
            delete_after: word.delete_after.expect("Achieved invariant checked"),
        }))
    }

    pub fn restore_achieved_and_capture(
        &self,
        word_id: Uuid,
        request: CaptureRequest,
    ) -> Result<CaptureCard, ApplicationError> {
        let stored = self
            .store
            .restore_achieved_and_capture(word_id, &capture_record(request))?;
        self.capture_card(stored)
    }

    fn capture_card(
        &self,
        stored: vocab_storage::StoredCapture,
    ) -> Result<CaptureCard, ApplicationError> {
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

    pub fn get_vocabulary_log(&self) -> Result<vocab_domain::VocabularyLog, ApplicationError> {
        use vocab_domain::VocabularyLogRepository;
        Ok(self.store.vocabulary_log()?)
    }

    pub fn get_global_insight(&self) -> Result<GlobalInsight, ApplicationError> {
        let statistics = self.store.lifetime_statistics()?;
        Ok(GlobalInsight {
            current_vocabulary_count: statistics.current_vocabulary_count,
            current_achieved_count: statistics.current_achieved_count,
            lifetime_vocabulary_count: statistics.current_vocabulary_count
                + statistics.archived_vocabulary_count,
            lifetime_encounter_count: statistics.active_encounter_count
                + statistics.archived_encounter_count,
            lifetime_review_count: statistics.review_count + statistics.archived_review_count,
            lifetime_remembered_count: statistics.remembered_count
                + statistics.archived_remembered_count,
            lifetime_forgotten_count: statistics.forgotten_count
                + statistics.archived_forgotten_count,
            lifetime_rating_breakdown_complete: statistics.rating_breakdown_complete,
        })
    }

    pub fn undo_capture(&self, encounter_id: Uuid) -> Result<(), ApplicationError> {
        self.store.soft_delete(encounter_id, Utc::now())?;
        Ok(())
    }

    pub fn get_today(&self, now: DateTime<Utc>) -> Result<TodayView, ApplicationError> {
        let mut settings = SettingsRepository::get(self.store.as_ref())?;
        settings.normalize_languages();
        let words = self.words_with_captures()?;
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
                let (translation, translation_language) = self.displayed_translation(word.id)?;
                Ok(ReviewCard {
                    translation_language,
                    word_id: word.id,
                    display_form: word.display_form.clone(),
                    translation,
                    context,
                })
            })
            .collect::<Result<Vec<_>, RepositoryError>>()?;
        let mut recent_captures = self.list_words()?;
        recent_captures.sort_by_key(|word| std::cmp::Reverse(word.last_seen_at));
        recent_captures.truncate(settings.recent_captures_limit);
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

    fn words_with_captures(&self) -> Result<Vec<vocab_domain::Word>, ApplicationError> {
        let mut words = Vec::new();
        for word in self.store.list()? {
            if !self.store.list_for_word(word.id)?.is_empty() {
                words.push(word);
            }
        }
        Ok(words)
    }

    fn displayed_translation(
        &self,
        word_id: Uuid,
    ) -> Result<(Option<String>, Option<String>), RepositoryError> {
        let preferred = SettingsRepository::get(self.store.as_ref())?
            .target_language
            .to_lowercase();
        let values = self.store.translations(word_id)?;
        let value = values
            .iter()
            .find(|v| v.target_language == preferred)
            .or_else(|| values.first());
        Ok((
            value.map(|v| v.text.clone()),
            value.map(|v| v.target_language.clone()),
        ))
    }

    pub fn list_words(&self) -> Result<Vec<WordListItem>, ApplicationError> {
        self.words_with_captures()?
            .into_iter()
            .filter(|word| !word.is_achieved())
            .map(|word| {
                let encounters = self.store.list_for_word(word.id)?;
                let last_seen_at = encounters
                    .first()
                    .map_or(word.updated_at, |encounter| encounter.captured_at);
                let (translation, translation_language) = self.displayed_translation(word.id)?;
                Ok(WordListItem {
                    translation_language,
                    id: word.id,
                    display_form: word.display_form,
                    translation,
                    status: word.status,
                    encounter_count: encounters.len(),
                    next_review_at: word.review_state.map(|state| state.due_at),
                    last_seen_at,
                    achieved_at: word.achieved_at,
                    delete_after: word.delete_after,
                })
            })
            .collect::<Result<Vec<_>, RepositoryError>>()
            .map_err(Into::into)
    }

    pub fn achieve_word(
        &self,
        word_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<AchievedWordListItem, ApplicationError> {
        let settings = self.get_settings()?;
        let mut word = WordRepository::get(self.store.as_ref(), word_id)?
            .ok_or(ApplicationError::WordNotFound)?;
        word.achieve(now, settings.achieved_retention_days)?;
        WordRepository::save(self.store.as_ref(), &word)?;
        self.achieved_list_item(word, now)
    }

    pub fn list_achieved_words(&self) -> Result<Vec<AchievedWordListItem>, ApplicationError> {
        self.list_achieved_words_at(Utc::now())
    }

    pub fn list_achieved_words_at(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<AchievedWordListItem>, ApplicationError> {
        let mut items = self
            .words_with_captures()?
            .into_iter()
            .filter(|word| word.is_achieved())
            .map(|word| self.achieved_list_item(word, now))
            .collect::<Result<Vec<_>, _>>()?;
        items.sort_by(|a, b| {
            a.delete_after
                .cmp(&b.delete_after)
                .then_with(|| a.display_form.cmp(&b.display_form))
        });
        Ok(items)
    }

    pub fn unachieve_words(
        &self,
        ids: &[Uuid],
        now: DateTime<Utc>,
    ) -> Result<usize, ApplicationError> {
        let ids = validated_batch(ids)?;
        Ok(self.store.unachieve_words(&ids, now)?)
    }

    pub fn delete_achieved_words(
        &self,
        ids: &[Uuid],
        now: DateTime<Utc>,
    ) -> Result<usize, ApplicationError> {
        let ids = validated_batch(ids)?;
        Ok(self.store.delete_achieved_words(&ids, now)?)
    }

    pub fn run_lifecycle_sweep(
        &self,
        now: DateTime<Utc>,
    ) -> Result<LifecycleSweepResult, ApplicationError> {
        let settings = self.get_settings()?;
        let mut achieved_count = 0;
        if settings.automatic_achieve_enabled {
            for mut word in self
                .store
                .list()?
                .into_iter()
                .filter(|word| word.is_automatic_achieve_due(now))
            {
                word.achieve(now, settings.achieved_retention_days)?;
                WordRepository::save(self.store.as_ref(), &word)?;
                achieved_count += 1;
            }
        }
        let expired = self
            .store
            .list()?
            .into_iter()
            .filter(|word| {
                word.is_achieved() && word.delete_after.is_some_and(|deadline| deadline <= now)
            })
            .map(|word| word.id)
            .collect::<Vec<_>>();
        let purged_count = if expired.is_empty() {
            0
        } else {
            self.store.delete_achieved_words(&expired, now)?
        };
        Ok(LifecycleSweepResult {
            achieved_count,
            purged_count,
        })
    }

    fn achieved_list_item(
        &self,
        word: vocab_domain::Word,
        now: DateTime<Utc>,
    ) -> Result<AchievedWordListItem, ApplicationError> {
        let delete_after = word.delete_after.expect("Achieved invariant checked");
        let seconds_remaining = delete_after.signed_duration_since(now).num_seconds().max(0);
        let remaining_days = u32::try_from((seconds_remaining + 86_399) / 86_400)
            .expect("Achieved retention fits u32");
        let urgency = match remaining_days {
            0..=2 => DeletionUrgency::Urgent,
            3..=7 => DeletionUrgency::Warning,
            _ => DeletionUrgency::Normal,
        };
        let (translation, translation_language) = self.displayed_translation(word.id)?;
        Ok(AchievedWordListItem {
            translation_language,
            id: word.id,
            lemma: word.lemma,
            display_form: word.display_form,
            translation,
            encounter_count: self.store.list_for_word(word.id)?.len(),
            achieved_at: word.achieved_at.expect("Achieved invariant checked"),
            delete_after,
            remaining_days,
            urgency,
        })
    }

    pub fn get_word(&self, word_id: Uuid) -> Result<WordDetail, ApplicationError> {
        let word = WordRepository::get(self.store.as_ref(), word_id)?
            .ok_or(ApplicationError::WordNotFound)?;
        let encounters = self.store.list_for_word(word_id)?;
        let last_seen_at = encounters
            .first()
            .map_or(word.updated_at, |encounter| encounter.captured_at);
        let (translation, translation_language) = self.displayed_translation(word_id)?;
        Ok(WordDetail {
            translations: self.store.translations(word_id)?,
            item: WordListItem {
                translation_language,
                id: word.id,
                display_form: word.display_form,
                translation,
                status: word.status,
                encounter_count: encounters.len(),
                next_review_at: word.review_state.as_ref().map(|state| state.due_at),
                last_seen_at,
                achieved_at: word.achieved_at,
                delete_after: word.delete_after,
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
        let words = self.words_with_captures()?;
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
        if !UserSettings::retention_days_are_valid(settings.achieved_retention_days) {
            return Err(ApplicationError::InvalidAchievedRetention);
        }
        SettingsRepository::save(self.store.as_ref(), &settings)?;
        Ok(())
    }
}

fn capture_record(request: CaptureRequest) -> CaptureRecord {
    CaptureRecord {
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
    }
}

fn validated_batch(ids: &[Uuid]) -> Result<Vec<Uuid>, ApplicationError> {
    let unique = ids.iter().copied().collect::<HashSet<_>>();
    if unique.is_empty() || unique.len() > 500 {
        return Err(ApplicationError::InvalidLifecycleBatch);
    }
    Ok(unique.into_iter().collect())
}
