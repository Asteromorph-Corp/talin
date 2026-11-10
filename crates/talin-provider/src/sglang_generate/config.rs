use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};

use crate::error::Error;

/// The configuration for the SGLang generate API client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The api base URL.
    api_base: String,
    /// The API key used for authentication.
    api_key: Option<SecretString>,
}

impl ClientConfig {
    /// Creates a new `ClientConfig` with the given API base URL.
    pub fn new(api_base: String) -> Self {
        Self {
            api_base,
            api_key: None,
        }
    }

    /// Sets the API key for the client configuration.
    pub fn with_api_key(mut self, api_key: SecretString) -> Self {
        self.api_key = Some(api_key);
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
    /// See [`HeaderValue::from_str`] for more details.
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
