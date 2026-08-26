use chrono::Utc;
use tauri::State;
use uuid::Uuid;
use vocab_application::CaptureRequest;
use vocab_domain::{CaptureCard, ReviewRating, TodayView, UserSettings, WordDetail, WordListItem};

use crate::bootstrap::AppState;

#[tauri::command]
pub fn capture_word(
    state: State<'_, AppState>,
    request: CaptureRequest,
) -> Result<CaptureCard, String> {
    state
        .application()
        .capture(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn undo_capture(state: State<'_, AppState>, encounter_id: Uuid) -> Result<(), String> {
    state
        .application()
        .undo_capture(encounter_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_today(state: State<'_, AppState>) -> Result<TodayView, String> {
    state
        .application()
        .get_today(Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_words(state: State<'_, AppState>) -> Result<Vec<WordListItem>, String> {
    state
        .application()
        .list_words()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_word(state: State<'_, AppState>, word_id: Uuid) -> Result<WordDetail, String> {
    state
        .application()
        .get_word(word_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn submit_review(
    state: State<'_, AppState>,
    word_id: Uuid,
    rating: ReviewRating,
) -> Result<(), String> {
    state
        .application()
        .submit_review(word_id, rating, Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<UserSettings, String> {
    state
        .application()
        .get_settings()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppState>, settings: UserSettings) -> Result<(), String> {
    state
        .application()
        .update_settings(settings)
        .map_err(|error| error.to_string())
}
