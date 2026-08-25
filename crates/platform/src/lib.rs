#![forbid(unsafe_code)]

//! Platform-neutral contracts for capture, translation, notification, and future enrichment.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureCandidate {
    pub selected_text: String,
    pub sentence: String,
    pub source_app: Option<String>,
    pub source_title: Option<String>,
    pub source_url: Option<String>,
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
    async fn capture_screen_region(&self) -> Result<CaptureCandidate, PlatformError>;
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
