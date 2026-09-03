use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
};

use async_trait::async_trait;
use vocab_platform_api::{
    Capability, PermissionKind, PlatformCapabilities, PlatformError, TranslationProvider,
    TranslationResult,
};
use vocab_platform_windows::WindowsPlatform;
use vocab_platform_windows::window::capture_window_extended_style;

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
fn only_physically_verified_native_capabilities_are_enabled() {
    let services = WindowsPlatform::new();

    assert_eq!(
        services.capabilities,
        PlatformCapabilities {
            selection_capture: true,
            selection_bounds: true,
            ..PlatformCapabilities::default()
        }
    );
}

struct ConfiguredTranslation;

#[async_trait]
impl TranslationProvider for ConfiguredTranslation {
    async fn translate(
        &self,
        _text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        Ok(TranslationResult {
            translated_text: "configured".into(),
            source_language: source.into(),
            target_language: target.into(),
        })
    }
}

#[test]
fn injected_translation_provider_enables_only_translation_capability() {
    let services = WindowsPlatform::with_translation(Arc::new(ConfiguredTranslation));

    assert_eq!(
        services.capabilities,
        PlatformCapabilities {
            selection_capture: true,
            selection_bounds: true,
            translation: true,
            ..PlatformCapabilities::default()
        }
    );
    assert_eq!(
        block_on(services.translation.translate("word", "en", "de"))
            .unwrap()
            .translated_text,
        "configured"
    );
}

#[test]
fn deferred_providers_return_capability_specific_unsupported_errors() {
    let services = WindowsPlatform::new();

    assert_eq!(
        block_on(services.translation.translate("word", "en", "de")),
        Err(PlatformError::Unsupported(Capability::Translation))
    );
    assert_eq!(
        block_on(services.permissions.status(PermissionKind::Accessibility)),
        Err(PlatformError::Unsupported(Capability::Selection))
    );
    assert_eq!(
        block_on(services.permissions.request(PermissionKind::Accessibility)),
        Err(PlatformError::Unsupported(Capability::Selection))
    );
    assert_eq!(
        block_on(services.permissions.status(PermissionKind::ScreenRecording)),
        Err(PlatformError::Unsupported(Capability::ScreenshotOcr))
    );
    assert_eq!(
        block_on(
            services
                .permissions
                .request(PermissionKind::ScreenRecording)
        ),
        Err(PlatformError::Unsupported(Capability::ScreenshotOcr))
    );
    assert_eq!(
        services.window.configure_capture_window(),
        Err(PlatformError::Unsupported(Capability::NonActivatingWindow))
    );
}

#[test]
fn capture_window_style_is_tool_window_that_can_activate_after_passive_show() {
    const WS_EX_APPWINDOW: u32 = 0x0004_0000;
    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const WS_EX_NOACTIVATE: u32 = 0x0800_0000;

    let style = capture_window_extended_style(WS_EX_APPWINDOW);

    assert_eq!(style & WS_EX_APPWINDOW, 0);
    assert_ne!(style & WS_EX_TOOLWINDOW, 0);
    assert_eq!(style & WS_EX_NOACTIVATE, 0);
}

#[test]
#[ignore = "requires a prepared, focused application selection on the target Windows 10 Pro physical machine"]
fn physical_uia_selection_matches_the_portable_contract() {
    let expected = std::env::var("VOCAB_UIA_EXPECTED")
        .expect("set VOCAB_UIA_EXPECTED to the exact selected application text");
    let expected_app = std::env::var("VOCAB_UIA_EXPECTED_APP")
        .expect("set VOCAB_UIA_EXPECTED_APP to the selected application's executable name");
    std::thread::sleep(std::time::Duration::from_secs(2));

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build physical test runtime");
    let candidate = runtime
        .block_on(WindowsPlatform::new().selection.capture_selection())
        .unwrap();

    assert_eq!(candidate.selected_text, expected);
    assert!(candidate.sentence.contains(&candidate.selected_text));
    assert!(
        candidate
            .source_app
            .as_deref()
            .is_some_and(|name| name.eq_ignore_ascii_case(&expected_app))
    );
    assert!(
        candidate
            .source_title
            .as_deref()
            .is_some_and(|title| !title.is_empty())
    );
    assert!(
        candidate
            .selection_bounds
            .is_some_and(|bounds| bounds.is_available())
    );
}
