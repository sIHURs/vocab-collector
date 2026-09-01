use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
};

use vocab_platform_api::{
    Capability, PermissionKind, PlatformCapabilities, PlatformError, ScreenPoint,
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
fn unverified_native_capabilities_remain_disabled() {
    let services = WindowsPlatform::new();

    assert_eq!(services.capabilities, PlatformCapabilities::default());
}

#[test]
fn deferred_providers_return_capability_specific_unsupported_errors() {
    let services = WindowsPlatform::new();

    assert_eq!(
        block_on(services.ocr.recognize_near(ScreenPoint::new(12.0, 34.0))),
        Err(PlatformError::Unsupported(Capability::ScreenshotOcr))
    );
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
fn capture_window_style_is_tool_window_without_app_window_activation() {
    const WS_EX_APPWINDOW: u32 = 0x0004_0000;
    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const WS_EX_NOACTIVATE: u32 = 0x0800_0000;

    let style = capture_window_extended_style(WS_EX_APPWINDOW);

    assert_eq!(style & WS_EX_APPWINDOW, 0);
    assert_ne!(style & WS_EX_TOOLWINDOW, 0);
    assert_ne!(style & WS_EX_NOACTIVATE, 0);
}

#[test]
#[ignore = "requires a prepared, focused Notepad selection on a Windows 11 physical machine"]
fn physical_notepad_unicode_selection_matches_the_portable_contract() {
    let expected = std::env::var("VOCAB_UIA_EXPECTED")
        .expect("set VOCAB_UIA_EXPECTED to the exact selected Notepad text");
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
            .is_some_and(|name| name.eq_ignore_ascii_case("notepad.exe"))
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
