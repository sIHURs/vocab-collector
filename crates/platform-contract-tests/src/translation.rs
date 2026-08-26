use std::sync::Arc;

use async_trait::async_trait;
use vocab_platform_api::{Capability, PlatformError, TranslationProvider, TranslationResult};

/// A translation provider for platforms that report translation as unavailable.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnavailableTranslationProvider;

#[async_trait]
impl TranslationProvider for UnavailableTranslationProvider {
    async fn translate(
        &self,
        _text: &str,
        _source: &str,
        _target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        Err(PlatformError::Unsupported(Capability::Translation))
    }
}

/// Verifies exact translation output or a typed error through the public contract.
pub async fn assert_translation_contract<F>(
    provider_factory: F,
    text: &str,
    source: &str,
    target: &str,
    expected: Result<TranslationResult, PlatformError>,
) where
    F: FnOnce() -> Arc<dyn TranslationProvider>,
{
    let actual = provider_factory().translate(text, source, target).await;
    assert_eq!(actual, expected);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use vocab_platform_api::{Capability, PlatformError, TranslationProvider};

    use super::{UnavailableTranslationProvider, assert_translation_contract};

    #[test]
    fn unavailable_translation_contract_returns_the_typed_capability_error() {
        let provider: Arc<dyn TranslationProvider> = Arc::new(UnavailableTranslationProvider);
        crate::block_on(assert_translation_contract(
            || provider,
            "Straße—CAFÉ 👩🏽‍💻",
            "de",
            "en",
            Err(PlatformError::Unsupported(Capability::Translation)),
        ));
    }

    #[test]
    #[should_panic]
    fn translation_contract_rejects_a_different_typed_error() {
        let provider: Arc<dyn TranslationProvider> = Arc::new(UnavailableTranslationProvider);
        crate::block_on(assert_translation_contract(
            || provider,
            "Straße",
            "de",
            "en",
            Err(PlatformError::EmptySelection),
        ));
    }
}
