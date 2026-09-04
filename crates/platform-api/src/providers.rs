use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    CaptureCandidate, EnrichmentResult, OcrCandidate, PermissionKind, PermissionStatus,
    PlatformCapabilities, PlatformError, ScreenRect, TranslationResult,
};

#[async_trait]
pub trait SelectionProvider: Send + Sync {
    async fn capture_selection(&self) -> Result<CaptureCandidate, PlatformError>;
}

#[async_trait]
pub trait OcrProvider: Send + Sync {
    async fn recognize_region(
        &self,
        region: ScreenRect,
    ) -> Result<Vec<OcrCandidate>, PlatformError>;
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError>;
}

#[async_trait]
pub trait PermissionProvider: Send + Sync {
    async fn status(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError>;
    async fn request(&self, kind: PermissionKind) -> Result<PermissionStatus, PlatformError>;
}

pub trait WindowProvider: Send + Sync {
    fn configure_capture_window(&self) -> Result<(), PlatformError>;
}

pub struct PlatformServices {
    pub capabilities: PlatformCapabilities,
    pub selection: Arc<dyn SelectionProvider>,
    pub ocr: Arc<dyn OcrProvider>,
    pub translation: Arc<dyn TranslationProvider>,
    pub permissions: Arc<dyn PermissionProvider>,
    pub window: Arc<dyn WindowProvider>,
}

#[async_trait]
pub trait EnrichmentProvider: Send + Sync {
    async fn enrich(
        &self,
        word: &str,
        context: &str,
    ) -> Result<Option<EnrichmentResult>, PlatformError>;
}

pub struct NoopEnrichmentProvider;

#[async_trait]
impl EnrichmentProvider for NoopEnrichmentProvider {
    async fn enrich(
        &self,
        _word: &str,
        _context: &str,
    ) -> Result<Option<EnrichmentResult>, PlatformError> {
        Ok(None)
    }
}
