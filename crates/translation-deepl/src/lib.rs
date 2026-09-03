#![forbid(unsafe_code)]

use std::{fmt, time::Duration};

use async_trait::async_trait;
use reqwest::{Client, StatusCode, Url, header};
use serde::{Deserialize, Serialize};
use tokio::time::{Instant, sleep, timeout_at};
use vocab_platform_api::{PlatformError, TranslationProvider, TranslationResult};

const MAX_ATTEMPTS: usize = 3;

#[derive(Clone)]
pub struct DeepLTranslatorConfig {
    endpoint: Url,
    key: String,
    timeout: Duration,
}

impl DeepLTranslatorConfig {
    pub fn new(
        endpoint: impl AsRef<str>,
        key: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, PlatformError> {
        Self::parse(endpoint.as_ref(), key.into(), timeout, false)
    }

    #[doc(hidden)]
    pub fn for_test(
        endpoint: impl AsRef<str>,
        key: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, PlatformError> {
        Self::parse(endpoint.as_ref(), key.into(), timeout, true)
    }

    fn parse(
        endpoint: &str,
        key: String,
        timeout: Duration,
        allow_http: bool,
    ) -> Result<Self, PlatformError> {
        let endpoint = Url::parse(endpoint).map_err(|_| {
            PlatformError::Operation("deepl translation endpoint is invalid".into())
        })?;
        if endpoint.scheme() != "https" && !(allow_http && endpoint.scheme() == "http") {
            return Err(PlatformError::Operation(
                "deepl translation endpoint must use HTTPS".into(),
            ));
        }
        if key.trim().is_empty() {
            return Err(PlatformError::Operation(
                "deepl translation key is missing".into(),
            ));
        }
        if timeout.is_zero() {
            return Err(PlatformError::Operation(
                "deepl translation timeout is invalid".into(),
            ));
        }
        Ok(Self {
            endpoint,
            key,
            timeout,
        })
    }
}

impl fmt::Debug for DeepLTranslatorConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeepLTranslatorConfig")
            .field("endpoint", &"<redacted>")
            .field("key", &"<redacted>")
            .field("timeout", &self.timeout)
            .finish()
    }
}

#[derive(Clone)]
pub struct DeepLTranslationProvider {
    client: Client,
    config: DeepLTranslatorConfig,
}

impl DeepLTranslationProvider {
    pub fn new(config: DeepLTranslatorConfig) -> Result<Self, PlatformError> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|_| {
                PlatformError::Operation("deepl translation client setup failed".into())
            })?;
        Ok(Self { client, config })
    }

    async fn translate_once(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, AttemptFailure> {
        let url = self
            .config
            .endpoint
            .join("v2/translate")
            .map_err(|_| AttemptFailure::new(DeepLError::InvalidResponse))?;
        let payload = DeepLRequest {
            text: [text],
            source_lang: (source != "auto").then(|| provider_language(source)),
            target_lang: provider_language(target),
        };
        let response = self
            .client
            .post(url)
            .header(
                header::AUTHORIZATION,
                format!("DeepL-Auth-Key {}", self.config.key),
            )
            .json(&payload)
            .send()
            .await
            .map_err(|error| {
                AttemptFailure::new(if error.is_timeout() {
                    DeepLError::Timeout
                } else {
                    DeepLError::Network
                })
            })?;
        if !response.status().is_success() {
            return Err(AttemptFailure {
                kind: DeepLError::from_status(response.status()),
                retry_after: retry_after(response.headers()),
            });
        }
        let response: DeepLResponse = response
            .json()
            .await
            .map_err(|_| AttemptFailure::new(DeepLError::InvalidResponse))?;
        let translation = response
            .translations
            .into_iter()
            .next()
            .filter(|value| !value.text.trim().is_empty())
            .ok_or_else(|| AttemptFailure::new(DeepLError::InvalidResponse))?;
        Ok(TranslationResult {
            translated_text: translation.text,
            source_language: app_language(&translation.detected_source_language),
            target_language: target.to_string(),
        })
    }
}

#[async_trait]
impl TranslationProvider for DeepLTranslationProvider {
    async fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        let deadline = Instant::now() + self.config.timeout;
        let mut last_error = DeepLError::InvalidResponse;
        for attempt in 0..MAX_ATTEMPTS {
            match timeout_at(deadline, self.translate_once(text, source, target)).await {
                Ok(Ok(result)) => return Ok(result),
                Err(_) => return Err(DeepLError::Timeout.into()),
                Ok(Err(failure)) => {
                    last_error = failure.kind;
                    if !failure.kind.retryable() || attempt + 1 == MAX_ATTEMPTS {
                        break;
                    }
                    let delay = failure
                        .retry_after
                        .unwrap_or_else(|| Duration::from_millis(25 << attempt));
                    if Instant::now() + delay >= deadline {
                        return Err(DeepLError::Timeout.into());
                    }
                    sleep(delay).await;
                }
            }
        }
        Err(last_error.into())
    }
}

#[derive(Serialize)]
struct DeepLRequest<'a> {
    text: [&'a str; 1],
    #[serde(skip_serializing_if = "Option::is_none")]
    source_lang: Option<String>,
    target_lang: String,
}

#[derive(Deserialize)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Deserialize)]
struct DeepLTranslation {
    detected_source_language: String,
    text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeepLError {
    Unauthorized,
    QuotaExceeded,
    RateLimited,
    Timeout,
    Network,
    UnsupportedLanguage,
    Server,
    InvalidResponse,
}

impl DeepLError {
    fn from_status(status: StatusCode) -> Self {
        match status.as_u16() {
            401 | 403 => Self::Unauthorized,
            429 => Self::RateLimited,
            456 => Self::QuotaExceeded,
            400 => Self::UnsupportedLanguage,
            500..=599 => Self::Server,
            _ => Self::InvalidResponse,
        }
    }

    fn retryable(self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::Timeout | Self::Network | Self::Server
        )
    }

    fn diagnostic(self) -> &'static str {
        match self {
            Self::Unauthorized => "deepl translation authentication failed",
            Self::QuotaExceeded => "deepl translation quota is exhausted",
            Self::RateLimited => "deepl translation was rate limited",
            Self::Timeout => "deepl translation timed out",
            Self::Network => "deepl translation network request failed",
            Self::UnsupportedLanguage => "deepl translation language pair is unsupported",
            Self::Server => "deepl translation service failed",
            Self::InvalidResponse => "deepl translation returned an invalid response",
        }
    }
}

impl From<DeepLError> for PlatformError {
    fn from(value: DeepLError) -> Self {
        Self::Operation(value.diagnostic().into())
    }
}

struct AttemptFailure {
    kind: DeepLError,
    retry_after: Option<Duration>,
}

impl AttemptFailure {
    fn new(kind: DeepLError) -> Self {
        Self {
            kind,
            retry_after: None,
        }
    }
}

fn provider_language(language: &str) -> String {
    language.to_ascii_uppercase()
}

fn app_language(language: &str) -> String {
    match language.to_ascii_uppercase().as_str() {
        "ZH" | "ZH-HANS" => "zh-Hans".into(),
        "ZH-HANT" => "zh-Hant".into(),
        value => value.to_ascii_lowercase(),
    }
}

fn retry_after(headers: &header::HeaderMap) -> Option<Duration> {
    headers
        .get(header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}
