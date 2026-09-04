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
