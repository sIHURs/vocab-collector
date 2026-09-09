use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Encounter, ReviewRating, UserSettings, WordStatus};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureCard {
    pub word_id: Uuid,
    pub encounter_id: Uuid,
    pub display_form: String,
    pub translation: Option<String>,
    pub context: String,
    pub encounter_count: usize,
    pub is_existing_word: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordListItem {
    pub id: Uuid,
    pub display_form: String,
    pub translation: Option<String>,
    pub status: WordStatus,
    pub encounter_count: usize,
    pub next_review_at: Option<DateTime<Utc>>,
    pub last_seen_at: DateTime<Utc>,
    pub achieved_at: Option<DateTime<Utc>>,
    pub delete_after: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievedWordListItem {
    pub id: Uuid,
    pub lemma: String,
    pub display_form: String,
    pub translation: Option<String>,
    pub encounter_count: usize,
    pub achieved_at: DateTime<Utc>,
    pub delete_after: DateTime<Utc>,
    pub remaining_days: u32,
    pub urgency: DeletionUrgency,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletionUrgency {
    Normal,
    Warning,
    Urgent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievedCaptureConflict {
    pub word_id: Uuid,
    pub display_form: String,
    pub achieved_at: DateTime<Utc>,
    pub delete_after: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalInsight {
    pub current_vocabulary_count: usize,
    pub current_achieved_count: usize,
    pub lifetime_vocabulary_count: usize,
    pub lifetime_encounter_count: usize,
    pub lifetime_review_count: usize,
    pub lifetime_remembered_count: usize,
    pub lifetime_forgotten_count: usize,
    pub lifetime_rating_breakdown_complete: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleSweepResult {
    pub achieved_count: usize,
    pub purged_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCard {
    pub word_id: Uuid,
    pub display_form: String,
    pub translation: Option<String>,
    pub context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub submission_id: Uuid,
    pub word_id: Uuid,
    pub rating: ReviewRating,
    pub reviewed_at: DateTime<Utc>,
    pub previous_due_at: DateTime<Utc>,
    pub next_due_at: DateTime<Utc>,
    pub previous_stability: f32,
    pub stability: f32,
    pub difficulty: f32,
    pub lapse_count: u32,
    pub encounter_count: usize,
    pub repeated_forgetting: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSessionInsight {
    pub reviewed_count: usize,
    pub remembered_count: usize,
    pub forgotten_count: usize,
    pub attention_word_ids: Vec<Uuid>,
    pub next_day_due_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayView {
    pub total_due_count: usize,
    pub planned_review_count: usize,
    pub estimated_minutes: usize,
    pub review_queue: Vec<ReviewCard>,
    pub recent_captures: Vec<WordListItem>,
    pub settings: UserSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordDetail {
    pub item: WordListItem,
    pub lemma: String,
    pub part_of_speech: Option<String>,
    pub encounters: Vec<Encounter>,
}

/// Coverage is explicit: an unknown day is never represented as a zero.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogCoverage {
    Unknown,
    Partial,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyLogDay {
    pub date: chrono::NaiveDate,
    pub count: Option<u32>,
    pub coverage: LogCoverage,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyLog {
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub days: Vec<VocabularyLogDay>,
}
