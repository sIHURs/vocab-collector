use std::sync::Arc;

use async_trait::async_trait;
use vocab_platform_api::{
    CaptureCandidate, OcrCandidate, OcrProvider, PermissionKind, PermissionProvider,
    PermissionStatus, PlatformCapabilities, PlatformError, PlatformServices, ScreenPoint,
    SelectionProvider, TranslationProvider, TranslationResult, WindowProvider,
};

use crate::ffi;

/// The macOS implementation of the portable platform service bundle.
#[derive(Clone, Copy, Debug, Default)]
pub struct MacPlatform;

impl MacPlatform {
    #[allow(clippy::new_ret_no_self)]
    /// Creates the portable service bundle backed by the macOS native bridge.
    pub fn new() -> Result<PlatformServices, PlatformError> {
        Ok(PlatformServices {
            capabilities: PlatformCapabilities {
                selection_capture: true,
                selection_bounds: true,
                screenshot_ocr: true,
                translation: true,
                non_activating_window: true,
            },
            selection: Arc::new(MacSelectionProvider),
            ocr: Arc::new(MacOcrProvider),
            translation: Arc::new(MacTranslationProvider),
            permissions: Arc::new(MacPermissionProvider),
            window: Arc::new(MacWindowProvider),
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MacSelectionProvider;

#[async_trait]
impl SelectionProvider for MacSelectionProvider {
    async fn capture_selection(&self) -> Result<CaptureCandidate, PlatformError> {
        ffi::capture_selection()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MacOcrProvider;

#[async_trait]
impl OcrProvider for MacOcrProvider {
    async fn recognize_near(
        &self,
        _pointer: ScreenPoint,
    ) -> Result<Vec<OcrCandidate>, PlatformError> {
        let candidate = ffi::capture_ocr()?;
        Ok(vec![OcrCandidate {
            text: candidate.selected_text,
            bounds: candidate.selection_bounds.unwrap_or_default(),
            confidence: 1.0,
        }])
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MacTranslationProvider;

#[async_trait]
impl TranslationProvider for MacTranslationProvider {
    async fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        ffi::translate(text, source, target)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MacPermissionProvider;

#[async_trait]
impl PermissionProvider for MacPermissionProvider {
    async fn status(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
        ffi::permission_status(kind)
    }

    async fn request(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
        match kind {
            PermissionKind::Accessibility => ffi::request_accessibility(),
            PermissionKind::ScreenRecording => ffi::request_screen_recording(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MacWindowProvider;

impl WindowProvider for MacWindowProvider {
    fn configure_capture_window(&self) -> Result<(), PlatformError> {
        ffi::configure_capture_window()
    }
}
