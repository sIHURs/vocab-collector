//! User credentials never enter UserSettings, SQLite, exports, or command responses.
#[cfg(test)]
mod tests {
    use super::*;
    struct FakeProvider(&'static str);
    #[async_trait::async_trait]
    impl TranslationProvider for FakeProvider {
        async fn translate(
            &self,
            _: &str,
            source: &str,
            target: &str,
        ) -> Result<TranslationResult, PlatformError> {
            Ok(TranslationResult {
                translated_text: self.0.into(),
                source_language: source.into(),
                target_language: target.into(),
            })
        }
    }
    #[test]
    fn shared_runtime_switches_existing_callers_and_restores_fallback() {
        let runtime = Arc::new(TranslationRuntime::new(Arc::new(FakeProvider("default"))));
        let existing_caller: Arc<dyn TranslationProvider> = runtime.clone();
        let translate = || {
            tauri::async_runtime::block_on(existing_caller.translate("word", "en", "de"))
                .unwrap()
                .translated_text
        };
        assert_eq!(translate(), "default");
        assert!(!runtime.configured());
        runtime.state.lock().unwrap().provider = Some(Arc::new(FakeProvider("personal")));
        assert_eq!(translate(), "personal");
        assert!(runtime.configured());
        runtime.state.lock().unwrap().provider = Some(Arc::new(FakeProvider("replacement")));
        assert_eq!(translate(), "replacement");
        runtime.state.lock().unwrap().provider = None;
        assert_eq!(translate(), "default");
        assert!(!runtime.configured());
    }
    #[test]
    #[cfg(target_os = "windows")]
    fn invalid_keys_are_rejected() {
        for key in [
            "",
            "   ",
            "secret with spaces",
            "secret\r\nheader",
            "secret\0value",
            "密钥",
        ] {
            assert!(provider(key).is_err());
        }
        assert!(provider("test-key:fx").is_ok());
        assert!(provider("test-pro-key").is_ok());
    }
    #[test]
    fn status_contains_only_non_secret_metadata() {
        let runtime = TranslationRuntime::new(Arc::new(FakeProvider("private")));
        let value = serde_json::to_value(runtime.status()).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 3);
        assert_eq!(value["configured"], false);
        assert_eq!(value["storageError"], false);
    }
}
use crate::bootstrap::AppState;
use std::sync::{Arc, Mutex};
use vocab_platform_api::{PlatformError, TranslationProvider, TranslationResult};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepLSettings {
    supported: bool,
    configured: bool,
    storage_error: bool,
}

struct State {
    provider: Option<Arc<dyn TranslationProvider>>,
    storage_error: bool,
}

pub struct TranslationRuntime {
    fallback: Arc<dyn TranslationProvider>,
    state: Mutex<State>,
}

impl TranslationRuntime {
    pub fn new(fallback: Arc<dyn TranslationProvider>) -> Self {
        let state = State {
            provider: None,
            storage_error: false,
        };
        Self {
            fallback,
            state: Mutex::new(state),
        }
    }

    pub fn load_saved_key(&self) {
        #[cfg(target_os = "windows")]
        let mut state = self.state.lock().unwrap();
        #[cfg(target_os = "windows")]
        match read_key().and_then(|key| key.map(|key| provider(&key)).transpose()) {
            Ok(provider) => state.provider = provider.map(|p| p as Arc<dyn TranslationProvider>),
            Err(_) => state.storage_error = true,
        }
    }

    pub fn configured(&self) -> bool {
        self.state.lock().unwrap().provider.is_some()
    }

    fn status(&self) -> DeepLSettings {
        let state = self.state.lock().unwrap();
        DeepLSettings {
            supported: cfg!(target_os = "windows"),
            configured: state.provider.is_some(),
            storage_error: state.storage_error,
        }
    }
}

#[async_trait::async_trait]
impl TranslationProvider for TranslationRuntime {
    async fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        // Clone before awaiting so saving a key never blocks on a network request.
        let provider = self
            .state
            .lock()
            .unwrap()
            .provider
            .clone()
            .unwrap_or_else(|| self.fallback.clone());
        provider.translate(text, source, target).await
    }
}

#[cfg(target_os = "windows")]
fn credential() -> Result<keyring::Entry, String> {
    keyring::Entry::new("vocab-collector.deepl", "api-key")
        .map_err(|_| "Windows credential storage is unavailable.".into())
}

#[cfg(target_os = "windows")]
fn read_key() -> Result<Option<String>, String> {
    match credential()?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Could not read the saved DeepL key.".into()),
    }
}

#[cfg(target_os = "windows")]
fn provider(key: &str) -> Result<Arc<vocab_translation_deepl::DeepLTranslationProvider>, String> {
    use vocab_translation_deepl::{DeepLTranslationProvider, DeepLTranslatorConfig};
    let key = key.trim();
    if key.is_empty()
        || key.len() > 1024
        || key.chars().any(char::is_whitespace)
        || !key.is_ascii()
        || key.chars().any(char::is_control)
    {
        return Err("Enter a valid DeepL API key without spaces.".into());
    }
    let endpoint = if key.ends_with(":fx") {
        "https://api-free.deepl.com"
    } else {
        "https://api.deepl.com"
    };
    let config = DeepLTranslatorConfig::new(endpoint, key, std::time::Duration::from_secs(10))
        .map_err(|_| "Could not configure DeepL.".to_string())?;
    Ok(Arc::new(
        DeepLTranslationProvider::new(config)
            .map_err(|_| "Could not configure DeepL.".to_string())?,
    ))
}

#[tauri::command]
pub fn get_deepl_settings(application: tauri::State<'_, AppState>) -> DeepLSettings {
    application.translation.status()
}

#[tauri::command]
pub async fn save_deepl_key(
    application: tauri::State<'_, AppState>,
    key: String,
) -> Result<DeepLSettings, String> {
    #[cfg(target_os = "windows")]
    {
        let provider = provider(&key).map_err(|_| "invalid_key".to_string())?;
        provider.validate_key().await.map_err(|error| {
            use vocab_translation_deepl::KeyValidationError;
            match error {
                KeyValidationError::InvalidKey => "invalid_key",
                KeyValidationError::Timeout => "validation_timeout",
                KeyValidationError::Network => "validation_network",
                KeyValidationError::Unavailable | KeyValidationError::InvalidResponse => {
                    "validation_unavailable"
                }
            }
            .to_string()
        })?;
        let mut state = application.translation.state.lock().unwrap();
        credential()?.set_password(key.trim()).map_err(|_| {
            "Could not save the DeepL key in Windows credential storage.".to_string()
        })?;
        state.provider = Some(provider);
        state.storage_error = false;
        drop(state);
        Ok(application.translation.status())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (application, key);
        Err("DeepL key settings are available in the Windows app.".into())
    }
}

#[tauri::command]
pub fn remove_deepl_key(application: tauri::State<'_, AppState>) -> Result<DeepLSettings, String> {
    #[cfg(target_os = "windows")]
    {
        let mut state = application.translation.state.lock().unwrap();
        match credential()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(_) => return Err("Could not remove the saved DeepL key.".into()),
        }
        state.provider = None;
        state.storage_error = false;
        drop(state);
        Ok(application.translation.status())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = application;
        Err("DeepL key settings are available in the Windows app.".into())
    }
}
