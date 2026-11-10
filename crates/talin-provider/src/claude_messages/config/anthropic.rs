use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};

use super::{ANTHROPIC_BETA, ANTHROPIC_VERSION, X_API_KEY};
use crate::claude_messages::types::ApiErrorDetails;
use crate::error::Error;

const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";

const DEFAULT_API_BASE: &str = "https://api.anthropic.com";
const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";

/// Configuration for Anthropic's first-party Messages API.
///
/// Contains settings such as the API base URL, API key, beta versions, and API version.
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// The api base.
    ///
    /// Defaults to "<https://api.anthropic.com>".
    api_base: String,

    /// The API key used for authentication.
    ///
    /// See [`x-api-key`](https://docs.claude.com/en/api/messages#parameter-x-api-key).
    api_key: Option<SecretString>,

    /// The beta version(s) to use.
    ///
    /// See [`anthropic-beta`](https://docs.claude.com/en/api/messages#parameter-anthropic-beta)
    anthropic_beta: Option<Vec<String>>,

    /// The version of the Claude API to use.
    ///
    /// Defaults to "2023-06-01".
    ///
    /// See [`anthropic-version`](https://docs.claude.com/en/api/messages#parameter-anthropic-version)
    anthropic_version: String,
}

impl AnthropicConfig {
    /// Creates a new [`AnthropicConfig`] with the given API base.
    ///
    /// The api key is read from the environment variable `ANTHROPIC_API_KEY`.
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            api_base: api_base.into(),
            ..Default::default()
        }
    }

    /// Sets the API key for the Claude API.
    pub fn with_api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets the Anthropic beta flag for the Claude API.
    pub fn with_anthropic_beta(mut self, anthropic_beta: impl IntoIterator<Item = String>) -> Self {
        self.anthropic_beta = Some(anthropic_beta.into_iter().collect());
        self
    }

    /// Sets the Anthropic version for the Claude API.
    pub fn with_anthropic_version(mut self, anthropic_version: impl Into<String>) -> Self {
        self.anthropic_version = anthropic_version.into();
        self
    }

    /// Checks whether the specific beta flag is enabled.
    pub fn is_beta_enabled(&self, flag: &str) -> bool {
        self.anthropic_beta
            .as_deref()
            .is_some_and(|beta| beta.iter().any(|b| b == flag))
    }

    /// Returns the `url` by concatenating the `api_base` and `path`.
    pub(super) fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base, path)
    }

    /// Returns the [`HeaderMap`] with provided configurations.
    ///
    /// # Error
    /// Returns error if any of the header values provided contain invalid characters. See
    /// [`HeaderValue::from_str`] for more details.
    pub(super) fn headers(&self) -> Result<HeaderMap, Error<ApiErrorDetails>> {
        let mut headers = HeaderMap::new();

        if let Some(key) = &self.api_key {
            let Ok(mut api_key) = key.expose_secret().parse::<HeaderValue>() else {
                return Err(Error::invalid_header(X_API_KEY, "(secret)"));
            };
            api_key.set_sensitive(true);
            headers.insert(X_API_KEY, api_key);
        }

        let Ok(anthropic_version) = self.anthropic_version.parse() else {
            return Err(Error::InvalidHeader(
                ANTHROPIC_VERSION,
                self.anthropic_version.clone(),
            ));
        };
        headers.insert(ANTHROPIC_VERSION, anthropic_version);

        if let Some(anthropic_beta) = &self.anthropic_beta {
            let anthropic_beta = anthropic_beta.join(",");
            let Ok(anthropic_beta) = anthropic_beta.parse() else {
                return Err(Error::InvalidHeader(ANTHROPIC_BETA, anthropic_beta));
            };
            headers.insert(ANTHROPIC_BETA, anthropic_beta);
        }

        Ok(headers)
    }
}

/// The default implementation for the [`AnthropicConfig`].
///
/// The api key is read from the environment variable `ANTHROPIC_API_KEY`.
impl Default for AnthropicConfig {
    fn default() -> Self {
        let api_key = std::env::var(ANTHROPIC_API_KEY).ok().map(Into::into);

        Self {
            api_base: DEFAULT_API_BASE.to_string(),
            api_key,
            anthropic_beta: None,
            anthropic_version: DEFAULT_ANTHROPIC_VERSION.to_string(),
        }
    }
}
