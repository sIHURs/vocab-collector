use std::{
    ffi::{CStr, CString, c_char},
    ptr::NonNull,
};

use serde::Deserialize;
use vocab_platform_api::{
    Capability, CaptureCandidate, PermissionKind, PermissionStatus, PlatformError,
    TranslationResult,
};

unsafe extern "C" {
    fn vocab_mac_permission_status(kind: i32) -> *mut c_char;
    fn vocab_mac_request_accessibility() -> *mut c_char;
    fn vocab_mac_capture_selection() -> *mut c_char;
    fn vocab_mac_request_screen_recording() -> *mut c_char;
    fn vocab_mac_capture_ocr() -> *mut c_char;
    fn vocab_mac_capture_ocr_at(x: f64, y: f64) -> *mut c_char;
    fn vocab_mac_translate(
        text: *const c_char,
        source: *const c_char,
        target: *const c_char,
    ) -> *mut c_char;
    fn vocab_mac_configure_capture_window() -> *mut c_char;
    fn vocab_mac_free_string(pointer: *mut c_char);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeResponse<T> {
    ok: bool,
    payload: Option<T>,
    error: Option<String>,
}

fn decode_owned<T: for<'de> Deserialize<'de>>(pointer: *mut c_char) -> Result<T, PlatformError> {
    let pointer = NonNull::new(pointer)
        .ok_or_else(|| PlatformError::Operation("native bridge returned no data".into()))?;
    let json = unsafe { CStr::from_ptr(pointer.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    unsafe { vocab_mac_free_string(pointer.as_ptr()) };
    decode(&json)
}

/// Decodes a JSON response from the native bridge without exposing its envelope to callers.
pub fn decode<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, PlatformError> {
    let response: BridgeResponse<T> = serde_json::from_str(json)
        .map_err(|_| PlatformError::Operation("native bridge returned invalid data".into()))?;
    if response.ok {
        response
            .payload
            .ok_or_else(|| PlatformError::Operation("native bridge omitted its result".into()))
    } else {
        Err(map_native_error(response.error.as_deref()))
    }
}

fn map_native_error(code: Option<&str>) -> PlatformError {
    match code {
        Some("noSelection") | Some("noTextFound") => PlatformError::EmptySelection,
        Some("noFocusedElement") => PlatformError::UnsupportedElement,
        Some("invalidSelectionRange") => PlatformError::InvalidSelectionRange,
        Some("accessibilityPermissionRequired") => {
            PlatformError::PermissionRequired(PermissionKind::Accessibility)
        }
        Some("screenRecordingPermissionRequired") => {
            PlatformError::PermissionRequired(PermissionKind::ScreenRecording)
        }
        Some("accessibilityPermissionDenied") => {
            PlatformError::PermissionDenied(PermissionKind::Accessibility)
        }
        Some("screenRecordingPermissionDenied") => {
            PlatformError::PermissionDenied(PermissionKind::ScreenRecording)
        }
        Some("cancelled") => PlatformError::Cancelled,
        Some("translationUnavailable") => PlatformError::Unsupported(Capability::Translation),
        Some("screenshotUnavailable") => {
            PlatformError::Operation("native screenshot is unavailable".into())
        }
        Some("ocrUnavailable") => PlatformError::Operation("native OCR is unavailable".into()),
        Some("translationTimedOut") => {
            PlatformError::Operation("native translation timed out".into())
        }
        _ => PlatformError::Operation("native bridge operation failed".into()),
    }
}

pub fn permission_status(kind: PermissionKind) -> Result<PermissionStatus, PlatformError> {
    let raw: String = decode_owned(unsafe {
        vocab_mac_permission_status(if kind == PermissionKind::Accessibility {
            0
        } else {
            1
        })
    })?;
    match raw.as_str() {
        "granted" => Ok(PermissionStatus::Granted),
        "denied" => Ok(PermissionStatus::Denied),
        "notDetermined" => Ok(PermissionStatus::NotDetermined),
        "restricted" => Ok(PermissionStatus::Restricted),
        _ => Err(PlatformError::Operation(
            "native bridge returned an unknown permission status".into(),
        )),
    }
}

pub fn request_accessibility() -> Result<PermissionStatus, PlatformError> {
    let _: String = decode_owned(unsafe { vocab_mac_request_accessibility() })?;
    permission_status(PermissionKind::Accessibility)
}

pub fn capture_selection() -> Result<CaptureCandidate, PlatformError> {
    decode_owned(unsafe { vocab_mac_capture_selection() })
}

pub fn request_screen_recording() -> Result<PermissionStatus, PlatformError> {
    let _: String = decode_owned(unsafe { vocab_mac_request_screen_recording() })?;
    permission_status(PermissionKind::ScreenRecording)
}

pub fn capture_ocr() -> Result<CaptureCandidate, PlatformError> {
    decode_owned(unsafe { vocab_mac_capture_ocr() })
}

pub fn capture_ocr_at(
    pointer: vocab_platform_api::ScreenPoint,
) -> Result<CaptureCandidate, PlatformError> {
    forward_ocr_coordinates(pointer, |x, y| {
        decode_owned(unsafe { vocab_mac_capture_ocr_at(x, y) })
    })
}

fn forward_ocr_coordinates<T>(
    pointer: vocab_platform_api::ScreenPoint,
    invoke: impl FnOnce(f64, f64) -> T,
) -> T {
    invoke(pointer.x, pointer.y)
}

pub fn translate(
    text: &str,
    source: &str,
    target: &str,
) -> Result<TranslationResult, PlatformError> {
    let text = CString::new(text)
        .map_err(|_| PlatformError::Operation("text contains an invalid null byte".into()))?;
    let source = CString::new(source)
        .map_err(|_| PlatformError::Operation("source language is invalid".into()))?;
    let target = CString::new(target)
        .map_err(|_| PlatformError::Operation("target language is invalid".into()))?;
    decode_owned(unsafe { vocab_mac_translate(text.as_ptr(), source.as_ptr(), target.as_ptr()) })
}

pub(crate) fn configure_capture_window() -> Result<(), PlatformError> {
    let _: String = decode_owned(unsafe { vocab_mac_configure_capture_window() })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use vocab_platform_api::ScreenPoint;

    use super::forward_ocr_coordinates;

    #[test]
    fn forwards_requested_ocr_coordinates_without_sampling_new_pointer_data() {
        let point = ScreenPoint::new(137.25, -48.5);
        let forwarded = forward_ocr_coordinates(point, |x, y| (x, y));

        assert_eq!(forwarded, (137.25, -48.5));
    }
}
