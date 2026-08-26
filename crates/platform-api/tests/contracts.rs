use vocab_platform_api::{Capability, PlatformCapabilities, PlatformError};

#[test]
fn unavailable_capabilities_are_explicit() {
    let value = PlatformCapabilities::default();
    assert!(!value.selection_capture);
    assert!(!value.selection_bounds);
    assert!(!value.screenshot_ocr);
    assert!(!value.translation);
    assert!(!value.non_activating_window);
}

#[test]
fn unsupported_error_is_content_free() {
    assert_eq!(
        PlatformError::Unsupported(Capability::Selection).to_string(),
        "unsupported platform capability: selection"
    );
}
