use crate::{Capability, PermissionKind};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlatformError {
    #[error("platform permission is required: {0:?}")]
    PermissionRequired(PermissionKind),
    #[error("platform permission was denied: {0:?}")]
    PermissionDenied(PermissionKind),
    #[error("selection is empty")]
    EmptySelection,
    #[error("selected element is unsupported")]
    UnsupportedElement,
    #[error("selection range is invalid")]
    InvalidSelectionRange,
    #[error("unsupported platform capability: {0}")]
    Unsupported(Capability),
    #[error("platform operation was cancelled")]
    Cancelled,
    #[error("platform operation failed: {0}")]
    Operation(String),
}
