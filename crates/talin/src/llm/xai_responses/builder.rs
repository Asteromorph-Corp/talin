use secrecy::SecretString;
use talin_provider::xai_responses::{self, ClientConfig};

use crate::llm::LlmRequestOptions;
use crate::llm::xai_responses::{ResponsesOptions, ResponsesPricing, XaiResponses};

/// A builder for constructing [`XaiResponses`].
pub struct XaiResponsesBuilder {
    http_client: Option<reqwest::Client>,
    api_base: String,
    api_key: Option<SecretString>,
    model: Option<String>,
    default_options: ResponsesOptions,
    pricing: Option<ResponsesPricing>,
}

impl XaiResponsesBuilder {
    /// Constructs a new [`XaiResponsesBuilder`].
    ///
    /// This is the same as calling [`XaiResponses::builder`].
    #[must_use]
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            http_client: None,
            api_base: api_base.into(),
            api_key: None,
            model: None,
            default_options: ResponsesOptions::default(),
            pricing: None,
        }
    }

    /// Sets the HTTP client to use for requests.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Sets the API base.
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    /// Sets the API key for authentication.
    pub fn with_api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets the model id.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Configures the default call options, merging the given options into any already set.
    ///
    /// Accepts either [`ResponsesOptions`] or the canonical [`LlmOptions`](crate::llm::LlmOptions).
    pub fn with_default_options(mut self, options: impl Into<ResponsesOptions>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }

    /// Sets the pricing for the configured model.
    pub fn with_pricing(mut self, pricing: ResponsesPricing) -> Self {
        self.pricing = Some(pricing);
        self
    }

    /// Returns a [`XaiResponses`] instance that uses this [`XaiResponsesBuilder`] configuration.
    pub fn build(self) -> XaiResponses {
        let http_client = self.http_client.unwrap_or_default();

        let mut config = ClientConfig::new(self.api_base);
        if let Some(api_key) = self.api_key {
            config = config.with_api_key(api_key);
        }

        let client = xai_responses::Client::with_config(config).with_http_client(http_client);

        let model = self.model.unwrap_or_else(|| "grok-beta".to_string());

        XaiResponses::new(client, model, self.default_options, self.pricing)
    }
}

impl Default for XaiResponsesBuilder {
    fn default() -> Self {
        Self::new("https://api.x.ai/v1")
    }
}
