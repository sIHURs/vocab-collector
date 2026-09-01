use std::{
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    thread,
};

use async_trait::async_trait;
use vocab_application::PlatformCaptureError;
use vocab_capture::CoordinatorError;
use vocab_desktop_lib::{
    bootstrap::build_app_state,
    commands::capture::{
        capture_selected_text, capture_with_ocr, confirm_ocr, correct_native_capture,
        get_permission_status, get_platform_capabilities, hide_capture_window,
        hide_capture_window_for, request_accessibility_permission,
        request_screen_recording_permission, save_native_capture, translate_text,
        undo_native_capture,
    },
    commands::presentation::get_presentation_family,
    events::{CaptureFailure, CaptureFailureCode, NativeCaptureError, NativeCaptureErrorEvent},
    lifecycle::{open_main_window, terminate_session},
    system_settings::{
        ReviewSchedule, SettingsEffects, SystemSettingsStatus, apply_settings_transaction,
        merge_attempted_status, restore_startup_review_schedule, restore_startup_shortcut,
    },
};
use vocab_domain::{CaptureOrigin, UserSettings};
use vocab_platform_api::{
    CaptureCandidate, OcrCandidate, OcrProvider, PermissionKind, PermissionProvider,
    PermissionStatus, PlatformCapabilities, PlatformError, PlatformServices, ScreenPoint,
    ScreenRect, SelectionProvider, TranslationProvider, TranslationResult, WindowProvider,
};
use vocab_storage::SqliteStore;

#[test]
fn capture_command_names_are_available_on_the_platform_neutral_surface() {
    let _ = get_permission_status;
    let _ = request_accessibility_permission;
    let _ = capture_selected_text;
    let _ = request_screen_recording_permission;
    let _ = capture_with_ocr;
    let _ = translate_text;
    let _ = confirm_ocr;
    let _ = save_native_capture;
    let _ = correct_native_capture;
    let _ = undo_native_capture;
    let _ = hide_capture_window;
    let _ = get_platform_capabilities;
}

#[test]
fn desktop_exposes_the_target_presentation_family() {
    let expected = if cfg!(target_os = "windows") {
        "windows"
    } else {
        "shared"
    };
    assert_eq!(get_presentation_family(), expected);
}

#[derive(Default)]
struct RecordingSettingsEffects {
    calls: Vec<String>,
    reject_persistence: bool,
    reject_unregister: Vec<String>,
}

impl SettingsEffects for RecordingSettingsEffects {
    fn register_shortcut(&mut self, shortcut: &str) -> Result<(), String> {
        self.calls.push(format!("register:{shortcut}"));
        Ok(())
    }

    fn unregister_shortcut(&mut self, shortcut: &str) -> Result<(), String> {
        self.calls.push(format!("unregister:{shortcut}"));
        if self.reject_unregister.iter().any(|value| value == shortcut) {
            Err(format!("cannot unregister {shortcut}"))
        } else {
            Ok(())
        }
    }

    fn set_autostart(&mut self, enabled: bool) -> Result<(), String> {
        self.calls.push(format!("autostart:{enabled}"));
        Ok(())
    }

    fn set_review_time(&mut self, review_time: &str) -> Result<(), String> {
        self.calls.push(format!("review:{review_time}"));
        Ok(())
    }

    fn persist(&mut self, settings: &UserSettings) -> Result<(), String> {
        self.calls
            .push(format!("persist:{}", settings.capture_shortcut));
        if self.reject_persistence {
            Err("database unavailable".into())
        } else {
            Ok(())
        }
    }
}

#[test]
fn settings_status_keeps_errors_for_system_effects_that_were_not_attempted() {
    let existing = SystemSettingsStatus {
        shortcut_error: None,
        autostart_error: Some("startup registration is unresolved".into()),
        notification_error: None,
    };
    let result = vocab_desktop_lib::system_settings::SettingsApplyResult {
        settings: UserSettings::default(),
        shortcut_error: None,
        autostart_error: None,
        notification_error: None,
    };

    assert_eq!(
        merge_attempted_status(existing, &result, false, false, false).autostart_error,
        Some("startup registration is unresolved".into())
    );
}

#[test]
fn shortcut_rollback_reports_when_the_staged_shortcut_cannot_be_removed() {
    let current = UserSettings::default();
    let mut requested = current.clone();
    requested.capture_shortcut = "Control+Shift+W".into();
    let mut effects = RecordingSettingsEffects {
        reject_unregister: vec!["Alt+Shift+V".into(), "Control+Shift+W".into()],
        ..Default::default()
    };

    let result = apply_settings_transaction(current, requested, &mut effects).unwrap();

    assert_eq!(result.settings.capture_shortcut, "Alt+Shift+V");
    assert_eq!(
        result.shortcut_error.as_deref(),
        Some(
            "The previous shortcut could not be released, and the staged shortcut could not be removed. Restart the app to restore a single shortcut."
        )
    );
}

#[test]
fn settings_effects_commit_before_the_previous_shortcut_is_removed() {
    let current = UserSettings::default();
    let mut requested = current.clone();
    requested.capture_shortcut = "Control+Shift+W".into();
    requested.launch_at_login = true;
    requested.review_time = "08:30".into();
    let mut effects = RecordingSettingsEffects::default();

    let result = apply_settings_transaction(current, requested.clone(), &mut effects).unwrap();

    assert_eq!(result.settings, requested);
    assert!(result.shortcut_error.is_none());
    assert!(result.autostart_error.is_none());
    assert!(result.notification_error.is_none());
    assert_eq!(
        effects.calls,
        [
            "register:Control+Shift+W",
            "autostart:true",
            "review:08:30",
            "persist:Control+Shift+W",
            "unregister:Alt+Shift+V",
        ]
    );
}

#[test]
fn settings_persistence_failure_rolls_back_every_staged_system_effect() {
    let current = UserSettings::default();
    let mut requested = current.clone();
    requested.capture_shortcut = "Control+Shift+W".into();
    requested.launch_at_login = true;
    requested.review_time = "08:30".into();
    let mut effects = RecordingSettingsEffects {
        reject_persistence: true,
        ..Default::default()
    };

    assert_eq!(
        apply_settings_transaction(current, requested, &mut effects).unwrap_err(),
        "database unavailable"
    );
    assert_eq!(
        effects.calls,
        [
            "register:Control+Shift+W",
            "autostart:true",
            "review:08:30",
            "persist:Control+Shift+W",
            "review:18:00",
            "autostart:false",
            "unregister:Control+Shift+W",
        ]
    );
}

#[test]
fn startup_shortcut_conflict_falls_back_without_blocking_application_start() {
    let mut settings = UserSettings {
        capture_shortcut: "Control+Shift+W".into(),
        ..UserSettings::default()
    };
    let calls = Arc::new(Mutex::new(Vec::new()));
    let register_calls = Arc::clone(&calls);
    let persist_calls = Arc::clone(&calls);

    let error = restore_startup_shortcut(
        &mut settings,
        move |shortcut| {
            register_calls
                .lock()
                .unwrap()
                .push(format!("register:{shortcut}"));
            if shortcut == "Control+Shift+W" {
                Err("conflict".into())
            } else {
                Ok(())
            }
        },
        move |settings| {
            persist_calls
                .lock()
                .unwrap()
                .push(format!("persist:{}", settings.capture_shortcut));
            Ok(())
        },
    );

    assert_eq!(settings.capture_shortcut, "Alt+Shift+V");
    assert_eq!(
        error.as_deref(),
        Some("The saved shortcut was unavailable. Alt+Shift+V is active instead.")
    );
    assert_eq!(
        *calls.lock().unwrap(),
        [
            "register:Control+Shift+W",
            "register:Alt+Shift+V",
            "persist:Alt+Shift+V",
        ]
    );
}

#[test]
fn invalid_startup_review_time_falls_back_without_blocking_application_start() {
    let mut settings = UserSettings {
        review_time: "25:99".into(),
        ..UserSettings::default()
    };
    let persisted = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&persisted);

    let (schedule, error) = restore_startup_review_schedule(&mut settings, move |settings| {
        observed.lock().unwrap().push(settings.review_time.clone());
        Ok(())
    });

    assert_eq!(settings.review_time, "18:00");
    assert_eq!(*persisted.lock().unwrap(), ["18:00"]);
    assert_eq!(
        error.as_deref(),
        Some("The saved Review time was invalid. 18:00 is active instead.")
    );
    let due = chrono::DateTime::parse_from_rfc3339("2026-09-01T18:00:05+02:00").unwrap();
    assert!(schedule.take_due(due).is_some());
}

#[test]
fn review_schedule_fires_once_per_local_date() {
    let mut schedule = ReviewSchedule::parse("18:00").unwrap();
    let first = chrono::DateTime::parse_from_rfc3339("2026-09-01T18:00:05+02:00").unwrap();
    let repeated = chrono::DateTime::parse_from_rfc3339("2026-09-01T18:00:45+02:00").unwrap();
    let next_day = chrono::DateTime::parse_from_rfc3339("2026-09-02T18:00:01+02:00").unwrap();

    let date = schedule.take_due(first).unwrap();
    schedule.mark_delivered(date);
    assert!(schedule.take_due(repeated).is_none());
    assert!(schedule.take_due(next_day).is_some());
}

#[test]
fn failed_review_delivery_retries_at_a_bounded_interval() {
    let mut schedule = ReviewSchedule::parse("18:00").unwrap();
    let first = chrono::DateTime::parse_from_rfc3339("2026-09-01T18:00:05+02:00").unwrap();
    let too_soon = chrono::DateTime::parse_from_rfc3339("2026-09-01T18:04:59+02:00").unwrap();
    let retry = chrono::DateTime::parse_from_rfc3339("2026-09-01T18:05:05+02:00").unwrap();

    assert!(schedule.take_due(first).is_some());
    assert!(schedule.take_due(too_soon).is_none());
    assert!(schedule.take_due(retry).is_some());
}

#[test]
fn review_retry_interval_is_not_extended_when_the_local_time_zone_moves_backward() {
    let mut schedule = ReviewSchedule::parse("18:00").unwrap();
    let first = chrono::DateTime::parse_from_rfc3339("2026-09-01T18:00:05+12:00").unwrap();
    let after_zone_change =
        chrono::DateTime::parse_from_rfc3339("2026-09-01T18:01:05-12:00").unwrap();

    assert!(schedule.take_due(first).is_some());
    assert!(schedule.take_due(after_zone_change).is_some());
}

#[test]
fn changing_review_time_keeps_a_successful_delivery_suppressed_for_that_date() {
    let scheduler = vocab_desktop_lib::system_settings::ReviewScheduler::new("18:00").unwrap();
    let delivered = chrono::DateTime::parse_from_rfc3339("2026-09-01T18:00:05+02:00").unwrap();
    let date = scheduler.take_due(delivered).unwrap();
    scheduler.mark_delivered(date);

    scheduler.configure("19:00").unwrap();

    let same_date = chrono::DateTime::parse_from_rfc3339("2026-09-01T19:00:05+02:00").unwrap();
    assert!(scheduler.take_due(same_date).is_none());
}

#[test]
fn changing_review_time_does_not_clear_an_unresolved_delivery_failure() {
    let existing = SystemSettingsStatus {
        shortcut_error: None,
        autostart_error: None,
        notification_error: Some(
            vocab_desktop_lib::system_settings::NOTIFICATION_DELIVERY_ERROR.into(),
        ),
    };
    let result = vocab_desktop_lib::system_settings::SettingsApplyResult {
        settings: UserSettings::default(),
        shortcut_error: None,
        autostart_error: None,
        notification_error: None,
    };

    assert_eq!(
        merge_attempted_status(existing, &result, false, false, true).notification_error,
        Some(vocab_desktop_lib::system_settings::NOTIFICATION_DELIVERY_ERROR.into())
    );
}

#[test]
fn desktop_lifecycle_opens_then_focuses_and_cleans_before_exit() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let show_calls = Arc::clone(&calls);
    let focus_calls = Arc::clone(&calls);
    open_main_window(
        move || {
            show_calls.lock().unwrap().push("show");
            Ok(())
        },
        move || {
            focus_calls.lock().unwrap().push("focus");
            Ok(())
        },
    )
    .unwrap();
    let cleanup_calls = Arc::clone(&calls);
    let exit_calls = Arc::clone(&calls);
    terminate_session(
        move || {
            cleanup_calls.lock().unwrap().push("cleanup");
            Ok(())
        },
        move || exit_calls.lock().unwrap().push("exit"),
    )
    .unwrap();

    assert_eq!(*calls.lock().unwrap(), ["show", "focus", "cleanup", "exit"]);
}

#[test]
fn passive_capture_presentation_positions_then_shows_without_activation_before_emitting() {
    use vocab_desktop_lib::commands::capture::present_capture_window;

    let calls = Arc::new(Mutex::new(Vec::new()));
    let position_calls = Arc::clone(&calls);
    let show_calls = Arc::clone(&calls);
    let emit_calls = Arc::clone(&calls);

    present_capture_window(
        || {
            position_calls.lock().unwrap().push("position");
            Ok(())
        },
        || {
            show_calls.lock().unwrap().push("show-no-activate");
            Ok(())
        },
        || {
            emit_calls.lock().unwrap().push("emit");
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(
        *calls.lock().unwrap(),
        ["position", "show-no-activate", "emit"]
    );
}

#[test]
fn successful_capture_action_restores_focus_but_failed_action_does_not() {
    use vocab_desktop_lib::commands::capture::complete_capture_action;

    let restored = AtomicBool::new(false);
    let success: Result<&str, &str> =
        complete_capture_action(|| Ok("saved"), || restored.store(true, Ordering::SeqCst));
    assert_eq!(success, Ok("saved"));
    assert!(restored.load(Ordering::SeqCst));

    restored.store(false, Ordering::SeqCst);
    let failure: Result<&str, &str> = complete_capture_action(
        || Err("not saved"),
        || restored.store(true, Ordering::SeqCst),
    );
    assert_eq!(failure, Err("not saved"));
    assert!(!restored.load(Ordering::SeqCst));
}

#[test]
fn desktop_exit_still_terminates_when_explicit_cleanup_reports_an_error() {
    let exited = Arc::new(AtomicBool::new(false));
    let exit_observer = Arc::clone(&exited);

    assert_eq!(
        terminate_session(
            || Err("shortcut cleanup failed".into()),
            move || exit_observer.store(true, Ordering::SeqCst),
        )
        .unwrap_err(),
        "shortcut cleanup failed"
    );
    assert!(exited.load(Ordering::SeqCst));
}

#[test]
fn platform_error_variants_serialize_to_the_exact_capture_failure_contract() {
    let cases = [
        (
            PlatformError::PermissionRequired(PermissionKind::Accessibility),
            CaptureFailureCode::PermissionRequired,
            "permission_required",
        ),
        (
            PlatformError::PermissionDenied(PermissionKind::ScreenRecording),
            CaptureFailureCode::PermissionDenied,
            "permission_denied",
        ),
        (
            PlatformError::EmptySelection,
            CaptureFailureCode::EmptySelection,
            "empty_selection",
        ),
        (
            PlatformError::UnsupportedElement,
            CaptureFailureCode::UnsupportedElement,
            "unsupported_element",
        ),
        (
            PlatformError::Unsupported(vocab_platform_api::Capability::Translation),
            CaptureFailureCode::TranslationUnavailable,
            "translation_unavailable",
        ),
        (
            PlatformError::Cancelled,
            CaptureFailureCode::Cancelled,
            "cancelled",
        ),
        (
            PlatformError::Operation("diagnostic".into()),
            CaptureFailureCode::Operation,
            "operation",
        ),
    ];

    for (error, expected_code, serialized_code) in cases {
        let failure = CaptureFailure::from(error);
        assert_eq!(failure.code, expected_code);
        assert_eq!(
            serde_json::to_value(&failure).unwrap()["code"],
            serialized_code
        );
    }
}

#[test]
fn native_capture_error_serializes_the_exact_frontend_event_contract() {
    let request_id = uuid::Uuid::parse_str("018f5d2e-6f53-7cc4-a6da-bf11a2f9c221").unwrap();
    let event = NativeCaptureErrorEvent::from(NativeCaptureError {
        request_id,
        failure: CaptureFailure::from(PlatformError::EmptySelection),
    });

    assert_eq!(
        serde_json::to_value(event).unwrap(),
        serde_json::json!({
            "requestId": "018f5d2e-6f53-7cc4-a6da-bf11a2f9c221",
            "code": "empty_selection",
            "message": "selection is empty",
        })
    );
}

#[test]
fn non_translation_unsupported_and_internal_failures_map_to_operation() {
    let unsupported = CaptureFailure::from(PlatformError::Unsupported(
        vocab_platform_api::Capability::ScreenshotOcr,
    ));
    let invalid_range = CaptureFailure::from(PlatformError::InvalidSelectionRange);
    let coordinator = CaptureFailure::from(PlatformCaptureError::Coordinator(
        CoordinatorError::StaleRequest,
    ));

    assert_eq!(unsupported.code, CaptureFailureCode::Operation);
    assert_eq!(invalid_range.code, CaptureFailureCode::Operation);
    assert_eq!(coordinator.code, CaptureFailureCode::Operation);
}

#[test]
fn translation_operation_failures_have_a_typed_retryable_context() {
    let failure = CaptureFailure::from_translation(PlatformCaptureError::Platform(
        PlatformError::Operation("native translation timed out".into()),
    ));

    assert_eq!(failure.code, CaptureFailureCode::TranslationFailed);
    assert_eq!(
        serde_json::to_value(&failure).unwrap()["code"],
        "translation_failed"
    );
}

#[test]
fn app_state_routes_capture_operations_through_injected_platform_services() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let capabilities = PlatformCapabilities {
        selection_capture: true,
        selection_bounds: false,
        screenshot_ocr: true,
        translation: true,
        non_activating_window: false,
    };
    let state = build_app_state(
        Arc::new(SqliteStore::open_in_memory().unwrap()),
        platform_services(capabilities, calls.clone()),
    );

    assert_eq!(state.platform_capabilities(), capabilities);
    assert_eq!(
        block_on(state.permission_status(PermissionKind::Accessibility)).unwrap(),
        PermissionStatus::Granted
    );
    assert_eq!(
        block_on(state.request_permission(PermissionKind::ScreenRecording)).unwrap(),
        PermissionStatus::Granted
    );
    assert_eq!(
        block_on(state.capture_selection()).unwrap().selected_text,
        "portable selection"
    );
    assert_eq!(
        block_on(state.recognize_near(ScreenPoint::new(25.0, 75.0))).unwrap()[0].text,
        "portable OCR"
    );
    assert_eq!(
        block_on(state.translate("portable selection", "en", "de"))
            .unwrap()
            .translated_text,
        "portable translation"
    );

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [
            "permission-status:Accessibility",
            "permission-request:ScreenRecording",
            "selection",
            "ocr:25,75",
            "translation:portable selection:en:de",
        ]
    );
}

#[test]
fn unavailable_non_activating_window_capability_skips_window_configuration() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let state = build_app_state(
        Arc::new(SqliteStore::open_in_memory().unwrap()),
        platform_services(
            PlatformCapabilities {
                non_activating_window: false,
                ..PlatformCapabilities::default()
            },
            calls.clone(),
        ),
    );

    state.configure_capture_window().unwrap();

    assert!(!calls.lock().unwrap().iter().any(|call| call == "window"));
}

#[test]
fn available_non_activating_window_capability_configures_the_window() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let state = build_app_state(
        Arc::new(SqliteStore::open_in_memory().unwrap()),
        platform_services(
            PlatformCapabilities {
                non_activating_window: true,
                ..PlatformCapabilities::default()
            },
            calls.clone(),
        ),
    );

    state.configure_capture_window().unwrap();

    assert_eq!(calls.lock().unwrap().as_slice(), ["window"]);
}

#[test]
fn stale_hide_does_not_run_the_window_side_effect() {
    let state = build_app_state(
        Arc::new(SqliteStore::open_in_memory().unwrap()),
        platform_services(
            PlatformCapabilities::default(),
            Arc::new(Mutex::new(Vec::new())),
        ),
    );
    let stale = state.start_capture_request();
    let current = state.start_capture_request();
    let hidden = AtomicBool::new(false);

    let result = hide_capture_window_for(&state, stale, || {
        hidden.store(true, Ordering::SeqCst);
        Ok(())
    });

    assert!(result.is_err());
    assert!(!hidden.load(Ordering::SeqCst));
    assert!(state.is_current_capture_request(current));
}

#[test]
fn app_state_uses_one_request_for_selection_translation_and_save() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let state = build_app_state(
        Arc::new(SqliteStore::open_in_memory().unwrap()),
        platform_services(PlatformCapabilities::default(), calls.clone()),
    );
    let request_id = state.start_capture_request();

    let prepared = block_on(state.prepare_selection_for(request_id)).unwrap();
    let translated = block_on(state.translate_capture(
        request_id,
        &prepared.candidate.selected_text,
        "en",
        "de",
    ))
    .unwrap();
    let saved = state.save_capture(request_id, false).unwrap();

    assert_eq!(prepared.request_id, request_id);
    assert_eq!(translated.translated_text, "portable translation");
    assert_eq!(saved.translation.as_deref(), Some("portable translation"));
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        ["selection", "translation:portable selection:en:de"]
    );
}

fn platform_services(
    capabilities: PlatformCapabilities,
    calls: Arc<Mutex<Vec<String>>>,
) -> PlatformServices {
    PlatformServices {
        capabilities,
        selection: Arc::new(FakeSelection {
            calls: calls.clone(),
        }),
        ocr: Arc::new(FakeOcr {
            calls: calls.clone(),
        }),
        translation: Arc::new(FakeTranslation {
            calls: calls.clone(),
        }),
        permissions: Arc::new(FakePermissions {
            calls: calls.clone(),
        }),
        window: Arc::new(FakeWindow { calls }),
    }
}

struct FakeSelection {
    calls: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl SelectionProvider for FakeSelection {
    async fn capture_selection(&self) -> Result<CaptureCandidate, PlatformError> {
        self.calls.lock().unwrap().push("selection".into());
        Ok(CaptureCandidate {
            selected_text: "portable selection".into(),
            sentence: "A portable selection in context.".into(),
            source_app: Some("Contract Reader".into()),
            source_title: None,
            source_url: None,
            selection_bounds: None,
            origin: CaptureOrigin::Accessibility,
        })
    }
}

struct FakeOcr {
    calls: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl OcrProvider for FakeOcr {
    async fn recognize_near(
        &self,
        pointer: ScreenPoint,
    ) -> Result<Vec<OcrCandidate>, PlatformError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("ocr:{},{}", pointer.x, pointer.y));
        Ok(vec![OcrCandidate {
            text: "portable OCR".into(),
            bounds: ScreenRect::new(20.0, 70.0, 50.0, 18.0),
            confidence: 0.9,
        }])
    }
}

struct FakeTranslation {
    calls: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl TranslationProvider for FakeTranslation {
    async fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("translation:{text}:{source}:{target}"));
        Ok(TranslationResult {
            translated_text: "portable translation".into(),
            source_language: source.into(),
            target_language: target.into(),
        })
    }
}

struct FakePermissions {
    calls: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl PermissionProvider for FakePermissions {
    async fn status(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("permission-status:{kind:?}"));
        Ok(PermissionStatus::Granted)
    }

    async fn request(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("permission-request:{kind:?}"));
        Ok(PermissionStatus::Granted)
    }
}

struct FakeWindow {
    calls: Arc<Mutex<Vec<String>>>,
}

impl WindowProvider for FakeWindow {
    fn configure_capture_window(&self) -> Result<(), PlatformError> {
        self.calls.lock().unwrap().push("window".into());
        Ok(())
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    struct ThreadWake(thread::Thread);

    impl Wake for ThreadWake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park(),
        }
    }
}
