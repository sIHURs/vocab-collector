#![forbid(unsafe_code)]

//! Platform-neutral contracts for capture, translation, notification, and future enrichment.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

impl ScreenPoint {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenSize {
    pub width: f64,
    pub height: f64,
}

impl ScreenSize {
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ScreenRect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn is_available(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
    pub fn center(self) -> ScreenPoint {
        ScreenPoint::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorWorkArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

impl MonitorWorkArea {
    pub const fn new(x: f64, y: f64, width: f64, height: f64, scale_factor: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            scale_factor,
        }
    }

    pub fn contains(self, point: ScreenPoint) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureOrigin {
    Manual,
    Accessibility,
    Ocr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionKind {
    Accessibility,
    ScreenRecording,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureCandidate {
    pub selected_text: String,
    pub sentence: String,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub source_url: Option<String>,
    pub selection_bounds: Option<ScreenRect>,
    pub origin: CaptureOrigin,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrCandidate {
    pub text: String,
    pub bounds: ScreenRect,
    pub confidence: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResult {
    pub translated_text: String,
    pub source_language: String,
    pub target_language: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichmentResult {
    pub explanation: String,
    pub provider: String,
    pub model: String,
    pub schema_version: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("permission is required: {0}")]
    PermissionRequired(String),
    #[error("feature is unavailable: {0}")]
    Unavailable(String),
    #[error("platform operation failed: {0}")]
    Operation(String),
}

#[async_trait]
pub trait CaptureProvider: Send + Sync {
    async fn capture_selected_text(&self) -> Result<CaptureCandidate, PlatformError>;
}

#[async_trait]
pub trait OcrProvider: Send + Sync {
    async fn capture_screen_region(&self) -> Result<Vec<OcrCandidate>, PlatformError>;
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(
        &self,
        text: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<TranslationResult, PlatformError>;
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
