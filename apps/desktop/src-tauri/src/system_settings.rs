use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use vocab_domain::UserSettings;

pub const NOTIFICATION_DELIVERY_ERROR: &str =
    "Windows could not deliver the Review notification. Check notification settings.";
pub const REVIEW_QUEUE_ERROR: &str =
    "Review reminders could not read the due queue. The app will retry.";

pub trait SettingsEffects {
    fn register_shortcut(&mut self, shortcut: &str) -> Result<(), String>;
    fn unregister_shortcut(&mut self, shortcut: &str) -> Result<(), String>;
    fn set_autostart(&mut self, enabled: bool) -> Result<(), String>;
    fn set_review_time(&mut self, review_time: &str) -> Result<(), String>;
    fn persist(&mut self, settings: &UserSettings) -> Result<(), String>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsApplyResult {
    pub settings: UserSettings,
    pub selection_shortcut_error: Option<String>,
    pub region_ocr_shortcut_error: Option<String>,
    pub autostart_error: Option<String>,
    pub notification_error: Option<String>,
}

pub fn restore_startup_shortcut(
    settings: &mut UserSettings,
    mut register: impl FnMut(&str) -> Result<(), String>,
    persist: impl FnOnce(&UserSettings) -> Result<(), String>,
) -> Option<String> {
    if settings.selection_capture_shortcut.is_empty()
        || register(&settings.selection_capture_shortcut).is_ok()
    {
        return None;
    }
    const FALLBACK: &str = "Alt+Shift+V";
    if settings.selection_capture_shortcut != FALLBACK && register(FALLBACK).is_ok() {
        settings.selection_capture_shortcut = FALLBACK.into();
        return Some(match persist(settings) {
            Ok(()) => {
                "The saved shortcut was unavailable. Alt+Shift+V is active instead.".into()
            }
            Err(_) => "The saved shortcut was unavailable. Alt+Shift+V is active for this session, but the preference could not be updated.".into(),
        });
    }
    Some("Global shortcut unavailable. Choose another combination in Settings.".into())
}

pub fn restore_startup_region_ocr_shortcut(
    settings: &UserSettings,
    mut register: impl FnMut(&str) -> Result<(), String>,
) -> Option<String> {
    if settings.region_ocr_capture_shortcut.is_empty()
        || register(&settings.region_ocr_capture_shortcut).is_ok()
    {
        None
    } else {
        Some("Region OCR shortcut unavailable. Choose another combination in Settings.".into())
    }
}

pub fn restore_startup_review_schedule(
    settings: &mut UserSettings,
    persist: impl FnOnce(&UserSettings) -> Result<(), String>,
) -> (ReviewScheduler, Option<String>) {
    if let Ok(scheduler) = ReviewScheduler::new(&settings.review_time) {
        return (scheduler, None);
    }

    settings.review_time = "18:00".into();
    let scheduler = ReviewScheduler::new(&settings.review_time)
        .expect("the built-in review time must remain valid");
    let error = match persist(settings) {
        Ok(()) => "The saved Review time was invalid. 18:00 is active instead.",
        Err(_) => {
            "The saved Review time was invalid. 18:00 is active for this session, but the preference could not be updated."
        }
    };
    (scheduler, Some(error.into()))
}

pub fn apply_settings_transaction(
    current: UserSettings,
    mut requested: UserSettings,
    effects: &mut impl SettingsEffects,
) -> Result<SettingsApplyResult, String> {
    let mut autostart_changed = false;
    let mut review_time_changed = false;
    let duplicate = !requested.selection_capture_shortcut.trim().is_empty()
        && requested.selection_capture_shortcut == requested.region_ocr_capture_shortcut;
    let (selection, selection_staged, mut selection_shortcut_error) = stage_shortcut(
        &current.selection_capture_shortcut,
        &requested.selection_capture_shortcut,
        effects,
    );
    requested.selection_capture_shortcut = selection;
    let (region, region_staged, mut region_ocr_shortcut_error) = if duplicate {
        (
            current.region_ocr_capture_shortcut.clone(),
            false,
            Some("Capture shortcuts must be different.".into()),
        )
    } else {
        stage_shortcut(
            &current.region_ocr_capture_shortcut,
            &requested.region_ocr_capture_shortcut,
            effects,
        )
    };
    requested.region_ocr_capture_shortcut = region;

    let autostart_error = if requested.launch_at_login != current.launch_at_login {
        match effects.set_autostart(requested.launch_at_login) {
            Ok(()) => {
                autostart_changed = true;
                None
            }
            Err(error) => {
                requested.launch_at_login = current.launch_at_login;
                Some(error)
            }
        }
    } else {
        None
    };

    let notification_error = if requested.review_time != current.review_time {
        match effects.set_review_time(&requested.review_time) {
            Ok(()) => {
                review_time_changed = true;
                None
            }
            Err(error) => {
                requested.review_time = current.review_time.clone();
                Some(error)
            }
        }
    } else {
        None
    };

    if let Err(error) = effects.persist(&requested) {
        let mut rollback_errors = Vec::new();
        if review_time_changed && let Err(rollback) = effects.set_review_time(&current.review_time)
        {
            rollback_errors.push(rollback);
        }
        if autostart_changed && let Err(rollback) = effects.set_autostart(current.launch_at_login) {
            rollback_errors.push(rollback);
        }
        for (staged, shortcut) in [
            (selection_staged, &requested.selection_capture_shortcut),
            (region_staged, &requested.region_ocr_capture_shortcut),
        ] {
            if staged
                && !shortcut.is_empty()
                && let Err(rollback) = effects.unregister_shortcut(shortcut)
            {
                rollback_errors.push(rollback);
            }
        }
        if rollback_errors.is_empty() {
            return Err(error);
        }
        return Err(format!(
            "{error}; rollback was incomplete: {}",
            rollback_errors.join("; ")
        ));
    }

    if selection_staged
        && !current.selection_capture_shortcut.is_empty()
        && effects
            .unregister_shortcut(&current.selection_capture_shortcut)
            .is_err()
    {
        if !requested.selection_capture_shortcut.is_empty() {
            let _ = effects.unregister_shortcut(&requested.selection_capture_shortcut);
        }
        requested.selection_capture_shortcut = current.selection_capture_shortcut.clone();
        selection_shortcut_error =
            Some("Shortcut unavailable. The previous shortcut is still active.".into());
        effects.persist(&requested)?;
    }
    if region_staged
        && !current.region_ocr_capture_shortcut.is_empty()
        && effects
            .unregister_shortcut(&current.region_ocr_capture_shortcut)
            .is_err()
    {
        if !requested.region_ocr_capture_shortcut.is_empty() {
            let _ = effects.unregister_shortcut(&requested.region_ocr_capture_shortcut);
        }
        requested.region_ocr_capture_shortcut = current.region_ocr_capture_shortcut.clone();
        region_ocr_shortcut_error =
            Some("Shortcut unavailable. The previous shortcut is still active.".into());
        effects.persist(&requested)?;
    }

    Ok(SettingsApplyResult {
        settings: requested,
        selection_shortcut_error,
        region_ocr_shortcut_error,
        autostart_error,
        notification_error,
    })
}

fn stage_shortcut(
    current: &str,
    requested: &str,
    effects: &mut impl SettingsEffects,
) -> (String, bool, Option<String>) {
    if requested == current {
        return (current.into(), false, None);
    }
    if requested.trim().is_empty() {
        return (String::new(), true, None);
    }
    match vocab_capture::parse_shortcut(requested) {
        Ok(shortcut) => match effects.register_shortcut(shortcut.canonical()) {
            Ok(()) => (shortcut.canonical().into(), true, None),
            Err(error) => (current.into(), false, Some(error)),
        },
        Err(error) => (current.into(), false, Some(error.to_string())),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewSchedule {
    time: NaiveTime,
    last_delivered: Option<NaiveDate>,
    last_attempt: Option<DateTime<Utc>>,
}

impl ReviewSchedule {
    pub fn parse(value: &str) -> Result<Self, String> {
        let time = NaiveTime::parse_from_str(value, "%H:%M")
            .map_err(|_| "Review time must use HH:MM.".to_string())?;
        Ok(Self {
            time,
            last_delivered: None,
            last_attempt: None,
        })
    }

    pub fn take_due<Tz: TimeZone>(&mut self, now: DateTime<Tz>) -> Option<NaiveDate> {
        let date = now.date_naive();
        let attempt_now = now.with_timezone(&Utc);
        if now.time() < self.time || self.last_delivered == Some(date) {
            return None;
        }
        if self
            .last_attempt
            .is_some_and(|attempt| attempt_now < attempt + TimeDelta::minutes(5))
        {
            return None;
        }
        self.last_attempt = Some(attempt_now);
        Some(date)
    }

    pub fn mark_delivered(&mut self, date: NaiveDate) {
        self.last_delivered = Some(date);
    }
}

#[derive(Clone)]
pub struct ReviewScheduler {
    schedule: Arc<Mutex<ReviewSchedule>>,
}

impl ReviewScheduler {
    pub fn new(review_time: &str) -> Result<Self, String> {
        Ok(Self {
            schedule: Arc::new(Mutex::new(ReviewSchedule::parse(review_time)?)),
        })
    }

    pub fn configure(&self, review_time: &str) -> Result<(), String> {
        let mut replacement = ReviewSchedule::parse(review_time)?;
        let mut schedule = self.schedule.lock().expect("review schedule lock poisoned");
        replacement.last_delivered = schedule.last_delivered;
        *schedule = replacement;
        Ok(())
    }

    pub fn take_due<Tz: TimeZone>(&self, now: DateTime<Tz>) -> Option<NaiveDate> {
        self.schedule
            .lock()
            .expect("review schedule lock poisoned")
            .take_due(now)
    }

    pub fn mark_delivered(&self, date: NaiveDate) {
        self.schedule
            .lock()
            .expect("review schedule lock poisoned")
            .mark_delivered(date);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSettingsStatus {
    pub selection_shortcut_error: Option<String>,
    pub region_ocr_shortcut_error: Option<String>,
    pub autostart_error: Option<String>,
    pub notification_error: Option<String>,
}

#[derive(Clone, Default)]
pub struct SystemSettingsRuntime {
    status: Arc<Mutex<SystemSettingsStatus>>,
}

impl SystemSettingsRuntime {
    pub fn status(&self) -> SystemSettingsStatus {
        self.status
            .lock()
            .expect("system settings status lock poisoned")
            .clone()
    }

    pub fn replace(&self, status: SystemSettingsStatus) {
        *self
            .status
            .lock()
            .expect("system settings status lock poisoned") = status;
    }

    pub fn set_notification_error(&self, error: Option<String>) {
        self.status
            .lock()
            .expect("system settings status lock poisoned")
            .notification_error = error;
    }
}

pub fn merge_attempted_status(
    existing: SystemSettingsStatus,
    result: &SettingsApplyResult,
    shortcut_attempted: bool,
    autostart_attempted: bool,
    review_time_attempted: bool,
) -> SettingsApplyResult {
    let mut merged = result.clone();
    if !shortcut_attempted {
        merged.selection_shortcut_error = existing.selection_shortcut_error;
        merged.region_ocr_shortcut_error = existing.region_ocr_shortcut_error;
    }
    if !autostart_attempted {
        merged.autostart_error = existing.autostart_error;
    }
    let preserve_notification_error = !review_time_attempted
        || (merged.notification_error.is_none()
            && existing.notification_error.as_deref().is_some_and(|error| {
                error == NOTIFICATION_DELIVERY_ERROR || error == REVIEW_QUEUE_ERROR
            }));
    if preserve_notification_error {
        merged.notification_error = existing.notification_error;
    }
    merged
}
