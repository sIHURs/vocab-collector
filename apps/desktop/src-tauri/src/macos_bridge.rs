use std::{
    ffi::{CStr, CString, c_char},
    ptr::NonNull,
};

use serde::Deserialize;
use vocab_platform::{
    CaptureCandidate, PermissionKind, PermissionStatus, PlatformError, TranslationResult,
};

unsafe extern "C" {
    fn vocab_mac_permission_status(kind: i32) -> *mut c_char;
    fn vocab_mac_request_accessibility() -> *mut c_char;
    fn vocab_mac_capture_selection() -> *mut c_char;
    fn vocab_mac_request_screen_recording() -> *mut c_char;
    fn vocab_mac_capture_ocr() -> *mut c_char;
    fn vocab_mac_translate(
        text: *const c_char,
        source: *const c_char,
        target: *const c_char,
    ) -> *mut c_char;
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

fn decode<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, PlatformError> {
    let response: BridgeResponse<T> = serde_json::from_str(json)
        .map_err(|_| PlatformError::Operation("native bridge returned invalid data".into()))?;
    if response.ok {
        response
            .payload
            .ok_or_else(|| PlatformError::Operation("native bridge omitted its result".into()))
    } else {
        Err(PlatformError::Operation(
            response
                .error
                .unwrap_or_else(|| "native operation failed".into()),
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_selection_without_exposing_native_envelope() {
        let candidate: CaptureCandidate = decode(r#"{"ok":true,"payload":{"selectedText":"serendipity","sentence":"A moment of serendipity.","sourceApp":"Safari","sourceTitle":null,"sourceUrl":null,"selectionBounds":{"x":10.0,"y":20.0,"width":80.0,"height":18.0},"origin":"accessibility"},"error":null}"#).unwrap();
        assert_eq!(candidate.selected_text, "serendipity");
        assert_eq!(candidate.sentence, "A moment of serendipity.");
    }

    #[test]
    fn maps_native_failure_to_a_content_free_error() {
        let error =
            decode::<CaptureCandidate>(r#"{"ok":false,"payload":null,"error":"noSelection"}"#)
                .unwrap_err();
        assert_eq!(error.to_string(), "platform operation failed: noSelection");
    }
}
