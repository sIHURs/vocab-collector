use serde::Serialize;
use uuid::Uuid;
use vocab_platform_api::CaptureCandidate;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeCaptureEvent {
    pub request_id: Uuid,
    pub candidate: CaptureCandidate,
}

#[derive(Debug)]
pub struct NativeCaptureError {
    pub request_id: Uuid,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeCaptureErrorEvent {
    pub request_id: Uuid,
    pub message: String,
}

impl From<NativeCaptureError> for NativeCaptureErrorEvent {
    fn from(error: NativeCaptureError) -> Self {
        Self {
            request_id: error.request_id,
            message: error.message,
        }
    }
}
