use serde::Serialize;
use uuid::Uuid;
use vocab_application::PlatformCaptureError;
use vocab_capture::CoordinatorError;
use vocab_platform_api::{Capability, CaptureCandidate, OcrCandidate, PlatformError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureFailureCode {
    PermissionRequired,
    PermissionDenied,
    EmptySelection,
    UnsupportedElement,
    TranslationUnavailable,
    TranslationFailed,
    Cancelled,
    Operation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CaptureFailure {
    pub code: CaptureFailureCode,
    pub message: String,
}

impl CaptureFailure {
    pub fn operation(message: impl Into<String>) -> Self {
        Self {
            code: CaptureFailureCode::Operation,
            message: message.into(),
        }
    }

    pub fn from_translation(error: PlatformCaptureError) -> Self {
        match error {
            PlatformCaptureError::Platform(PlatformError::Operation(message)) => Self {
                code: CaptureFailureCode::TranslationFailed,
                message: PlatformError::Operation(message).to_string(),
            },
            error => error.into(),
        }
    }
}

impl From<PlatformError> for CaptureFailure {
    fn from(error: PlatformError) -> Self {
        let code = match &error {
            PlatformError::PermissionRequired(_) => CaptureFailureCode::PermissionRequired,
            PlatformError::PermissionDenied(_) => CaptureFailureCode::PermissionDenied,
            PlatformError::EmptySelection => CaptureFailureCode::EmptySelection,
            PlatformError::UnsupportedElement => CaptureFailureCode::UnsupportedElement,
            PlatformError::Unsupported(Capability::Translation) => {
                CaptureFailureCode::TranslationUnavailable
            }
            PlatformError::Cancelled => CaptureFailureCode::Cancelled,
            PlatformError::InvalidSelectionRange
            | PlatformError::Unsupported(_)
            | PlatformError::Operation(_) => CaptureFailureCode::Operation,
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<CoordinatorError> for CaptureFailure {
    fn from(error: CoordinatorError) -> Self {
        Self::operation(error.to_string())
    }
}

impl From<PlatformCaptureError> for CaptureFailure {
    fn from(error: PlatformCaptureError) -> Self {
        match error {
            PlatformCaptureError::Platform(error) => error.into(),
            PlatformCaptureError::OcrConfirmationRequired
            | PlatformCaptureError::Coordinator(_)
            | PlatformCaptureError::Application(_) => Self::operation(error.to_string()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeCaptureEvent {
    pub request_id: Uuid,
    pub candidate: CaptureCandidate,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrCandidatesEvent {
    pub request_id: Uuid,
    pub candidates: Vec<OcrCandidate>,
    pub ambiguous: bool,
}

#[derive(Debug)]
pub struct NativeCaptureError {
    pub request_id: Uuid,
    pub failure: CaptureFailure,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeCaptureErrorEvent {
    pub request_id: Uuid,
    pub code: CaptureFailureCode,
    pub message: String,
}

impl From<NativeCaptureError> for NativeCaptureErrorEvent {
    fn from(error: NativeCaptureError) -> Self {
        Self {
            request_id: error.request_id,
            code: error.failure.code,
            message: error.failure.message,
        }
    }
}
