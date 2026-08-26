use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
    thread,
};

use async_trait::async_trait;
use vocab_desktop_lib::{
    bootstrap::build_app_state,
    commands::capture::{
        capture_selected_text, capture_with_ocr, confirm_ocr, get_permission_status,
        get_platform_capabilities, hide_capture_window, request_accessibility_permission,
        request_screen_recording_permission, save_native_capture, translate_text,
    },
};
use vocab_domain::CaptureOrigin;
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
    let _ = hide_capture_window;
    let _ = get_platform_capabilities;
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
        permissions: Arc::new(FakePermissions { calls }),
        window: Arc::new(FakeWindow),
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

struct FakeWindow;

impl WindowProvider for FakeWindow {
    fn configure_capture_window(&self) -> Result<(), PlatformError> {
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
