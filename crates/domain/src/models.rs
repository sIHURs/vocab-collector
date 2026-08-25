use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::normalize_context;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerScope {
    Guest,
    Account(Uuid),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WordStatus {
    #[default]
    Learning,
    Mastered,
    Paused,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewState {
    pub difficulty: f32,
    pub stability: f32,
    pub due_at: DateTime<Utc>,
    pub last_reviewed_at: Option<DateTime<Utc>>,
    pub lapse_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    pub id: Uuid,
    pub owner_scope: OwnerScope,
    pub lemma: String,
    pub display_form: String,
    pub source_language: String,
    pub target_language: String,
    pub translation: Option<String>,
    pub part_of_speech: Option<String>,
    pub status: WordStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub review_state: Option<ReviewState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub id: Uuid,
    pub word_id: Uuid,
    pub selected_text: String,
    pub sentence: String,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub source_url: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Encounter {
    pub fn new(
        word_id: Uuid,
        selected_text: String,
        sentence: String,
        source_app: Option<String>,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            word_id,
            selected_text,
            sentence: normalize_context(&sentence),
            source_app,
            source_title: None,
            source_url: None,
            captured_at,
            updated_at: captured_at,
            deleted_at: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewRating {
    Forgot,
    Remembered,
}

impl ReviewRating {
    pub const fn fsrs_grade(self) -> u8 {
        match self {
            Self::Forgot => 1,
            Self::Remembered => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewLog {
    pub id: Uuid,
    pub word_id: Uuid,
    pub rating: ReviewRating,
    pub reviewed_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub device_id: Uuid,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub source_language: String,
    pub target_language: String,
    pub capture_shortcut: String,
    pub review_time: String,
    pub daily_limit: usize,
    pub launch_at_login: bool,
    pub appearance: Appearance,
    pub reduced_motion: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            source_language: "en".into(),
            target_language: "de".into(),
            capture_shortcut: "Alt+Space+V".into(),
            review_time: "18:00".into(),
            daily_limit: 5,
            launch_at_login: false,
            appearance: Appearance::System,
            reduced_motion: false,
        }
    }
}
