use reqwest::StatusCode;
use vocab_platform_api::PlatformError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AzureTranslationError {
    Unauthorized,
    QuotaExceeded,
    RateLimited,
    Timeout,
    Network,
    UnsupportedLanguage,
    Server,
    InvalidResponse,
}

impl AzureTranslationError {
    pub(crate) fn from_status(status: StatusCode) -> Self {
        match status.as_u16() {
            401 | 403 => Self::Unauthorized,
            429 => Self::RateLimited,
            400 => Self::UnsupportedLanguage,
            402 => Self::QuotaExceeded,
            500..=599 => Self::Server,
            _ => Self::InvalidResponse,
        }
    }

    pub(crate) const fn retryable(self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::Timeout | Self::Network | Self::Server
        )
    }

    pub(crate) const fn diagnostic(self) -> &'static str {
        match self {
            Self::Unauthorized => "azure translation authentication failed",
            Self::QuotaExceeded => "azure translation quota is exhausted",
            Self::RateLimited => "azure translation was rate limited",
            Self::Timeout => "azure translation timed out",
            Self::Network => "azure translation network request failed",
            Self::UnsupportedLanguage => "azure translation language pair is unsupported",
            Self::Server => "azure translation service failed",
            Self::InvalidResponse => "azure translation returned an invalid response",
        }
    }
}

impl From<AzureTranslationError> for PlatformError {
    fn from(value: AzureTranslationError) -> Self {
        Self::Operation(value.diagnostic().into())
    }
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::AzureTranslationError;

    #[test]
    fn status_codes_keep_actionable_categories_inside_the_provider() {
        assert_eq!(
            AzureTranslationError::from_status(StatusCode::UNAUTHORIZED),
            AzureTranslationError::Unauthorized
        );
        assert_eq!(
            AzureTranslationError::from_status(StatusCode::TOO_MANY_REQUESTS),
            AzureTranslationError::RateLimited
        );
        assert_eq!(
            AzureTranslationError::from_status(StatusCode::PAYMENT_REQUIRED),
            AzureTranslationError::QuotaExceeded
        );
        assert_eq!(
            AzureTranslationError::from_status(StatusCode::BAD_REQUEST),
            AzureTranslationError::UnsupportedLanguage
        );
        assert_eq!(
            AzureTranslationError::from_status(StatusCode::BAD_GATEWAY),
            AzureTranslationError::Server
        );
        assert_eq!(
            AzureTranslationError::from_status(StatusCode::NOT_FOUND),
            AzureTranslationError::InvalidResponse
        );
        assert!(AzureTranslationError::RateLimited.retryable());
        assert!(AzureTranslationError::Timeout.retryable());
        assert!(AzureTranslationError::Network.retryable());
        assert!(!AzureTranslationError::Unauthorized.retryable());
    }
}
