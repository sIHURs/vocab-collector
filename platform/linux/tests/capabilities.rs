use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
};

use vocab_platform_api::{
    Capability, PermissionKind, PlatformCapabilities, PlatformError, ScreenPoint,
};
use vocab_platform_linux::LinuxPlatform;

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
fn skeleton_reports_no_verified_native_capabilities() {
    let services = LinuxPlatform::new();

    assert_eq!(services.capabilities, PlatformCapabilities::default());
}

#[test]
fn skeleton_providers_return_capability_specific_unsupported_errors() {
    let services = LinuxPlatform::new();

    assert_eq!(
        block_on(services.selection.capture_selection()),
        Err(PlatformError::Unsupported(Capability::Selection))
    );
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
