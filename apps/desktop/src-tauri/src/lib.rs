#![cfg_attr(mobile, tauri::mobile_entry_point)]

use std::{fs, sync::Arc};

use chrono::Utc;
use serde::Serialize;
use tauri::{Emitter, LogicalPosition, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use uuid::Uuid;
use vocab_application::{AppService, CaptureRequest};
use vocab_domain::{CaptureCard, ReviewRating, TodayView, UserSettings, WordDetail, WordListItem};
use vocab_storage::SqliteStore;

#[cfg(target_os = "macos")]
mod macos_bridge;

struct AppState {
    service: AppService,
    coordinator: vocab_capture::CaptureCoordinator,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeCaptureEvent {
    request_id: Uuid,
    candidate: vocab_platform::CaptureCandidate,
}

#[derive(Debug)]
struct NativeCaptureError {
    request_id: Uuid,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeCaptureErrorEvent {
    request_id: Uuid,
    message: String,
}

#[tauri::command]
fn capture_word(
    state: State<'_, AppState>,
    request: CaptureRequest,
) -> Result<CaptureCard, String> {
    state
        .service
        .capture(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn undo_capture(state: State<'_, AppState>, encounter_id: Uuid) -> Result<(), String> {
    state
        .service
        .undo_capture(encounter_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_today(state: State<'_, AppState>) -> Result<TodayView, String> {
    state
        .service
        .get_today(Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_words(state: State<'_, AppState>) -> Result<Vec<WordListItem>, String> {
    state
        .service
        .list_words()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_word(state: State<'_, AppState>, word_id: Uuid) -> Result<WordDetail, String> {
    state
        .service
        .get_word(word_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn submit_review(
    state: State<'_, AppState>,
    word_id: Uuid,
    rating: ReviewRating,
) -> Result<(), String> {
    state
        .service
        .submit_review(word_id, rating, Utc::now())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<UserSettings, String> {
    state
        .service
        .get_settings()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn update_settings(state: State<'_, AppState>, settings: UserSettings) -> Result<(), String> {
    state
        .service
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
    let mut settings = state
        .service
        .get_settings()
        .map_err(|error| error.to_string())?;
    let previous = settings.capture_shortcut.clone();
    if previous == parsed.canonical() {
        return Ok(settings);
    }
    app.global_shortcut()
        .register(parsed.canonical())
        .map_err(|_| "Shortcut unavailable. Try another combination.".to_string())?;
    settings.capture_shortcut = parsed.canonical().to_string();
    if let Err(error) = state.service.update_settings(settings.clone()) {
        let _ = app.global_shortcut().unregister(parsed.canonical());
        return Err(error.to_string());
    }
    if let Err(error) = app.global_shortcut().unregister(previous.as_str()) {
        let _ = app.global_shortcut().unregister(parsed.canonical());
        settings.capture_shortcut = previous;
        let _ = state.service.update_settings(settings.clone());
        return Err(error.to_string());
    }
    Ok(settings)
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn get_permission_status(
    kind: vocab_platform::PermissionKind,
) -> Result<vocab_platform::PermissionStatus, String> {
    macos_bridge::permission_status(kind).map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn request_accessibility_permission() -> Result<vocab_platform::PermissionStatus, String> {
    macos_bridge::request_accessibility().map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn capture_selected_text() -> Result<vocab_platform::CaptureCandidate, String> {
    macos_bridge::capture_selection().map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn request_screen_recording_permission() -> Result<vocab_platform::PermissionStatus, String> {
    macos_bridge::request_screen_recording().map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn capture_with_ocr(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
) -> Result<(), String> {
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| "capture window is unavailable".to_string())?;
    window.hide().map_err(|error| error.to_string())?;
    let candidate = match macos_bridge::capture_ocr() {
        Ok(candidate) => candidate,
        Err(error) => {
            let _ = window.show();
            return Err(error.to_string());
        }
    };
    state
        .coordinator
        .set_candidate(request_id, candidate.clone())
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window
        .emit(
            "ocr-candidate",
            NativeCaptureEvent {
                request_id,
                candidate,
            },
        )
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
async fn translate_text(
    state: State<'_, AppState>,
    request_id: Uuid,
    text: String,
    source_language: String,
    target_language: String,
) -> Result<vocab_platform::TranslationResult, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        macos_bridge::translate(&text, &source_language, &target_language)
    })
    .await
    .map_err(|error| error.to_string())?;
    match result {
        Ok(translation) => {
            state
                .coordinator
                .set_translation(request_id, translation.clone())
                .map_err(|error| error.to_string())?;
            Ok(translation)
        }
        Err(error) => {
            state
                .coordinator
                .translation_failed(request_id)
                .map_err(|error| error.to_string())?;
            Err(error.to_string())
        }
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn confirm_ocr(state: State<'_, AppState>, request_id: Uuid) -> Result<(), String> {
    state
        .coordinator
        .confirm_ocr(request_id)
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn save_native_capture(
    state: State<'_, AppState>,
    request_id: Uuid,
    without_translation: bool,
) -> Result<CaptureCard, String> {
    let settings = state
        .service
        .get_settings()
        .map_err(|error| error.to_string())?;
    state
        .coordinator
        .save_with(request_id, without_translation, |snapshot| {
            let candidate = &snapshot.candidate;
            let translation = snapshot.translation.as_ref();
            state.service.capture(CaptureRequest {
                selected_text: candidate.selected_text.clone(),
                lemma: None,
                sentence: candidate.sentence.clone(),
                source_language: translation.map_or_else(
                    || settings.source_language.clone(),
                    |value| value.source_language.clone(),
                ),
                target_language: translation.map_or_else(
                    || settings.target_language.clone(),
                    |value| value.target_language.clone(),
                ),
                translation: translation.map(|value| value.translated_text.clone()),
                part_of_speech: None,
                source_app: candidate.source_app.clone(),
                source_title: candidate.source_title.clone(),
                source_url: candidate.source_url.clone(),
                capture_origin: candidate.origin,
                captured_at: Utc::now(),
            })
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn hide_capture_window(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("capture")
        .ok_or_else(|| "capture window is unavailable".to_string())?
        .hide()
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn present_native_capture(app: &tauri::AppHandle) -> Result<(), NativeCaptureError> {
    let state = app.state::<AppState>();
    let request_id = state.coordinator.start();
    let candidate = macos_bridge::capture_selection().map_err(|error| NativeCaptureError {
        request_id,
        message: error.to_string(),
    })?;
    state
        .coordinator
        .set_candidate(request_id, candidate.clone())
        .map_err(|error| NativeCaptureError {
            request_id,
            message: error.to_string(),
        })?;
    let window = app
        .get_webview_window("capture")
        .ok_or_else(|| NativeCaptureError {
            request_id,
            message: "capture window is unavailable".into(),
        })?;
    let pointer = app.cursor_position().map_err(|error| NativeCaptureError {
        request_id,
        message: error.to_string(),
    })?;
    let monitors = app
        .available_monitors()
        .map_err(|error| NativeCaptureError {
            request_id,
            message: error.to_string(),
        })?;
    let monitor_scale = monitors
        .iter()
        .find(|monitor| {
            let area = monitor.work_area();
            pointer.x >= f64::from(area.position.x)
                && pointer.x < f64::from(area.position.x) + f64::from(area.size.width)
                && pointer.y >= f64::from(area.position.y)
                && pointer.y < f64::from(area.position.y) + f64::from(area.size.height)
        })
        .map_or(1.0, |monitor| monitor.scale_factor());
    let work_areas = monitors
        .iter()
        .map(|monitor| {
            let area = monitor.work_area();
            let scale = monitor.scale_factor();
            vocab_platform::MonitorWorkArea::new(
                f64::from(area.position.x) / scale,
                f64::from(area.position.y) / scale,
                f64::from(area.size.width) / scale,
                f64::from(area.size.height) / scale,
                scale,
            )
        })
        .collect::<Vec<_>>();
    let anchor = candidate.selection_bounds.unwrap_or_default();
    let position = vocab_capture::place_floating_window(
        anchor,
        vocab_platform::ScreenPoint::new(pointer.x / monitor_scale, pointer.y / monitor_scale),
        &work_areas,
        vocab_platform::ScreenSize::new(380.0, 280.0),
    );
    window
        .set_position(LogicalPosition::new(position.x, position.y))
        .map_err(|error| NativeCaptureError {
            request_id,
            message: error.to_string(),
        })?;
    window.show().map_err(|error| NativeCaptureError {
        request_id,
        message: error.to_string(),
    })?;
    window
        .emit(
            "capture-ready",
            NativeCaptureEvent {
                request_id,
                candidate,
            },
        )
        .map_err(|error| NativeCaptureError {
            request_id,
            message: error.to_string(),
        })
}

pub fn run() {
    let shortcut_gate = Arc::new(vocab_capture::PressGate::default());
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    let state = match event.state() {
                        ShortcutState::Pressed => vocab_capture::ShortcutState::Pressed,
                        ShortcutState::Released => vocab_capture::ShortcutState::Released,
                    };
                    if shortcut_gate.accept(state) {
                        #[cfg(target_os = "macos")]
                        if let Err(error) = present_native_capture(app)
                            && let Some(window) = app.get_webview_window("capture")
                        {
                            let _ = window.show();
                            let _ = window.emit(
                                "capture-error",
                                NativeCaptureErrorEvent {
                                    request_id: error.request_id,
                                    message: error.message,
                                },
                            );
                        }
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
            let mut settings = service.get_settings()?;
            if vocab_capture::parse_shortcut(&settings.capture_shortcut).is_err() {
                settings.capture_shortcut = "Alt+Shift+V".into();
                service.update_settings(settings.clone())?;
            }
            let shortcut = settings.capture_shortcut;
            app.global_shortcut().register(shortcut.as_str())?;
            app.manage(AppState {
                service,
                coordinator: vocab_capture::CaptureCoordinator::default(),
            });
            #[cfg(target_os = "macos")]
            macos_bridge::configure_capture_window()?;
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
            replace_shortcut,
            #[cfg(target_os = "macos")]
            get_permission_status,
            #[cfg(target_os = "macos")]
            request_accessibility_permission,
            #[cfg(target_os = "macos")]
            capture_selected_text,
            #[cfg(target_os = "macos")]
            request_screen_recording_permission,
            #[cfg(target_os = "macos")]
            capture_with_ocr,
            #[cfg(target_os = "macos")]
            translate_text,
            #[cfg(target_os = "macos")]
            confirm_ocr,
            #[cfg(target_os = "macos")]
            save_native_capture,
            hide_capture_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vocab Collector");
}
