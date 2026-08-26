use async_trait::async_trait;
use vocab_platform_api::{CaptureCandidate, PlatformError, SelectionProvider};

/// A deterministic selection provider for portable application tests.
pub struct FakeSelectionProvider {
    response: Result<CaptureCandidate, PlatformError>,
}

impl FakeSelectionProvider {
    pub const fn new(response: Result<CaptureCandidate, PlatformError>) -> Self {
        Self { response }
    }
}

#[async_trait]
impl SelectionProvider for FakeSelectionProvider {
    async fn capture_selection(&self) -> Result<CaptureCandidate, PlatformError> {
        clone_selection_response(&self.response)
    }
}

/// Verifies a selection provider solely through its public portable contract.
pub async fn assert_selection_contract<F, P>(
    provider_factory: F,
    expected: Result<CaptureCandidate, PlatformError>,
) where
    F: FnOnce() -> P,
    P: SelectionProvider,
{
    let actual = provider_factory().capture_selection().await;
    assert_eq!(actual, expected);
}

fn clone_selection_response(
    response: &Result<CaptureCandidate, PlatformError>,
) -> Result<CaptureCandidate, PlatformError> {
    match response {
        Ok(candidate) => Ok(candidate.clone()),
        Err(error) => Err(clone_platform_error(error)),
    }
}

fn clone_platform_error(error: &PlatformError) -> PlatformError {
    match error {
        PlatformError::PermissionRequired(kind) => PlatformError::PermissionRequired(*kind),
        PlatformError::PermissionDenied(kind) => PlatformError::PermissionDenied(*kind),
        PlatformError::EmptySelection => PlatformError::EmptySelection,
        PlatformError::UnsupportedElement => PlatformError::UnsupportedElement,
        PlatformError::InvalidSelectionRange => PlatformError::InvalidSelectionRange,
        PlatformError::Unsupported(capability) => PlatformError::Unsupported(*capability),
        PlatformError::Cancelled => PlatformError::Cancelled,
        PlatformError::Operation(message) => PlatformError::Operation(message.clone()),
    }
}

#[cfg(test)]
mod tests {
    use vocab_platform_api::{CaptureCandidate, CaptureOrigin, PlatformError};

    use super::{FakeSelectionProvider, assert_selection_contract};

    fn candidate(selected_text: &str) -> CaptureCandidate {
        CaptureCandidate {
            selected_text: selected_text.into(),
            sentence: format!("Context with {selected_text} intact."),
            source_app: Some("Fixture Reader".into()),
            source_title: None,
            source_url: None,
            selection_bounds: None,
            origin: CaptureOrigin::Accessibility,
        }
    }

    #[test]
    fn selection_contract_compares_exact_unicode_and_typed_errors() {
        let exact = candidate("naïve—CAFÉ 👩🏽‍💻");
        let provider_fixture = exact.clone();
        crate::block_on(assert_selection_contract(
            || FakeSelectionProvider::new(Ok(provider_fixture)),
            Ok(exact),
        ));
        crate::block_on(assert_selection_contract(
            || FakeSelectionProvider::new(Err(PlatformError::EmptySelection)),
            Err(PlatformError::EmptySelection),
        ));
        crate::block_on(assert_selection_contract(
            || FakeSelectionProvider::new(Err(PlatformError::UnsupportedElement)),
            Err(PlatformError::UnsupportedElement),
        ));
    }

    #[test]
    #[should_panic]
    fn selection_contract_rejects_a_changed_surface_form() {
        let actual = candidate("CAFÉ");
        let expected = candidate("cafe");

        crate::block_on(assert_selection_contract(
            || FakeSelectionProvider::new(Ok(actual)),
            Ok(expected),
        ));
    }

    #[test]
    fn fake_selection_provider_repeats_the_caller_supplied_response() {
        use vocab_platform_api::SelectionProvider;

        let exact = candidate("Straße");
        let provider = FakeSelectionProvider::new(Ok(exact.clone()));

        assert_eq!(
            crate::block_on(provider.capture_selection()),
            Ok(exact.clone())
        );
        assert_eq!(crate::block_on(provider.capture_selection()), Ok(exact));
    }
}
