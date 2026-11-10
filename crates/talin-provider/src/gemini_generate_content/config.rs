use reqwest::header::HeaderMap;
use secrecy::{ExposeSecret, SecretString};

use crate::error::Error;
use crate::gemini_generate_content::types::ApiErrorDetails;

const GEMINI_API_KEY: &str = "GEMINI_API_KEY";

const DEFAULT_API_BASE: &str = "https://generativelanguage.googleapis.com";

/// Configuration for the Gemini Generate Content API client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The api base.
    ///
    /// Defaults to "<https://generativelanguage.googleapis.com>".
    api_base: String,

    /// The API key used for authentication.
    ///
    /// See Gemini API authentication documentation.
    api_key: Option<SecretString>,
}

impl ClientConfig {
    /// Creates a new [`ClientConfig`] with the default configuration.
    ///
    /// The api key is read from the environment variable `GEMINI_API_KEY`.
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            api_base: api_base.into(),
            ..Default::default()
        }
    }

    /// Sets the API key for the Gemini API.
    pub fn with_api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Returns the url by concatenating the `api_base` and `path`.
    pub(super) fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base, path)
    }

    /// Returns the [`HeaderMap`] with provided configurations.
    ///
    /// # Error
    /// Returns error if any of the header values provided contain invalid characters. See
    /// [`HeaderValue::from_str`](reqwest::header::HeaderValue::from_str) for more details.
    pub(super) fn headers(&self) -> Result<HeaderMap, Error<ApiErrorDetails>> {
        let mut map = HeaderMap::new();

        if let Some(api_key) = &self.api_key {
            map.insert(
                "x-goog-api-key",
                api_key
                    .expose_secret()
                    .parse()
                    .map_err(|_| Error::invalid_header("x-goog-api-key", "(secret)"))?,
            );
        }

        Ok(map)
    }
}

/// The default implementation for the [`ClientConfig`].
///
/// The api key is read from the environment variable `GEMINI_API_KEY`.
impl Default for ClientConfig {
    fn default() -> Self {
        let api_key = std::env::var(GEMINI_API_KEY).ok().map(Into::into);

        Self {
            api_base: DEFAULT_API_BASE.to_string(),
            api_key,
        }
    }
}
