use std::{env, time::Duration};

use vocab_platform_api::TranslationProvider;
use vocab_translation_deepl::{DeepLTranslationProvider, DeepLTranslatorConfig};

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires a developer-owned DeepL API resource and explicit network access"]
async fn translates_fixed_text_with_developer_credentials() {
    let key = env::var("VOCAB_DEEPL_API_KEY")
        .expect("set VOCAB_DEEPL_API_KEY to run the ignored live test");
    let endpoint =
        env::var("VOCAB_DEEPL_ENDPOINT").unwrap_or_else(|_| "https://api-free.deepl.com".into());
    let target = env::var("VOCAB_DEEPL_LIVE_TARGET").unwrap_or_else(|_| "de".into());
    let config = DeepLTranslatorConfig::new(endpoint, key, Duration::from_secs(10))
        .expect("live DeepL configuration must be valid");
    let provider = DeepLTranslationProvider::new(config).expect("DeepL client must initialize");

    let result = provider
        .translate("hello", "auto", &target)
        .await
        .expect("fixed live translation must succeed");

    assert!(!result.translated_text.trim().is_empty());
    assert!(!result.source_language.trim().is_empty());
    assert_ne!(result.source_language, "auto");
    assert_eq!(result.target_language, target);
}
