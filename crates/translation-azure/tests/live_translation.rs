use std::{env, time::Duration};

use vocab_platform_api::TranslationProvider;
use vocab_translation_azure::{AzureTranslationProvider, AzureTranslatorConfig};

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires a developer-owned Azure Translator resource and explicit network access"]
async fn translates_fixed_text_with_developer_credentials() {
    let key = env::var("VOCAB_AZURE_TRANSLATOR_KEY")
        .expect("set VOCAB_AZURE_TRANSLATOR_KEY to run the ignored live test");
    let endpoint = env::var("VOCAB_AZURE_TRANSLATOR_ENDPOINT")
        .unwrap_or_else(|_| "https://api.cognitive.microsofttranslator.com".into());
    let region = env::var("VOCAB_AZURE_TRANSLATOR_REGION")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let target = env::var("VOCAB_AZURE_TRANSLATOR_LIVE_TARGET").unwrap_or_else(|_| "de".into());
    let config = AzureTranslatorConfig::new(endpoint, key, region, Duration::from_secs(10))
        .expect("live Azure configuration must be valid");
    let provider = AzureTranslationProvider::new(config).expect("Azure client must initialize");

    let result = provider
        .translate("hello", "auto", &target)
        .await
        .expect("fixed live translation must succeed");

    assert!(!result.translated_text.trim().is_empty());
    assert!(!result.source_language.trim().is_empty());
    assert_ne!(result.source_language, "auto");
    assert_eq!(result.target_language, target);
}
