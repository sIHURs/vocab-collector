use std::{
    future::Future,
    sync::{
        Arc, Condvar, Mutex,
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
    ScreenPoint, SelectionProvider, TranslationProvider, TranslationResult, WindowProvider,
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

struct RetryTranslationProvider {
    calls: Arc<AtomicUsize>,
}

struct ControlledSelectionProvider {
    calls: AtomicUsize,
    first_started: Arc<(Mutex<bool>, Condvar)>,
    release_first: Arc<(Mutex<bool>, Condvar)>,
}

#[async_trait::async_trait]
impl SelectionProvider for ControlledSelectionProvider {
    async fn capture_selection(&self) -> Result<CaptureCandidate, PlatformError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            let (started, signal) = &*self.first_started;
            *started.lock().unwrap() = true;
            signal.notify_one();

            let (released, signal) = &*self.release_first;
            let mut released = released.lock().unwrap();
            while !*released {
                released = signal.wait(released).unwrap();
            }
            Ok(candidate("first selection"))
        } else {
            Ok(candidate("second selection"))
        }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for RetryTranslationProvider {
    async fn translate(
        &self,
        _text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        let attempt = self.calls.fetch_add(1, Ordering::SeqCst);
        if attempt == 0 {
            return Err(PlatformError::Operation("first attempt failed".into()));
        }
        Ok(TranslationResult {
            translated_text: "glücklicher Zufall".into(),
            source_language: source.into(),
            target_language: target.into(),
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
    assert!(block_on(workflow.translate(prepared.request_id, selected_text, "en", "de")).is_err());
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
fn ocr_translation_does_not_call_the_provider_before_confirmation() {
    let calls = Arc::new(AtomicUsize::new(0));
    let translation: Arc<dyn TranslationProvider> = Arc::new(CountingTranslationProvider {
        calls: calls.clone(),
    });
    let (workflow, _) = workflow_with_translation(Ok(candidate("unused")), translation);
    let request = workflow.start_request();
    workflow
        .set_candidate(request, ocr_candidate("confirmed text"))
        .unwrap();

    assert!(matches!(
        block_on(workflow.translate(request, "confirmed text", "en", "de")),
        Err(PlatformCaptureError::Coordinator(
            CoordinatorError::InvalidTransition
        ))
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    workflow.confirm_ocr(request).unwrap();
    block_on(workflow.translate(request, "confirmed text", "en", "de")).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn failed_translation_can_retry_and_save_the_successful_translation() {
    let calls = Arc::new(AtomicUsize::new(0));
    let translation: Arc<dyn TranslationProvider> = Arc::new(RetryTranslationProvider {
        calls: calls.clone(),
    });
    let (workflow, application) =
        workflow_with_translation(Ok(candidate("serendipity")), translation);
    let request = workflow.start_request();
    workflow
        .set_candidate(request, candidate("serendipity"))
        .unwrap();

    assert!(matches!(
        block_on(workflow.translate(request, "serendipity", "en", "de")),
        Err(PlatformCaptureError::Platform(PlatformError::Operation(_)))
    ));
    let retried = block_on(workflow.translate(request, "serendipity", "en", "de")).unwrap();
    let saved = workflow.save(request, false, captured_at()).unwrap();

    assert_eq!(retried.translated_text, "glücklicher Zufall");
    assert_eq!(saved.translation.as_deref(), Some("glücklicher Zufall"));
    assert_eq!(application.list_words().unwrap().len(), 1);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn selection_preparation_returns_before_translation_is_invoked() {
    let calls = Arc::new(AtomicUsize::new(0));
    let translation: Arc<dyn TranslationProvider> = Arc::new(CountingTranslationProvider {
        calls: calls.clone(),
    });
    let (workflow, _) = workflow_with_translation(Ok(candidate("serendipity")), translation);

    let prepared = block_on(workflow.prepare_selection()).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let translated =
        block_on(workflow.translate(prepared.request_id, "serendipity", "en", "de")).unwrap();

    assert_eq!(translated.translated_text, "unused");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn overlapping_selection_completion_rejects_the_stale_real_request_id() {
    let first_started = Arc::new((Mutex::new(false), Condvar::new()));
    let release_first = Arc::new((Mutex::new(false), Condvar::new()));
    let selection: Arc<dyn SelectionProvider> = Arc::new(ControlledSelectionProvider {
        calls: AtomicUsize::new(0),
        first_started: first_started.clone(),
        release_first: release_first.clone(),
    });
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    let application = Arc::new(AppService::new(store, Uuid::now_v7()));
    let workflow = Arc::new(PlatformCaptureWorkflow::new(
        application,
        PlatformServices {
            capabilities: PlatformCapabilities::default(),
            selection,
            ocr: Arc::new(UnusedOcrProvider),
            translation: Arc::new(UnavailableTranslationProvider),
            permissions: Arc::new(UnusedPermissionProvider),
            window: Arc::new(UnusedWindowProvider),
        },
    ));

    let first_request = workflow.start_request();
    let first_workflow = workflow.clone();
    let first =
        thread::spawn(move || block_on(first_workflow.prepare_selection_for(first_request)));

    let (started, signal) = &*first_started;
    let mut started = started.lock().unwrap();
    while !*started {
        started = signal.wait(started).unwrap();
    }
    drop(started);

    let second_request = workflow.start_request();
    let second = block_on(workflow.prepare_selection_for(second_request)).unwrap();
    let (released, signal) = &*release_first;
    *released.lock().unwrap() = true;
    signal.notify_one();
    let stale = first.join().unwrap();

    assert_eq!(second.request_id, second_request);
    assert_eq!(second.candidate.selected_text, "second selection");
    assert!(matches!(
        stale,
        Err(PlatformCaptureError::Coordinator(
            CoordinatorError::StaleRequest
        ))
    ));
    assert_ne!(first_request, second_request);
}

#[test]
fn unavailable_translation_requires_the_real_coordinator_fallback() {
    let (workflow, application) = workflow(Ok(candidate("serendipity")));

    let prepared = block_on(workflow.prepare_selection()).unwrap();

    assert!(matches!(
        block_on(workflow.translate(prepared.request_id, "serendipity", "en", "de")),
        Err(PlatformCaptureError::Platform(PlatformError::Unsupported(
            Capability::Translation
        )))
    ));
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
    assert!(block_on(workflow.translate(prepared.request_id, "serendipity", "en", "de")).is_err());

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

#[test]
fn corrected_native_capture_and_manual_translation_use_the_shared_workflow() {
    let (workflow, application) = workflow(Ok(candidate("mispelled")));
    let prepared = block_on(workflow.prepare_selection()).unwrap();

    workflow
        .correct(
            prepared.request_id,
            "misspelled".into(),
            "This word was misspelled.".into(),
            Some("falsch geschrieben".into()),
        )
        .unwrap();
    let card = workflow
        .save(prepared.request_id, false, captured_at())
        .unwrap();
    let detail = application.get_word(card.word_id).unwrap();

    assert_eq!(card.display_form, "misspelled");
    assert_eq!(card.translation.as_deref(), Some("falsch geschrieben"));
    assert_eq!(detail.encounters[0].sentence, "This word was misspelled.");

    workflow
        .undo(prepared.request_id, card.encounter_id)
        .unwrap();
    assert!(
        application
            .get_word(card.word_id)
            .unwrap()
            .encounters
            .is_empty()
    );
}

#[test]
fn blank_manual_translation_still_requires_explicit_untranslated_save() {
    let (workflow, _) = workflow(Ok(candidate("serendipity")));
    let prepared = block_on(workflow.prepare_selection()).unwrap();
    workflow
        .correct(
            prepared.request_id,
            "serendipity".into(),
            "A sentence preserving serendipity exactly.".into(),
            Some("   ".into()),
        )
        .unwrap();

    assert!(matches!(
        workflow.save(prepared.request_id, false, captured_at()),
        Err(PlatformCaptureError::Coordinator(
            CoordinatorError::TranslationRequired
        ))
    ));
}

#[test]
fn operation_translation_failure_can_save_without_translation() {
    let calls = Arc::new(AtomicUsize::new(0));
    let translation: Arc<dyn TranslationProvider> = Arc::new(RetryTranslationProvider {
        calls: calls.clone(),
    });
    let (workflow, application) =
        workflow_with_translation(Ok(candidate("serendipity")), translation);
    let prepared = block_on(workflow.prepare_selection()).unwrap();

    assert!(matches!(
        block_on(workflow.translate(prepared.request_id, "serendipity", "en", "de")),
        Err(PlatformCaptureError::Platform(PlatformError::Operation(_)))
    ));
    let card = workflow
        .save(prepared.request_id, true, captured_at())
        .unwrap();

    assert_eq!(card.translation, None);
    assert_eq!(application.list_words().unwrap().len(), 1);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
