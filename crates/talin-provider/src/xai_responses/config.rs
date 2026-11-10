use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};

use crate::error::Error;

const XAI_API_KEY: &str = "XAI_API_KEY";
const DEFAULT_API_BASE: &str = "https://api.x.ai/v1";

/// Configuration for the xAI Responses API client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The api base.
    ///
    /// Defaults to "<https://api.x.ai/v1>".
    api_base: String,

    /// The API key used for authentication.
    api_key: Option<SecretString>,
}

impl ClientConfig {
    /// Creates a new [`ClientConfig`] with the default configuration.
    ///
    /// The api key is read from the environment variable `XAI_API_KEY`.
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            api_base: api_base.into(),
            ..Default::default()
        }
    }

    /// Sets the API key for the xAI API.
    pub fn with_api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Returns the `url` by concatenating the `api_base` and `path`.
    pub(super) fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base, path)
    }

    /// Returns the [`HeaderMap`] with provided configurations.
    ///
    /// # Error
    /// Returns error if any of the header values provided contain invalid characters.
    pub(super) fn headers(&self) -> Result<HeaderMap, Error<serde_json::Value>> {
        let mut headers = HeaderMap::new();

        if let Some(key) = &self.api_key {
            let val = format!("Bearer {}", key.expose_secret());
            let Ok(mut auth_value) = HeaderValue::from_str(&val) else {
                return Err(Error::invalid_header(AUTHORIZATION.as_str(), "(secret)"));
            };
            auth_value.set_sensitive(true);
            headers.insert(AUTHORIZATION, auth_value);
        }

        Ok(headers)
    }
}

/// Default implementation for [`ClientConfig`].
///
/// The api key is read from the environment variable `XAI_API_KEY`.
impl Default for ClientConfig {
    fn default() -> Self {
        let api_key = std::env::var(XAI_API_KEY).ok().map(Into::into);

        Self {
            api_base: DEFAULT_API_BASE.to_string(),
            api_key,
        }
    }
}
