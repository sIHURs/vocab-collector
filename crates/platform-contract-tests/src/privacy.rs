use vocab_platform_api::PlatformError;

/// Asserts that a rendered provider error does not disclose captured fixture content.
pub fn assert_error_is_content_free(error: &PlatformError, private_values: &[&str]) {
    assert_diagnostic_is_content_free(&error.to_string(), private_values);
}

/// Asserts that provider diagnostics expose no caller-supplied captured values.
pub fn assert_diagnostic_is_content_free(diagnostic: &str, private_values: &[&str]) {
    for value in private_values
        .iter()
        .copied()
        .filter(|value| !value.is_empty())
    {
        assert!(
            !diagnostic.contains(value),
            "platform diagnostic disclosed captured content"
        );
    }
}

#[cfg(test)]
mod tests {
    use vocab_platform_api::{Capability, PlatformError};

    use super::{assert_diagnostic_is_content_free, assert_error_is_content_free};

    #[test]
    fn metadata_and_lengths_are_safe_when_private_values_are_absent() {
        assert_diagnostic_is_content_free(
            "uia_selection pattern=TextPattern2 selected_utf16_length=18 rectangle_count=2 source_title_present=false",
            &["private selected text", "private reading context"],
        );
    }

    #[test]
    fn typed_unavailable_error_excludes_private_fixture_values() {
        assert_error_is_content_free(
            &PlatformError::Unsupported(Capability::Translation),
            &["Straße—CAFÉ 👩🏽‍💻", "private reading context"],
        );
    }

    #[test]
    #[should_panic(expected = "platform diagnostic disclosed captured content")]
    fn privacy_assertion_rejects_a_diagnostic_containing_fixture_content() {
        assert_error_is_content_free(
            &PlatformError::Operation("failed while reading private context".into()),
            &["private context"],
        );
    }
}
