#![cfg_attr(mobile, tauri::mobile_entry_point)]

use std::{fs, sync::Arc};

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use vocab_storage::SqliteStore;

pub mod bootstrap;
pub mod commands;
#[cfg(target_os = "windows")]
pub mod config;
pub mod events;
pub mod lifecycle;
pub mod system_settings;

use bootstrap::build_app_state;
use commands::{capture, library, presentation, settings};
use events::NativeCaptureErrorEvent;

pub fn shortcut_matches(triggered: &Shortcut, configured: &str) -> bool {
    configured
        .parse::<Shortcut>()
        .is_ok_and(|saved| saved == *triggered)
}

#[cfg(target_os = "windows")]
fn show_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window is unavailable".to_string())?;
    lifecycle::open_main_window(
        || {
            window.unminimize().map_err(|error| error.to_string())?;
            window.show().map_err(|error| error.to_string())
        },
        || window.set_focus().map_err(|error| error.to_string()),
    )
}

#[cfg(target_os = "windows")]
fn install_windows_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::{
        menu::{Menu, MenuItemBuilder},
        tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    };

    let open = MenuItemBuilder::with_id(lifecycle::TRAY_OPEN_ID, "Open").build(app)?;
    let exit = MenuItemBuilder::with_id(lifecycle::TRAY_EXIT_ID, "Exit").build(app)?;
    let menu = Menu::with_items(app, &[&open, &exit])?;
    let mut builder = TrayIconBuilder::with_id("vocab-collector")
        .tooltip("Vocab Collector")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            lifecycle::TRAY_OPEN_ID => {
                let _ = show_main_window(app);
            }
            lifecycle::TRAY_EXIT_ID => {
                let cleanup_app = app.clone();
                let exit_app = app.clone();
                let _ = lifecycle::terminate_session(
                    move || {
                        cleanup_app
                            .global_shortcut()
                            .unregister_all()
                            .map_err(|error| error.to_string())
                    },
                    move || exit_app.exit(0),
                );
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let _ = show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn start_review_scheduler(
    app: tauri::AppHandle,
    scheduler: system_settings::ReviewScheduler,
    runtime: system_settings::SystemSettingsRuntime,
) {
    use tauri_plugin_notification::NotificationExt;

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(15));
            let Some(date) = scheduler.take_due(chrono::Local::now()) else {
                continue;
            };
            let due_count = app
                .state::<bootstrap::AppState>()
                .application()
                .get_today(chrono::Utc::now())
                .map(|today| today.planned_review_count);
            let due_count = match due_count {
                Ok(due_count) => due_count,
                Err(_) => {
                    runtime.set_notification_error(Some(
                        system_settings::REVIEW_QUEUE_ERROR.to_string(),
                    ));
                    continue;
                }
            };
            if due_count == 0 {
                scheduler.mark_delivered(date);
                runtime.set_notification_error(None);
                continue;
            }
            let result = app
                .notification()
                .builder()
                .title("Vocab Collector")
                .body(format!("{due_count} words are ready for Review."))
                .show();
            match result {
                Ok(()) => {
                    scheduler.mark_delivered(date);
                    runtime.set_notification_error(None);
                }
                Err(_) => runtime.set_notification_error(Some(
                    system_settings::NOTIFICATION_DELIVERY_ERROR.to_string(),
                )),
            }
        }
    });
}

#[cfg(target_os = "windows")]
fn start_lifecycle_scheduler(app: tauri::AppHandle) {
    use tauri_plugin_notification::NotificationExt;
    std::thread::spawn(move || {
        let mut last_run_date = chrono::Local::now().date_naive();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
            let local_date = chrono::Local::now().date_naive();
            if local_date == last_run_date {
                continue;
            }
            last_run_date = local_date;
            if let Ok(result) = app
                .state::<bootstrap::AppState>()
                .application()
                .run_lifecycle_sweep(chrono::Utc::now())
                && result.achieved_count + result.purged_count > 0
            {
                let _ = app
                    .notification()
                    .builder()
                    .title("Vocab Collector")
                    .body(format!(
                        "{} words moved to Achieved; {} expired words deleted.",
                        result.achieved_count, result.purged_count
                    ))
                    .show();
                let _ = app.emit("library-changed", ());
            }
        }
    });
}

pub fn run() {
    let shortcut_gate = Arc::new(vocab_capture::PressGate::default());
    let builder = tauri::Builder::default().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                let state = match event.state() {
                    ShortcutState::Pressed => vocab_capture::ShortcutState::Pressed,
                    ShortcutState::Released => vocab_capture::ShortcutState::Released,
                };
                if shortcut_gate.accept(state) {
                    let app = app.clone();
                    let shortcut = *shortcut;
                    tauri::async_runtime::spawn(async move {
                        let is_region_ocr = app
                            .state::<bootstrap::AppState>()
                            .application()
                            .get_settings()
                            .is_ok_and(|settings| {
                                !settings.region_ocr_capture_shortcut.is_empty()
                                    && shortcut_matches(
                                        &shortcut,
                                        &settings.region_ocr_capture_shortcut,
                                    )
                            });
                        if is_region_ocr {
                            let _ = capture::present_region_ocr_capture(&app);
                            return;
                        }
                        if let Err(error) = capture::present_native_capture(&app).await {
                            let request_id = error.request_id;
                            let state = app.state::<bootstrap::AppState>();
                            let _ = state.publish_if_current(request_id, || {
                                if let Some(window) = app.get_webview_window("capture") {
                                    let _ = window.show();
                                    let _ = window.emit(
                                        "capture-error",
                                        NativeCaptureErrorEvent::from(error),
                                    );
                                }
                            });
                        }
                    });
                }
            })
            .build(),
    );
    #[cfg(target_os = "windows")]
    let builder = builder
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        )
        .plugin(tauri_plugin_notification::init());
    builder
        .setup(|app| {
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                window.set_decorations(false)?;
            }
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&data_dir)?;
            let store = Arc::new(
                SqliteStore::open(data_dir.join("guest.db"))
                    .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?,
            );
            let platform = bootstrap::selected_platform()?;
            let state = build_app_state(store, platform);
            let mut settings = state.application().get_settings()?;
            if !settings.selection_capture_shortcut.is_empty()
                && vocab_capture::parse_shortcut(&settings.selection_capture_shortcut).is_err()
            {
                settings.selection_capture_shortcut = "Alt+Shift+V".into();
                state.application().update_settings(settings.clone())?;
            }
            state.configure_capture_window()?;
            #[cfg(target_os = "windows")]
            {
                use tauri_plugin_autostart::ManagerExt;

                if let Some(window) = app.get_webview_window("capture") {
                    vocab_platform_windows::window::configure_capture_window(
                        vocab_platform_windows::window::NativeWindowHandle::new(
                            window.hwnd()?.0 as isize,
                        ),
                    )?;
                }

                let runtime = system_settings::SystemSettingsRuntime::default();
                let shortcut_error = system_settings::restore_startup_shortcut(
                    &mut settings,
                    |shortcut| {
                        app.global_shortcut()
                            .register(shortcut)
                            .map_err(|error| error.to_string())
                    },
                    |settings| {
                        state
                            .application()
                            .update_settings(settings.clone())
                            .map_err(|error| error.to_string())
                    },
                );
                if shortcut_error.is_some() {
                    let mut status = runtime.status();
                    status.selection_shortcut_error = shortcut_error;
                    runtime.replace(status);
                }
                let region_ocr_shortcut_error =
                    system_settings::restore_startup_region_ocr_shortcut(&settings, |shortcut| {
                        app.global_shortcut()
                            .register(shortcut)
                            .map_err(|error| error.to_string())
                    });
                if region_ocr_shortcut_error.is_some() {
                    let mut status = runtime.status();
                    status.region_ocr_shortcut_error = region_ocr_shortcut_error;
                    runtime.replace(status);
                }
                let (scheduler, review_time_error) =
                    system_settings::restore_startup_review_schedule(&mut settings, |settings| {
                        state
                            .application()
                            .update_settings(settings.clone())
                            .map_err(|error| error.to_string())
                    });
                if review_time_error.is_some() {
                    let mut status = runtime.status();
                    status.notification_error = review_time_error;
                    runtime.replace(status);
                }
                let startup_lifecycle = state
                    .application()
                    .run_lifecycle_sweep(chrono::Utc::now())
                    .unwrap_or_default();
                app.manage(state);
                app.manage(scheduler.clone());
                app.manage(runtime.clone());

                let autostart_manager = app.autolaunch();
                let autostart_result = if settings.launch_at_login {
                    autostart_manager.enable()
                } else {
                    autostart_manager.disable()
                }
                .and_then(|()| {
                    autostart_manager.is_enabled().and_then(|enabled| {
                        if enabled == settings.launch_at_login {
                            Ok(())
                        } else {
                            Err(tauri_plugin_autostart::Error::Anyhow(
                                "startup state mismatch".into(),
                            ))
                        }
                    })
                });
                if autostart_result.is_err() {
                    let mut status = runtime.status();
                    status.autostart_error =
                        Some("Windows startup registration could not be restored.".into());
                    runtime.replace(status);
                }
                install_windows_tray(app.handle())?;
                if startup_lifecycle.achieved_count + startup_lifecycle.purged_count > 0 {
                    use tauri_plugin_notification::NotificationExt;
                    let _ = app
                        .notification()
                        .builder()
                        .title("Vocab Collector")
                        .body(format!(
                            "{} words moved to Achieved; {} expired words deleted.",
                            startup_lifecycle.achieved_count, startup_lifecycle.purged_count
                        ))
                        .show();
                }
                if std::env::args().any(|argument| argument == "--autostart")
                    && let Some(window) = app.get_webview_window("main")
                {
                    window.hide()?;
                }
                start_review_scheduler(app.handle().clone(), scheduler, runtime);
                start_lifecycle_scheduler(app.handle().clone());
            }
            #[cfg(not(target_os = "windows"))]
            {
                if !settings.selection_capture_shortcut.is_empty() {
                    app.global_shortcut()
                        .register(settings.selection_capture_shortcut.as_str())?;
                }
                if !settings.region_ocr_capture_shortcut.is_empty() {
                    app.global_shortcut()
                        .register(settings.region_ocr_capture_shortcut.as_str())?;
                }
                app.manage(state);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(target_os = "windows")]
            if window.label() == "main"
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            library::capture_word,
            library::find_achieved_capture,
            library::restore_achieved_and_capture,
            library::get_global_insight,
            library::get_vocabulary_log,
            library::undo_capture,
            library::get_today,
            library::list_words,
            library::list_achieved_words,
            library::achieve_word,
            library::unachieve_words,
            library::delete_achieved_words,
            library::run_lifecycle_sweep,
            library::get_word,
            library::change_learning_status,
            library::undo_learning_status,
            library::submit_review,
            library::get_review_session_insight,
            library::get_settings,
            library::update_settings,
            settings::apply_windows_settings,
            settings::get_windows_settings_status,
            capture::replace_shortcut,
            capture::get_permission_status,
            capture::request_accessibility_permission,
            capture::capture_selected_text,
            capture::request_screen_recording_permission,
            capture::capture_ocr_region,
            capture::cancel_region_ocr_capture,
            capture::show_region_ocr_failure,
            capture::open_manual_capture,
            capture::start_region_ocr_capture,
            capture::translate_text,
            capture::confirm_ocr,
            capture::save_native_capture,
            capture::find_achieved_native_capture,
            capture::restore_achieved_and_save_native_capture,
            capture::correct_native_capture,
            capture::undo_native_capture,
            capture::close_capture_window,
            capture::hide_capture_window,
            capture::focus_capture_window,
            capture::release_capture_window_focus,
            capture::get_platform_capabilities,
            presentation::get_presentation_family,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vocab Collector");
}
