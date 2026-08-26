#![cfg_attr(mobile, tauri::mobile_entry_point)]

use std::{fs, sync::Arc};

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use vocab_storage::SqliteStore;

pub mod bootstrap;
pub mod commands;
pub mod events;

use bootstrap::build_app_state;
use commands::{capture, library};
use events::NativeCaptureErrorEvent;

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
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(error) = capture::present_native_capture(&app).await
                                && let Some(window) = app.get_webview_window("capture")
                            {
                                let _ = window.show();
                                let _ = window
                                    .emit("capture-error", NativeCaptureErrorEvent::from(error));
                            }
                        });
                    }
                })
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&data_dir)?;
            let store = Arc::new(
                SqliteStore::open(data_dir.join("guest.db"))
                    .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?,
            );
            let platform = bootstrap::selected_platform()?;
            let state = build_app_state(store, platform);
            let mut settings = state.application().get_settings()?;
            if vocab_capture::parse_shortcut(&settings.capture_shortcut).is_err() {
                settings.capture_shortcut = "Alt+Shift+V".into();
                state.application().update_settings(settings.clone())?;
            }
            app.global_shortcut()
                .register(settings.capture_shortcut.as_str())?;
            state.configure_capture_window()?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            library::capture_word,
            library::undo_capture,
            library::get_today,
            library::list_words,
            library::get_word,
            library::submit_review,
            library::get_settings,
            library::update_settings,
            capture::replace_shortcut,
            capture::get_permission_status,
            capture::request_accessibility_permission,
            capture::capture_selected_text,
            capture::request_screen_recording_permission,
            capture::capture_with_ocr,
            capture::translate_text,
            capture::confirm_ocr,
            capture::save_native_capture,
            capture::hide_capture_window,
            capture::get_platform_capabilities,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vocab Collector");
}
