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
        pointer: ScreenPoint,
    ) -> Result<Vec<OcrCandidate>, PlatformError> {
        let candidate = ffi::capture_ocr_at(pointer)?;
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
        let text = text.to_owned();
        let source = source.to_owned();
        let target = target.to_owned();
        run_blocking(move || ffi::translate(&text, &source, &target)).await
    }
}

async fn run_blocking<T>(
    operation: impl FnOnce() -> Result<T, PlatformError> + Send + 'static,
) -> Result<T, PlatformError>
where
    T: Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| PlatformError::Operation("native translation worker failed".into()))?
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

#[cfg(test)]
mod tests {
    use std::thread;

    use vocab_platform_api::PlatformError;

    use super::run_blocking;

    #[test]
    fn blocking_translation_work_runs_off_the_async_caller_thread() {
        let caller = thread::current().id();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        let worker = runtime
            .block_on(run_blocking(move || Ok(thread::current().id())))
            .unwrap();

        assert_ne!(worker, caller);
    }

    #[test]
    fn blocking_translation_preserves_native_operation_errors() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        let error = runtime
            .block_on(run_blocking(|| {
                Err::<(), _>(PlatformError::Operation(
                    "native translation timed out".into(),
                ))
            }))
            .unwrap_err();

        assert_eq!(
            error,
            PlatformError::Operation("native translation timed out".into())
        );
    }

    #[test]
    fn blocking_translation_maps_worker_failures_to_a_static_operation_error() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        let error = runtime
            .block_on(run_blocking(|| -> Result<(), PlatformError> {
                panic!("worker failure must not become a bridge payload")
            }))
            .unwrap_err();

        assert_eq!(
            error,
            PlatformError::Operation("native translation worker failed".into())
        );
    }
}
