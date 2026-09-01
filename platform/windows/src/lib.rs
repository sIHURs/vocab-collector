#![deny(unsafe_op_in_unsafe_fn)]

//! Static Windows adapter boundary prepared for Plan B implementation on Windows 11.

use std::sync::Arc;

use async_trait::async_trait;
use vocab_platform_api::{
    Capability, OcrCandidate, OcrProvider, PermissionKind, PermissionProvider, PermissionStatus,
    PlatformCapabilities, PlatformError, PlatformServices, ScreenPoint, TranslationProvider,
    TranslationResult, WindowProvider,
};

mod selection;

/// Builds the Windows provider bundle. Native providers are intentionally deferred to Plan B.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindowsPlatform;

impl WindowsPlatform {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> PlatformServices {
        PlatformServices {
            capabilities: PlatformCapabilities::default(),
            selection: Arc::new(selection::WindowsSelectionProvider),
            ocr: Arc::new(UnsupportedOcrProvider),
            translation: Arc::new(UnsupportedTranslationProvider),
            permissions: Arc::new(UnsupportedPermissionProvider),
            window: Arc::new(UnsupportedWindowProvider),
        }
    }
}

struct UnsupportedOcrProvider;

#[async_trait]
impl OcrProvider for UnsupportedOcrProvider {
    async fn recognize_near(
        &self,
        _pointer: ScreenPoint,
    ) -> Result<Vec<OcrCandidate>, PlatformError> {
        Err(PlatformError::Unsupported(Capability::ScreenshotOcr))
    }
}

struct UnsupportedTranslationProvider;

#[async_trait]
impl TranslationProvider for UnsupportedTranslationProvider {
    async fn translate(
        &self,
        _text: &str,
        _source: &str,
        _target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        Err(PlatformError::Unsupported(Capability::Translation))
    }
}

struct UnsupportedPermissionProvider;

#[async_trait]
impl PermissionProvider for UnsupportedPermissionProvider {
    async fn status(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
        Err(PlatformError::Unsupported(permission_capability(kind)))
    }

    async fn request(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
        Err(PlatformError::Unsupported(permission_capability(kind)))
    }
}

const fn permission_capability(kind: PermissionKind) -> Capability {
    match kind {
        PermissionKind::Accessibility => Capability::Selection,
        PermissionKind::ScreenRecording => Capability::ScreenshotOcr,
    }
}

struct UnsupportedWindowProvider;

impl WindowProvider for UnsupportedWindowProvider {
    fn configure_capture_window(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(Capability::NonActivatingWindow))
    }
}
