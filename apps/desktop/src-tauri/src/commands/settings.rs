use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use vocab_domain::UserSettings;

use crate::{
    bootstrap::AppState,
    system_settings::{
        ReviewScheduler, SettingsApplyResult, SettingsEffects, SystemSettingsRuntime,
        SystemSettingsStatus, apply_settings_transaction, merge_attempted_status,
    },
};

struct TauriSettingsEffects<'a> {
    app: &'a AppHandle,
    application: &'a AppState,
    scheduler: &'a ReviewScheduler,
}

impl SettingsEffects for TauriSettingsEffects<'_> {
    fn register_shortcut(&mut self, shortcut: &str) -> Result<(), String> {
        self.app
            .global_shortcut()
            .register(shortcut)
            .map_err(|_| "Shortcut unavailable. Try another combination.".to_string())
    }

    fn unregister_shortcut(&mut self, shortcut: &str) -> Result<(), String> {
        self.app
            .global_shortcut()
            .unregister(shortcut)
            .map_err(|_| "The previous shortcut could not be released.".to_string())
    }

    fn set_autostart(&mut self, enabled: bool) -> Result<(), String> {
        let manager = self.app.autolaunch();
        if enabled {
            manager.enable()
        } else {
            manager.disable()
        }
        .map_err(|_| "Windows startup registration could not be updated.".to_string())?;
        let actual = manager
            .is_enabled()
            .map_err(|_| "Windows startup registration could not be verified.".to_string())?;
        if actual != enabled {
            return Err("Windows startup registration did not match the requested setting.".into());
        }
        Ok(())
    }

    fn set_review_time(&mut self, review_time: &str) -> Result<(), String> {
        self.scheduler.configure(review_time)
    }

    fn persist(&mut self, settings: &UserSettings) -> Result<(), String> {
        self.application
            .application()
            .update_settings(settings.clone())
            .map_err(|error| error.to_string())
    }
}

#[tauri::command]
pub fn apply_windows_settings(
    app: AppHandle,
    application: State<'_, AppState>,
    scheduler: State<'_, ReviewScheduler>,
    runtime: State<'_, SystemSettingsRuntime>,
    settings: UserSettings,
) -> Result<SettingsApplyResult, String> {
    let current = application
        .application()
        .get_settings()
        .map_err(|error| error.to_string())?;
    let shortcut_attempted = settings.capture_shortcut != current.capture_shortcut;
    let autostart_attempted = settings.launch_at_login != current.launch_at_login;
    let review_time_attempted = settings.review_time != current.review_time;
    let mut effects = TauriSettingsEffects {
        app: &app,
        application: &application,
        scheduler: &scheduler,
    };
    let result = apply_settings_transaction(current, settings, &mut effects)?;
    let result = merge_attempted_status(
        runtime.status(),
        &result,
        shortcut_attempted,
        autostart_attempted,
        review_time_attempted,
    );
    runtime.replace(SystemSettingsStatus {
        shortcut_error: result.shortcut_error.clone(),
        autostart_error: result.autostart_error.clone(),
        notification_error: result.notification_error.clone(),
    });
    Ok(result)
}

#[tauri::command]
pub fn get_windows_settings_status(
    runtime: State<'_, SystemSettingsRuntime>,
) -> SystemSettingsStatus {
    runtime.status()
}
