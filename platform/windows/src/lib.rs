#![deny(unsafe_op_in_unsafe_fn)]

//! Windows adapter boundary for native providers implemented during Plan B.

use std::sync::Arc;

use async_trait::async_trait;
use vocab_platform_api::{
    Capability, PermissionKind, PermissionProvider, PermissionStatus, PlatformCapabilities,
    PlatformError, PlatformServices, TranslationProvider, TranslationResult, WindowProvider,
};

mod ocr;
mod selection;
pub mod window;

/// Builds the Windows provider bundle with only physically verified capabilities enabled.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindowsPlatform;

impl WindowsPlatform {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> PlatformServices {
        Self::services(Arc::new(UnsupportedTranslationProvider), false)
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn with_translation(translation: Arc<dyn TranslationProvider>) -> PlatformServices {
        Self::services(translation, true)
    }

    fn services(
        translation: Arc<dyn TranslationProvider>,
        translation_enabled: bool,
    ) -> PlatformServices {
        PlatformServices {
            capabilities: PlatformCapabilities {
                selection_capture: true,
                selection_bounds: true,
                translation: translation_enabled,
                ..PlatformCapabilities::default()
            },
            selection: Arc::new(selection::WindowsSelectionProvider),
            ocr: Arc::new(ocr::WindowsOcrProvider),
            translation,
            permissions: Arc::new(UnsupportedPermissionProvider),
            window: Arc::new(UnsupportedWindowProvider),
        }
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
