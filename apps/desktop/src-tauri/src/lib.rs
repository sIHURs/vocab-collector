#![cfg_attr(mobile, tauri::mobile_entry_point)]

use std::{fs, sync::Arc};

use chrono::Utc;
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use uuid::Uuid;
use vocab_application::{AppService, CaptureRequest};
use vocab_domain::{CaptureCard, ReviewRating, TodayView, UserSettings, WordDetail, WordListItem};
use vocab_storage::SqliteStore;

struct AppState(AppService);

#[tauri::command]
fn capture_word(
    state: State<'_, AppState>,
    request: CaptureRequest,
) -> Result<CaptureCard, String> {
    state.0.capture(request).map_err(|error| error.to_string())
}

#[tauri::command]
fn undo_capture(state: State<'_, AppState>, encounter_id: Uuid) -> Result<(), String> {
    state
        .0
        .undo_capture(encounter_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_today(state: State<'_, AppState>) -> Result<TodayView, String> {
    state
        .0
        .get_today(Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_words(state: State<'_, AppState>) -> Result<Vec<WordListItem>, String> {
    state.0.list_words().map_err(|error| error.to_string())
}

#[tauri::command]
fn get_word(state: State<'_, AppState>, word_id: Uuid) -> Result<WordDetail, String> {
    state.0.get_word(word_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn submit_review(
    state: State<'_, AppState>,
    word_id: Uuid,
    rating: ReviewRating,
) -> Result<(), String> {
    state
        .0
        .submit_review(word_id, rating, Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<UserSettings, String> {
    state.0.get_settings().map_err(|error| error.to_string())
}

#[tauri::command]
fn update_settings(state: State<'_, AppState>, settings: UserSettings) -> Result<(), String> {
    state
        .0
        .update_settings(settings)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn replace_shortcut(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    candidate: String,
) -> Result<UserSettings, String> {
    let parsed = vocab_capture::parse_shortcut(&candidate).map_err(|error| error.to_string())?;
    let mut settings = state.0.get_settings().map_err(|error| error.to_string())?;
    let previous = settings.capture_shortcut.clone();
    if previous == parsed.canonical() {
        return Ok(settings);
    }
    app.global_shortcut().register(parsed.canonical()).map_err(|_| "Shortcut unavailable. Try another combination.".to_string())?;
    settings.capture_shortcut = parsed.canonical().to_string();
    if let Err(error) = state.0.update_settings(settings.clone()) {
        let _ = app.global_shortcut().unregister(parsed.canonical());
        return Err(error.to_string());
    }
    if let Err(error) = app.global_shortcut().unregister(previous.as_str()) {
        let _ = app.global_shortcut().unregister(parsed.canonical());
        settings.capture_shortcut = previous;
        let _ = state.0.update_settings(settings.clone());
        return Err(error.to_string());
    }
    Ok(settings)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = app.emit("capture-shortcut-pressed", ());
                    }
                })
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&data_dir)?;
            let store = SqliteStore::open(data_dir.join("guest.db"))
                .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
            let service = AppService::new(Arc::new(store), Uuid::now_v7());
            let shortcut = service.get_settings()?.capture_shortcut;
            app.global_shortcut().register(shortcut.as_str())?;
            app.manage(AppState(service));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            capture_word,
            undo_capture,
            get_today,
            list_words,
            get_word,
            submit_review,
            get_settings,
            update_settings,
            replace_shortcut
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vocab Collector");
}
