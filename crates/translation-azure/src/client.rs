use std::{fmt, time::Duration};

use async_trait::async_trait;
use reqwest::{Client, Url, header};
use tokio::time::sleep;
use vocab_platform_api::{PlatformError, TranslationProvider, TranslationResult};

use crate::{
    error::AzureTranslationError,
    protocol::{TranslateRequest, TranslateResponse},
};

const MAX_ATTEMPTS: usize = 3;

#[derive(Clone)]
pub struct AzureTranslatorConfig {
    endpoint: Url,
    key: String,
    region: Option<String>,
    timeout: Duration,
}

impl AzureTranslatorConfig {
    pub fn new(
        endpoint: impl AsRef<str>,
        key: impl Into<String>,
        region: Option<String>,
        timeout: Duration,
    ) -> Result<Self, PlatformError> {
        Self::parse(endpoint.as_ref(), key.into(), region, timeout, false)
    }

    #[doc(hidden)]
    pub fn for_test(
        endpoint: impl AsRef<str>,
        key: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, PlatformError> {
        Self::parse(endpoint.as_ref(), key.into(), None, timeout, true)
    }

    #[doc(hidden)]
    pub fn with_test_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    fn parse(
        endpoint: &str,
        key: String,
        region: Option<String>,
        timeout: Duration,
        allow_http: bool,
    ) -> Result<Self, PlatformError> {
        let endpoint = Url::parse(endpoint).map_err(|_| {
            PlatformError::Operation("azure translation endpoint is invalid".into())
        })?;
        if endpoint.scheme() != "https" && !(allow_http && endpoint.scheme() == "http") {
            return Err(PlatformError::Operation(
                "azure translation endpoint must use HTTPS".into(),
            ));
        }
        if key.trim().is_empty() {
            return Err(PlatformError::Operation(
                "azure translation key is missing".into(),
            ));
        }
        if timeout.is_zero() {
            return Err(PlatformError::Operation(
                "azure translation timeout is invalid".into(),
            ));
        }
        Ok(Self {
            endpoint,
            key,
            region: region.filter(|value| !value.trim().is_empty()),
            timeout,
        })
    }
}

impl fmt::Debug for AzureTranslatorConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AzureTranslatorConfig")
            .field("endpoint", &"<redacted>")
            .field("key", &"<redacted>")
            .field("region", &self.region.as_ref().map(|_| "<configured>"))
            .field("timeout", &self.timeout)
            .finish()
    }
}

#[derive(Clone)]
pub struct AzureTranslationProvider {
    client: Client,
    config: AzureTranslatorConfig,
}

impl AzureTranslationProvider {
    pub fn new(config: AzureTranslatorConfig) -> Result<Self, PlatformError> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|_| {
                PlatformError::Operation("azure translation client setup failed".into())
            })?;
        Ok(Self { client, config })
    }

    async fn translate_once(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, (AzureTranslationError, Option<Duration>)> {
        let mut url = self
            .config
            .endpoint
            .join("translate")
            .map_err(|_| (AzureTranslationError::InvalidResponse, None))?;
        {
            let mut query = url.query_pairs_mut();
            query
                .append_pair("api-version", "3.0")
                .append_pair("to", target);
            if source != "auto" {
                query.append_pair("from", source);
            }
        }
        let mut request = self
            .client
            .post(url)
            .header("Ocp-Apim-Subscription-Key", &self.config.key)
            .json(&[TranslateRequest { text }]);
        if let Some(region) = &self.config.region {
            request = request.header("Ocp-Apim-Subscription-Region", region);
        }
        let response = request.send().await.map_err(|error| {
            let kind = if error.is_timeout() {
                AzureTranslationError::Timeout
            } else {
                AzureTranslationError::Network
            };
            (kind, None)
        })?;
        if !response.status().is_success() {
            let retry_after = retry_after(response.headers());
            return Err((
                AzureTranslationError::from_status(response.status()),
                retry_after,
            ));
        }
        let mut payload: Vec<TranslateResponse> = response
            .json()
            .await
            .map_err(|_| (AzureTranslationError::InvalidResponse, None))?;
        let payload = payload
            .pop()
            .ok_or((AzureTranslationError::InvalidResponse, None))?;
        let translation = payload
            .translations
            .into_iter()
            .next()
            .filter(|value| !value.text.is_empty())
            .ok_or((AzureTranslationError::InvalidResponse, None))?;
        let source_language = if source == "auto" {
            payload
                .detected_language
                .map(|value| value.language)
                .filter(|value| !value.is_empty())
                .ok_or((AzureTranslationError::InvalidResponse, None))?
        } else {
            source.to_owned()
        };
        Ok(TranslationResult {
            translated_text: translation.text,
            source_language,
            target_language: translation.to,
        })
    }
}

#[async_trait]
impl TranslationProvider for AzureTranslationProvider {
    async fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<TranslationResult, PlatformError> {
        let mut last_error = AzureTranslationError::InvalidResponse;
        for attempt in 0..MAX_ATTEMPTS {
            match self.translate_once(text, source, target).await {
                Ok(result) => return Ok(result),
                Err((error, retry_after)) => {
                    last_error = error;
                    if !error.retryable() || attempt + 1 == MAX_ATTEMPTS {
                        break;
                    }
                    sleep(retry_after.unwrap_or_else(|| Duration::from_millis(25 << attempt)))
                        .await;
                }
            }
        }
        Err(last_error.into())
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
