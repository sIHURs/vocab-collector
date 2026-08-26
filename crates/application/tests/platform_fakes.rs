use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    thread,
};

use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;
use vocab_application::{AppService, PlatformCaptureError, PlatformCaptureWorkflow};
use vocab_capture::CoordinatorError;
use vocab_platform_api::{
    Capability, CaptureCandidate, CaptureOrigin, OcrCandidate, OcrProvider, PermissionKind,
    PermissionProvider, PermissionStatus, PlatformCapabilities, PlatformError, PlatformServices,
    ScreenPoint, TranslationProvider, TranslationResult, WindowProvider,
};
use vocab_platform_contract_tests::{FakeSelectionProvider, UnavailableTranslationProvider};
use vocab_storage::SqliteStore;

fn candidate(selected_text: &str) -> CaptureCandidate {
    CaptureCandidate {
        selected_text: selected_text.into(),
        sentence: format!("A sentence preserving {selected_text} exactly."),
        source_app: Some("Fixture Reader".into()),
        source_title: Some("Unicode fixture".into()),
        source_url: Some("https://example.invalid/private-fixture".into()),
        selection_bounds: None,
        origin: CaptureOrigin::Accessibility,
    }
}

fn ocr_candidate(selected_text: &str) -> CaptureCandidate {
    CaptureCandidate {
        origin: CaptureOrigin::Ocr,
        ..candidate(selected_text)
    }
}

fn captured_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap()
}

fn workflow(
    selection: Result<CaptureCandidate, PlatformError>,
) -> (PlatformCaptureWorkflow, Arc<AppService>) {
    workflow_with_translation(selection, Arc::new(UnavailableTranslationProvider))
}

fn workflow_with_translation(
    selection: Result<CaptureCandidate, PlatformError>,
    translation: Arc<dyn TranslationProvider>,
) -> (PlatformCaptureWorkflow, Arc<AppService>) {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let application = Arc::new(AppService::new(store, Uuid::now_v7()));
    (
        PlatformCaptureWorkflow::new(
            application.clone(),
            platform_services(selection, translation),
        ),
        application,
    )
}

fn platform_services(
    selection: Result<CaptureCandidate, PlatformError>,
    translation: Arc<dyn TranslationProvider>,
) -> PlatformServices {
    PlatformServices {
        capabilities: PlatformCapabilities::default(),
        selection: Arc::new(FakeSelectionProvider::new(selection)),
        ocr: Arc::new(UnusedOcrProvider),
        translation,
        permissions: Arc::new(UnusedPermissionProvider),
        window: Arc::new(UnusedWindowProvider),
    }
}

struct UnusedOcrProvider;

#[async_trait::async_trait]
impl OcrProvider for UnusedOcrProvider {
    async fn recognize_near(
        &self,
        _pointer: ScreenPoint,
    ) -> Result<Vec<OcrCandidate>, PlatformError> {
        Err(PlatformError::Unsupported(Capability::ScreenshotOcr))
    }
}

struct UnusedPermissionProvider;

#[async_trait::async_trait]
impl PermissionProvider for UnusedPermissionProvider {
    async fn status(&self, _kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
        Err(PlatformError::Unsupported(Capability::Selection))
    }

    async fn request(&self, _kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
        Err(PlatformError::Unsupported(Capability::Selection))
    }
}

struct UnusedWindowProvider;

impl WindowProvider for UnusedWindowProvider {
    fn configure_capture_window(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(Capability::NonActivatingWindow))
    }
}

struct CountingTranslationProvider {
    calls: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl TranslationProvider for CountingTranslationProvider {
    async fn translate(
        &self,
        _text: &str,
        _source: &str,
        _target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(TranslationResult {
            translated_text: "unused".into(),
            source_language: "en".into(),
            target_language: "de".into(),
        })
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

#[test]
fn exact_unicode_surface_form_is_preserved_through_the_production_workflow() {
    let selected_text = "Straße—CAFÉ 👩🏽‍💻";
    let (workflow, application) = workflow(Ok(candidate(selected_text)));

    let prepared = block_on(workflow.prepare_selection()).unwrap();
    let card = workflow
        .save(prepared.request_id, true, captured_at())
        .unwrap();
    let detail = application.get_word(card.word_id).unwrap();

    assert_eq!(prepared.candidate.selected_text, selected_text);
    assert_eq!(card.display_form, selected_text);
    assert_eq!(detail.encounters[0].selected_text, selected_text);
}

#[test]
fn empty_selection_returns_the_typed_error_without_saving() {
    let (workflow, application) = workflow(Err(PlatformError::EmptySelection));

    let result = block_on(workflow.prepare_selection());

    assert!(matches!(
        result,
        Err(PlatformCaptureError::Platform(
            PlatformError::EmptySelection
        ))
    ));
    assert!(application.list_words().unwrap().is_empty());
}

#[test]
fn unsupported_selection_returns_the_typed_error_without_saving() {
    let (workflow, application) = workflow(Err(PlatformError::UnsupportedElement));

    let result = block_on(workflow.prepare_selection());

    assert!(matches!(
        result,
        Err(PlatformCaptureError::Platform(
            PlatformError::UnsupportedElement
        ))
    ));
    assert!(application.list_words().unwrap().is_empty());
}

#[test]
fn ocr_origin_requires_task_six_confirmation_without_translation_or_persistence() {
    let calls = Arc::new(AtomicUsize::new(0));
    let translation: Arc<dyn TranslationProvider> = Arc::new(CountingTranslationProvider {
        calls: calls.clone(),
    });
    let (workflow, application) =
        workflow_with_translation(Ok(ocr_candidate("serendipity")), translation);

    assert!(matches!(
        block_on(workflow.prepare_selection()),
        Err(PlatformCaptureError::OcrConfirmationRequired)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(application.list_words().unwrap().is_empty());
}

#[test]
fn unavailable_translation_requires_the_real_coordinator_fallback() {
    let (workflow, application) = workflow(Ok(candidate("serendipity")));

    let prepared = block_on(workflow.prepare_selection()).unwrap();

    assert_eq!(
        prepared.translation_error,
        Some(PlatformError::Unsupported(Capability::Translation))
    );
    assert!(matches!(
        workflow.save(prepared.request_id, false, captured_at()),
        Err(PlatformCaptureError::Coordinator(
            CoordinatorError::TranslationRequired
        ))
    ));
    assert!(application.list_words().unwrap().is_empty());
}

#[test]
fn explicit_save_without_translation_persists_an_untranslated_capture_once() {
    let (workflow, application) = workflow(Ok(candidate("serendipity")));
    let prepared = block_on(workflow.prepare_selection()).unwrap();

    let card = workflow
        .save(prepared.request_id, true, captured_at())
        .unwrap();

    assert_eq!(card.display_form, "serendipity");
    assert_eq!(card.translation, None);
    assert_eq!(application.list_words().unwrap().len(), 1);
    assert!(matches!(
        workflow.save(prepared.request_id, true, captured_at()),
        Err(PlatformCaptureError::Coordinator(
            CoordinatorError::AlreadySaved
        ))
    ));
}
