use std::{collections::HashMap, time::Duration};

use vocab_platform_api::PlatformError;
use vocab_translation_azure::AzureTranslatorConfig;
use vocab_translation_deepl::DeepLTranslatorConfig;

const PROVIDER: &str = "VOCAB_TRANSLATION_PROVIDER";
const AZURE_KEY: &str = "VOCAB_AZURE_TRANSLATOR_KEY";
const AZURE_ENDPOINT: &str = "VOCAB_AZURE_TRANSLATOR_ENDPOINT";
const AZURE_REGION: &str = "VOCAB_AZURE_TRANSLATOR_REGION";
const AZURE_TIMEOUT: &str = "VOCAB_AZURE_TRANSLATOR_TIMEOUT_MS";
const DEEPL_KEY: &str = "VOCAB_DEEPL_API_KEY";
const DEEPL_ENDPOINT: &str = "VOCAB_DEEPL_ENDPOINT";
const DEEPL_TIMEOUT: &str = "VOCAB_DEEPL_TIMEOUT_MS";
const WINDOWS_OCR: &str = "VOCAB_ENABLE_WINDOWS_OCR";
const DEFAULT_AZURE_ENDPOINT: &str = "https://api.cognitive.microsofttranslator.com";
const DEFAULT_DEEPL_ENDPOINT: &str = "https://api-free.deepl.com";
const DEFAULT_TIMEOUT_MS: u64 = 10_000;
const MAX_TIMEOUT_MS: u64 = 60_000;
const CONFIG_NAMES: [&str; 9] = [
    PROVIDER,
    AZURE_KEY,
    AZURE_ENDPOINT,
    AZURE_REGION,
    AZURE_TIMEOUT,
    DEEPL_KEY,
    DEEPL_ENDPOINT,
    DEEPL_TIMEOUT,
    WINDOWS_OCR,
];

pub enum TranslationProviderConfig {
    Azure(AzureTranslatorConfig),
    DeepL(DeepLTranslatorConfig),
}

pub fn translation_provider_config() -> Result<Option<TranslationProviderConfig>, PlatformError> {
    #[cfg(debug_assertions)]
    let mut values = load_debug_env_file()?;
    #[cfg(not(debug_assertions))]
    let mut values = HashMap::new();
    for name in CONFIG_NAMES {
        if let Ok(value) = std::env::var(name) {
            values.insert(name.to_string(), value);
        }
    }
    translation_provider_config_from_values(&values)
}

pub fn windows_ocr_enabled() -> Result<bool, PlatformError> {
    #[cfg(debug_assertions)]
    {
        let mut values = load_debug_env_file()?;
        if let Ok(value) = std::env::var(WINDOWS_OCR) {
            values.insert(WINDOWS_OCR.to_string(), value);
        }
        windows_ocr_enabled_from_values(&values)
    }
    #[cfg(not(debug_assertions))]
    {
        Ok(false)
    }
}

fn windows_ocr_enabled_from_values(
    values: &HashMap<String, String>,
) -> Result<bool, PlatformError> {
    match non_empty(values, WINDOWS_OCR) {
        None | Some("false") => Ok(false),
        Some("true") => Ok(true),
        Some(_) => Err(PlatformError::Operation(
            "windows OCR developer flag is invalid".into(),
        )),
    }
}

fn translation_provider_config_from_values(
    values: &HashMap<String, String>,
) -> Result<Option<TranslationProviderConfig>, PlatformError> {
    let Some(provider) = non_empty(values, PROVIDER) else {
        return Ok(None);
    };
    match provider.to_ascii_lowercase().as_str() {
        "azure" => azure_config(values)
            .map(TranslationProviderConfig::Azure)
            .map(Some),
        "deepl" => deepl_config(values)
            .map(TranslationProviderConfig::DeepL)
            .map(Some),
        _ => Err(PlatformError::Operation(
            "translation provider selection is invalid".into(),
        )),
    }
}

fn azure_config(values: &HashMap<String, String>) -> Result<AzureTranslatorConfig, PlatformError> {
    let key = non_empty(values, AZURE_KEY)
        .ok_or_else(|| PlatformError::Operation("azure translation key is missing".into()))?;
    let endpoint = non_empty(values, AZURE_ENDPOINT).unwrap_or(DEFAULT_AZURE_ENDPOINT);
    let region = non_empty(values, AZURE_REGION).map(str::to_string);
    AzureTranslatorConfig::new(
        endpoint,
        key,
        region,
        timeout(values, AZURE_TIMEOUT, "azure")?,
    )
}

fn deepl_config(values: &HashMap<String, String>) -> Result<DeepLTranslatorConfig, PlatformError> {
    let key = non_empty(values, DEEPL_KEY)
        .ok_or_else(|| PlatformError::Operation("deepl translation key is missing".into()))?;
    let endpoint = non_empty(values, DEEPL_ENDPOINT).unwrap_or(DEFAULT_DEEPL_ENDPOINT);
    DeepLTranslatorConfig::new(endpoint, key, timeout(values, DEEPL_TIMEOUT, "deepl")?)
}

fn timeout(
    values: &HashMap<String, String>,
    name: &str,
    provider: &str,
) -> Result<Duration, PlatformError> {
    let timeout_ms = non_empty(values, name)
        .map_or(Ok(DEFAULT_TIMEOUT_MS), str::parse::<u64>)
        .map_err(|_| {
            PlatformError::Operation(format!("{provider} translation timeout is invalid"))
        })?;
    if timeout_ms == 0 || timeout_ms > MAX_TIMEOUT_MS {
        return Err(PlatformError::Operation(format!(
            "{provider} translation timeout is invalid"
        )));
    }
    Ok(Duration::from_millis(timeout_ms))
}

fn non_empty<'a>(values: &'a HashMap<String, String>, name: &str) -> Option<&'a str> {
    values
        .get(name)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(debug_assertions)]
fn load_debug_env_file() -> Result<HashMap<String, String>, PlatformError> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(".env.local");
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(_) => {
            return Err(PlatformError::Operation(
                "developer environment file could not be read".into(),
            ));
        }
    };
    let mut values = HashMap::new();
    for line in content.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            return Err(PlatformError::Operation(
                "developer environment file is invalid".into(),
            ));
        };
        if CONFIG_NAMES.contains(&name.trim()) {
            values.insert(name.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::{
        TranslationProviderConfig, translation_provider_config_from_values,
        windows_ocr_enabled_from_values,
    };
    use std::collections::HashMap;

    #[test]
    fn no_selector_stays_unavailable_even_when_keys_exist() {
        let values = HashMap::from([
            ("VOCAB_AZURE_TRANSLATOR_KEY".into(), "azure-secret".into()),
            ("VOCAB_DEEPL_API_KEY".into(), "deepl-secret".into()),
        ]);
        assert!(
            translation_provider_config_from_values(&values)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn selector_parses_only_the_chosen_provider() {
        let values = HashMap::from([
            ("VOCAB_TRANSLATION_PROVIDER".into(), "deepl".into()),
            ("VOCAB_DEEPL_API_KEY".into(), "deepl-secret".into()),
            ("VOCAB_AZURE_TRANSLATOR_ENDPOINT".into(), "not a url".into()),
        ]);
        assert!(matches!(
            translation_provider_config_from_values(&values).unwrap(),
            Some(TranslationProviderConfig::DeepL(_))
        ));
    }

    #[test]
    fn azure_and_deepl_can_each_be_selected() {
        for (provider, key_name, expected_azure) in [
            ("azure", "VOCAB_AZURE_TRANSLATOR_KEY", true),
            ("deepl", "VOCAB_DEEPL_API_KEY", false),
        ] {
            let values = HashMap::from([
                ("VOCAB_TRANSLATION_PROVIDER".into(), provider.into()),
                (key_name.into(), "developer-secret".into()),
            ]);
            let selected = translation_provider_config_from_values(&values)
                .unwrap()
                .unwrap();
            assert_eq!(
                matches!(selected, TranslationProviderConfig::Azure(_)),
                expected_azure
            );
        }
    }

    #[test]
    fn invalid_selection_and_missing_selected_key_are_content_safe() {
        let invalid =
            HashMap::from([("VOCAB_TRANSLATION_PROVIDER".into(), "private-value".into())]);
        let error = translation_provider_config_from_values(&invalid)
            .err()
            .unwrap();
        assert!(!error.to_string().contains("private-value"));
        let missing = HashMap::from([
            ("VOCAB_TRANSLATION_PROVIDER".into(), "deepl".into()),
            ("VOCAB_AZURE_TRANSLATOR_KEY".into(), "unused-secret".into()),
        ]);
        let error = translation_provider_config_from_values(&missing)
            .err()
            .unwrap();
        assert!(
            error
                .to_string()
                .contains("deepl translation key is missing")
        );
        assert!(!error.to_string().contains("unused-secret"));
    }

    #[test]
    fn windows_ocr_requires_an_explicit_valid_developer_flag() {
        assert!(!windows_ocr_enabled_from_values(&HashMap::new()).unwrap());
        assert!(
            windows_ocr_enabled_from_values(&HashMap::from([(
                "VOCAB_ENABLE_WINDOWS_OCR".into(),
                "true".into(),
            )]))
            .unwrap()
        );
        let invalid = windows_ocr_enabled_from_values(&HashMap::from([(
            "VOCAB_ENABLE_WINDOWS_OCR".into(),
            "private-invalid-value".into(),
        )]))
        .unwrap_err()
        .to_string();
        assert!(invalid.contains("windows OCR developer flag is invalid"));
        assert!(!invalid.contains("private-invalid-value"));
    }
}
