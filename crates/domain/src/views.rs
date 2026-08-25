use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Encounter, UserSettings, WordStatus};

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayView {
    pub due_count: usize,
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
