use std::{
    fmt,
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
};

use chrono::{TimeZone, Utc};
use uuid::Uuid;
use vocab_application::{AppService, CaptureRequest};
use vocab_platform_api::{
    Capability, CaptureCandidate, CaptureOrigin, PlatformError, SelectionProvider,
    TranslationProvider,
};
use vocab_platform_contract_tests::{FakeSelectionProvider, UnavailableTranslationProvider};
use vocab_storage::SqliteStore;

struct TestProviderBundle {
    selection: Arc<dyn SelectionProvider>,
    translation: Arc<dyn TranslationProvider>,
}

#[derive(Debug)]
enum TestCaptureError {
    Platform(PlatformError),
    Application(vocab_application::ApplicationError),
}

impl fmt::Display for TestCaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Platform(error) => error.fmt(formatter),
            Self::Application(error) => error.fmt(formatter),
        }
    }
}

impl From<PlatformError> for TestCaptureError {
    fn from(error: PlatformError) -> Self {
        Self::Platform(error)
    }
}

impl From<vocab_application::ApplicationError> for TestCaptureError {
    fn from(error: vocab_application::ApplicationError) -> Self {
        Self::Application(error)
    }
}

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

async fn capture_with_providers(
    service: &AppService,
    providers: &TestProviderBundle,
    save_without_translation: bool,
) -> Result<vocab_domain::CaptureCard, TestCaptureError> {
    let candidate = providers.selection.capture_selection().await?;
    let translation = match providers
        .translation
        .translate(&candidate.selected_text, "en", "de")
        .await
    {
        Ok(value) => Some(value),
        Err(PlatformError::Unsupported(Capability::Translation)) if save_without_translation => {
            None
        }
        Err(error) => return Err(error.into()),
    };

    Ok(service.capture(CaptureRequest {
        selected_text: candidate.selected_text,
        lemma: None,
        sentence: candidate.sentence,
        source_language: translation
            .as_ref()
            .map_or_else(|| "en".into(), |value| value.source_language.clone()),
        target_language: translation
            .as_ref()
            .map_or_else(|| "de".into(), |value| value.target_language.clone()),
        translation: translation.map(|value| value.translated_text),
        part_of_speech: None,
        source_app: candidate.source_app,
        source_title: candidate.source_title,
        source_url: candidate.source_url,
        capture_origin: candidate.origin,
        captured_at: Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap(),
    })?)
}

fn providers(selection: Result<CaptureCandidate, PlatformError>) -> TestProviderBundle {
    TestProviderBundle {
        selection: Arc::new(FakeSelectionProvider::new(selection)),
        translation: Arc::new(UnavailableTranslationProvider),
    }
}

fn service() -> (AppService, Arc<SqliteStore>) {
    let store = Arc::new(SqliteStore::open_in_memory().unwrap());
    (AppService::new(store.clone(), Uuid::now_v7()), store)
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
fn exact_unicode_surface_form_is_preserved_through_application_storage() {
    let selected_text = "Straße—CAFÉ 👩🏽‍💻";
    let (service, _) = service();

    let card = block_on(capture_with_providers(
        &service,
        &providers(Ok(candidate(selected_text))),
        true,
    ))
    .unwrap();
    let detail = service.get_word(card.word_id).unwrap();

    assert_eq!(card.display_form, selected_text);
    assert_eq!(detail.encounters[0].selected_text, selected_text);
}

#[test]
fn empty_selection_remains_a_typed_platform_error_and_saves_nothing() {
    let (service, _) = service();

    let result = block_on(capture_with_providers(
        &service,
        &providers(Err(PlatformError::EmptySelection)),
        false,
    ));

    assert!(matches!(
        result,
        Err(TestCaptureError::Platform(PlatformError::EmptySelection))
    ));
    assert!(service.list_words().unwrap().is_empty());
}

#[test]
fn unsupported_selection_remains_a_typed_platform_error_and_saves_nothing() {
    let (service, _) = service();

    let result = block_on(capture_with_providers(
        &service,
        &providers(Err(PlatformError::UnsupportedElement)),
        false,
    ));

    assert!(matches!(
        result,
        Err(TestCaptureError::Platform(
            PlatformError::UnsupportedElement
        ))
    ));
    assert!(service.list_words().unwrap().is_empty());
}

#[test]
fn unavailable_translation_remains_a_typed_platform_error() {
    let (service, _) = service();

    let result = block_on(capture_with_providers(
        &service,
        &providers(Ok(candidate("serendipity"))),
        false,
    ));

    assert!(matches!(
        result,
        Err(TestCaptureError::Platform(PlatformError::Unsupported(
            Capability::Translation
        )))
    ));
    assert!(service.list_words().unwrap().is_empty());
}

#[test]
fn explicit_save_without_translation_persists_an_untranslated_capture() {
    let (service, _) = service();

    let card = block_on(capture_with_providers(
        &service,
        &providers(Ok(candidate("serendipity"))),
        true,
    ))
    .unwrap();

    assert_eq!(card.display_form, "serendipity");
    assert_eq!(card.translation, None);
    assert_eq!(service.list_words().unwrap().len(), 1);
}
