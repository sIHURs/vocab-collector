//! Safe macOS implementations of the portable platform provider contracts.

pub mod ffi;
mod providers;

pub use ffi::{
    capture_ocr, capture_ocr_at, capture_selection, permission_status, request_accessibility,
    request_screen_recording, translate,
};
pub use providers::{
    MacOcrProvider, MacPermissionProvider, MacPlatform, MacSelectionProvider,
    MacTranslationProvider, MacWindowProvider,
};
