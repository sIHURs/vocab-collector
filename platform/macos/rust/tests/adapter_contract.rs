use vocab_platform_api::{CaptureCandidate, PermissionKind, PlatformError};
use vocab_platform_contract_tests::assert_error_is_content_free;
use vocab_platform_macos::{MacPlatform, ffi::decode};

const SELECTION_RESPONSE: &str = r#"{"ok":true,"payload":{"selectedText":"serendipity","sentence":"A moment of serendipity.","sourceApp":"Safari","sourceTitle":null,"sourceUrl":null,"selectionBounds":{"x":10.0,"y":20.0,"width":80.0,"height":18.0},"origin":"accessibility"},"error":null}"#;

#[test]
fn decodes_selection_without_exposing_native_envelope() {
    let candidate: CaptureCandidate = decode(SELECTION_RESPONSE).unwrap();

    assert_eq!(candidate.selected_text, "serendipity");
    assert_eq!(candidate.sentence, "A moment of serendipity.");
}

#[test]
fn maps_no_selection_to_a_typed_error() {
    let error = decode::<CaptureCandidate>(r#"{"ok":false,"payload":null,"error":"noSelection"}"#)
        .unwrap_err();

    assert_eq!(error, PlatformError::EmptySelection);
}

#[test]
fn maps_accessibility_permission_requirement_to_a_typed_error() {
    let error = decode::<CaptureCandidate>(
        r#"{"ok":false,"payload":null,"error":"accessibilityPermissionRequired"}"#,
    )
    .unwrap_err();

    assert_eq!(
        error,
        PlatformError::PermissionRequired(PermissionKind::Accessibility)
    );
}

#[test]
fn maps_screen_recording_permission_requirement_to_a_typed_error() {
    let error = decode::<CaptureCandidate>(
        r#"{"ok":false,"payload":null,"error":"screenRecordingPermissionRequired"}"#,
    )
    .unwrap_err();

    assert_eq!(
        error,
        PlatformError::PermissionRequired(PermissionKind::ScreenRecording)
    );
}

#[test]
fn native_errors_do_not_display_bridge_payloads() {
    let error = decode::<CaptureCandidate>(
        r#"{"ok":false,"payload":null,"error":"native failure: secret selection text"}"#,
    )
    .unwrap_err();

    assert_error_is_content_free(&error, &["native failure", "secret selection text"]);
}

#[test]
fn mac_platform_advertises_the_capabilities_backed_by_the_native_bridge() {
    let services = MacPlatform::new().unwrap();

    assert!(services.capabilities.selection_capture);
    assert!(services.capabilities.selection_bounds);
    assert!(services.capabilities.screenshot_ocr);
    assert!(services.capabilities.translation);
    assert!(services.capabilities.non_activating_window);
}
