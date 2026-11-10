use reqwest::header::{
    ACCEPT, AUTHORIZATION, HeaderMap, HeaderName, HeaderValue, InvalidHeaderName,
    InvalidHeaderValue,
};
use secrecy::{ExposeSecret, SecretString};

use crate::error::Error;
use crate::gemini_interactions::types::ErrorMessage;

const GEMINI_API_KEY: &str = "GEMINI_API_KEY";

const DEFAULT_API_BASE: &str = "https://generativelanguage.googleapis.com";

const DEFAULT_API_VERSION: &str = "v1";

const API_KEY_HEADER: &str = "x-goog-api-key";

/// Configuration for the Gemini Interactions API client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The API base URL.
    api_base: String,

    /// The API version segment (for example `v1`).
    api_version: String,

    /// The API key used for authentication, sent as `x-goog-api-key`.
    api_key: Option<SecretString>,

    /// An OAuth access token, sent as a bearer `Authorization` header.
    access_token: Option<SecretString>,

    /// Additional default headers applied to every request.
    default_headers: Vec<(String, String)>,
}

impl ClientConfig {
    /// Creates a new [`ClientConfig`] with the given API base URL and defaults.
    ///
    /// The API key is read from the environment variable `GEMINI_API_KEY`.
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            api_base: api_base.into(),
            ..Default::default()
        }
    }

    /// Sets the API key used for authentication.
    pub fn with_api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets an OAuth access token used for authentication (sent as a bearer token).
    pub fn with_access_token(mut self, access_token: impl Into<SecretString>) -> Self {
        self.access_token = Some(access_token.into());
        self
    }

    /// Sets the API version segment (for example `v1`).
    pub fn with_api_version(mut self, api_version: impl Into<String>) -> Self {
        self.api_version = api_version.into();
        self
    }

    /// Adds a default header applied to every request.
    pub fn with_default_header(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.default_headers.push((name.into(), value.into()));
        self
    }

    /// Returns the full URL for a resource `path` (which should begin with `/`).
    pub(super) fn url(&self, path: &str) -> String {
        format!("{}/{}{}", self.api_base, self.api_version, path)
    }

    /// Builds the [`HeaderMap`] for a request.
    ///
    /// If `streaming` is `true`, an `Accept: text/event-stream` header is added.
    ///
    /// # Errors
    /// Returns [`Error::InvalidHeader`] if any header name or value contains invalid
    /// characters.
    pub(super) fn headers(&self, streaming: bool) -> Result<HeaderMap, Error<ErrorMessage>> {
        let mut headers = HeaderMap::new();

        for (name, value) in &self.default_headers {
            let header_name = HeaderName::try_from(name.as_str())
                .map_err(|e: InvalidHeaderName| Error::InvalidHeader("default", e.to_string()))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|e: InvalidHeaderValue| Error::InvalidHeader("default", e.to_string()))?;
            headers.insert(header_name, header_value);
        }

        if let Some(token) = &self.access_token {
            let value = format!("Bearer {}", token.expose_secret());
            let header_value = HeaderValue::from_str(&value)
                .map_err(|_| Error::invalid_header("authorization", "(secret)"))?;
            headers.insert(AUTHORIZATION, header_value);
        } else if let Some(api_key) = &self.api_key {
            let header_value = HeaderValue::from_str(api_key.expose_secret())
                .map_err(|_| Error::invalid_header(API_KEY_HEADER, "(secret)"))?;
            headers.insert(HeaderName::from_static(API_KEY_HEADER), header_value);
        }

        if streaming {
            headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
        }

        Ok(headers)
    }
}

/// The default [`ClientConfig`] reads the API key from `GEMINI_API_KEY` and targets the
/// global endpoint at API version `v1`.
impl Default for ClientConfig {
    fn default() -> Self {
        let api_key = std::env::var(GEMINI_API_KEY).ok().map(Into::into);

        Self {
            api_base: DEFAULT_API_BASE.to_string(),
            api_version: DEFAULT_API_VERSION.to_string(),
            api_key,
            access_token: None,
            default_headers: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_url_with_version() {
        let config = ClientConfig::new("https://example.com");
        assert_eq!(
            config.url("/interactions"),
            "https://example.com/v1/interactions"
        );
    }

    #[test]
    fn honours_custom_api_version() {
        let config = ClientConfig::new("https://example.com").with_api_version("v1beta");
        assert_eq!(
            config.url("/interactions/abc/cancel"),
            "https://example.com/v1beta/interactions/abc/cancel"
        );
    }

    #[test]
    fn api_key_header_is_set() {
        let config = ClientConfig::new("https://example.com").with_api_key("secret".to_string());
        let headers = config.headers(false).unwrap();
        assert_eq!(headers.get(API_KEY_HEADER).unwrap(), "secret");
        assert!(!headers.contains_key(ACCEPT));
    }

    #[test]
    fn streaming_sets_accept_header() {
        let config = ClientConfig::new("https://example.com").with_api_key("secret".to_string());
        let headers = config.headers(true).unwrap();
        assert_eq!(headers.get(ACCEPT).unwrap(), "text/event-stream");
    }

    #[test]
    fn access_token_takes_precedence() {
        let config = ClientConfig::new("https://example.com")
            .with_api_key("secret".to_string())
            .with_access_token("token".to_string());
        let headers = config.headers(false).unwrap();
        assert_eq!(headers.get(AUTHORIZATION).unwrap(), "Bearer token");
        assert!(!headers.contains_key(API_KEY_HEADER));
    }
}
