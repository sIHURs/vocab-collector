use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Encounter, ReviewLog, UserSettings, Word};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("record not found")]
    NotFound,
    #[error("persistence failure: {0}")]
    Persistence(String),
}

pub trait WordRepository: Send + Sync {
    fn find_by_dedupe_key(&self, key: &str) -> Result<Option<Word>, RepositoryError>;
    fn get(&self, id: Uuid) -> Result<Option<Word>, RepositoryError>;
    fn list(&self) -> Result<Vec<Word>, RepositoryError>;
    fn save(&self, word: &Word) -> Result<(), RepositoryError>;
}

pub trait EncounterRepository: Send + Sync {
    fn save(&self, encounter: &Encounter) -> Result<(), RepositoryError>;
    fn list_for_word(&self, word_id: Uuid) -> Result<Vec<Encounter>, RepositoryError>;
    fn soft_delete(&self, id: Uuid, deleted_at: DateTime<Utc>) -> Result<(), RepositoryError>;
}

pub trait ReviewRepository: Send + Sync {
    fn append(&self, review: &ReviewLog) -> Result<(), RepositoryError>;
    fn list_for_word(&self, word_id: Uuid) -> Result<Vec<ReviewLog>, RepositoryError>;
}

pub trait SettingsRepository: Send + Sync {
    fn get(&self) -> Result<UserSettings, RepositoryError>;
    fn save(&self, settings: &UserSettings) -> Result<(), RepositoryError>;
}
