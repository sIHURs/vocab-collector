use chrono::Utc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;
use vocab_application::CaptureRequest;
use vocab_domain::{
    AchievedCaptureConflict, AchievedWordListItem, CaptureCard, GlobalInsight,
    LifecycleSweepResult, ReviewRating, ReviewResult, ReviewSessionInsight, TodayView,
    UserSettings, WordDetail, WordListItem,
};

use crate::bootstrap::AppState;

#[tauri::command]
pub fn change_learning_status(
    state: State<'_, AppState>,
    word_id: Uuid,
    status: vocab_domain::WordStatus,
) -> Result<Option<Uuid>, String> {
    state
        .application()
        .change_learning_status(word_id, status, Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn undo_learning_status(state: State<'_, AppState>, token: Uuid) -> Result<Uuid, String> {
    state
        .application()
        .undo_learning_status(token, Utc::now())
        .map_err(|error| error.to_string())
}

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
pub fn find_achieved_capture(
    state: State<'_, AppState>,
    request: CaptureRequest,
) -> Result<Option<AchievedCaptureConflict>, String> {
    state
        .application()
        .find_achieved_capture(&request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn restore_achieved_and_capture(
    state: State<'_, AppState>,
    word_id: Uuid,
    request: CaptureRequest,
) -> Result<CaptureCard, String> {
    state
        .application()
        .restore_achieved_and_capture(word_id, request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_global_insight(state: State<'_, AppState>) -> Result<GlobalInsight, String> {
    state
        .application()
        .get_global_insight()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn achieve_word(
    state: State<'_, AppState>,
    word_id: Uuid,
) -> Result<AchievedWordListItem, String> {
    state
        .application()
        .achieve_word(word_id, Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_achieved_words(
    state: State<'_, AppState>,
) -> Result<Vec<AchievedWordListItem>, String> {
    state
        .application()
        .run_lifecycle_sweep(Utc::now())
        .map_err(|error| error.to_string())?;
    state
        .application()
        .list_achieved_words_at(Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn run_lifecycle_sweep(state: State<'_, AppState>) -> Result<LifecycleSweepResult, String> {
    state
        .application()
        .run_lifecycle_sweep(Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn unachieve_words(state: State<'_, AppState>, word_ids: Vec<Uuid>) -> Result<usize, String> {
    state
        .application()
        .unachieve_words(&word_ids, Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_achieved_words(
    state: State<'_, AppState>,
    word_ids: Vec<Uuid>,
) -> Result<usize, String> {
    state
        .application()
        .delete_achieved_words(&word_ids, Utc::now())
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
pub fn get_today(
    state: State<'_, AppState>,
    completed_word_ids: Option<Vec<Uuid>>,
) -> Result<TodayView, String> {
    state
        .application()
        .run_lifecycle_sweep(Utc::now())
        .map_err(|error| error.to_string())?;
    state
        .application()
        .get_today_excluding(Utc::now(), &completed_word_ids.unwrap_or_default())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_words(state: State<'_, AppState>) -> Result<Vec<WordListItem>, String> {
    state
        .application()
        .run_lifecycle_sweep(Utc::now())
        .map_err(|error| error.to_string())?;
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
    submission_id: Uuid,
    word_id: Uuid,
    rating: ReviewRating,
) -> Result<ReviewResult, String> {
    state
        .application()
        .submit_review_once(submission_id, word_id, rating, Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_review_session_insight(
    state: State<'_, AppState>,
    submission_ids: Vec<Uuid>,
    next_day_end: chrono::DateTime<Utc>,
) -> Result<ReviewSessionInsight, String> {
    state
        .application()
        .get_review_session_insight(&submission_ids, next_day_end)
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
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: UserSettings,
) -> Result<(), String> {
    state
        .application()
        .update_settings(settings)
        .map_err(|error| error.to_string())?;
    let _ = app.emit("settings-changed", ());
    Ok(())
}

#[tauri::command]
pub fn get_vocabulary_log(
    state: State<'_, AppState>,
) -> Result<vocab_domain::VocabularyLog, String> {
    state
        .application()
        .get_vocabulary_log()
        .map_err(|error| error.to_string())
}
