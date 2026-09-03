use std::{collections::HashMap, time::Duration};

use vocab_platform_api::PlatformError;
use vocab_translation_azure::AzureTranslatorConfig;

const KEY: &str = "VOCAB_AZURE_TRANSLATOR_KEY";
const ENDPOINT: &str = "VOCAB_AZURE_TRANSLATOR_ENDPOINT";
const REGION: &str = "VOCAB_AZURE_TRANSLATOR_REGION";
const TIMEOUT: &str = "VOCAB_AZURE_TRANSLATOR_TIMEOUT_MS";
const DEFAULT_ENDPOINT: &str = "https://api.cognitive.microsofttranslator.com";
const DEFAULT_TIMEOUT_MS: u64 = 10_000;
const MAX_TIMEOUT_MS: u64 = 60_000;

pub fn azure_translation_config() -> Result<Option<AzureTranslatorConfig>, PlatformError> {
    #[cfg(debug_assertions)]
    let mut values = load_debug_env_file()?;
    #[cfg(not(debug_assertions))]
    let mut values = HashMap::new();

    for name in [KEY, ENDPOINT, REGION, TIMEOUT] {
        if let Ok(value) = std::env::var(name) {
            values.insert(name.to_string(), value);
        }
    }
    azure_translation_config_from_values(&values)
}

fn azure_translation_config_from_values(
    values: &HashMap<String, String>,
) -> Result<Option<AzureTranslatorConfig>, PlatformError> {
    let configured = [KEY, ENDPOINT, REGION, TIMEOUT].iter().any(|name| {
        values
            .get(*name)
            .is_some_and(|value| !value.trim().is_empty())
    });
    if !configured {
        return Ok(None);
    }
    let key = values
        .get(KEY)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| PlatformError::Operation("azure translation key is missing".into()))?;
    let endpoint = values
        .get(ENDPOINT)
        .filter(|value| !value.trim().is_empty())
        .map_or(DEFAULT_ENDPOINT, String::as_str);
    let region = values
        .get(REGION)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let timeout_ms = values
        .get(TIMEOUT)
        .filter(|value| !value.trim().is_empty())
        .map_or(Ok(DEFAULT_TIMEOUT_MS), |value| value.parse::<u64>())
        .map_err(|_| PlatformError::Operation("azure translation timeout is invalid".into()))?;
    if timeout_ms == 0 || timeout_ms > MAX_TIMEOUT_MS {
        return Err(PlatformError::Operation(
            "azure translation timeout is invalid".into(),
        ));
    }
    AzureTranslatorConfig::new(endpoint, key, region, Duration::from_millis(timeout_ms)).map(Some)
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
        if [KEY, ENDPOINT, REGION, TIMEOUT].contains(&name.trim()) {
            values.insert(name.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::azure_translation_config_from_values;

    #[test]
    fn absent_configuration_disables_translation() {
        assert!(
            azure_translation_config_from_values(&HashMap::new())
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_key_uses_safe_defaults_without_disclosing_it() {
        let values = HashMap::from([(
            "VOCAB_AZURE_TRANSLATOR_KEY".to_string(),
            "developer-secret".to_string(),
        )]);
        let config = azure_translation_config_from_values(&values)
            .unwrap()
            .unwrap();

        assert!(!format!("{config:?}").contains("developer-secret"));
    }

    #[test]
    fn partial_configuration_without_a_key_fails_safely() {
        let values = HashMap::from([(
            "VOCAB_AZURE_TRANSLATOR_REGION".to_string(),
            "private-region-value".to_string(),
        )]);
        let error = azure_translation_config_from_values(&values).unwrap_err();

        assert!(!error.to_string().contains("private-region-value"));
        assert!(error.to_string().contains("key is missing"));
    }
}
