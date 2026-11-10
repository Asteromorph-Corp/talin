use reqwest::Client;
use talin_provider::openai_responses;
use talin_provider::openai_responses::{Config, OpenAIConfig};

use super::{OpenAIResponses, ResponsesOptions, ResponsesPricing};
use crate::llm::LlmRequestOptions;

/// A builder for constructing a [`OpenAIResponses`].
pub struct OpenAIResponsesBuilder<C: Config> {
    /// The HTTP client to use.
    http_client: Option<Client>,
    /// The API configuration.
    api_config: C,
    /// The model id.
    model: String,
    /// The default call options for the LLM.
    default_options: ResponsesOptions,
    /// The pricing for the configured model.
    pricing: Option<ResponsesPricing>,
}

impl<C: Config + Default> OpenAIResponsesBuilder<C> {
    /// Constructs a new [`OpenAIResponsesBuilder`].
    ///
    /// This is the same as calling [`OpenAIResponses::builder`].
    #[must_use]
    pub fn new() -> Self {
        OpenAIResponsesBuilder {
            http_client: None,
            api_config: C::default(),
            model: "gpt-4o-mini".to_string(),
            default_options: ResponsesOptions::default(),
            pricing: None,
        }
    }
}

impl<C: Config> OpenAIResponsesBuilder<C> {
    /// Sets the HTTP client to use.
    #[must_use]
    pub fn with_http_client(mut self, http_client: Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Sets the API configuration.
    pub fn with_api_config(mut self, api_config: C) -> Self {
        self.api_config = api_config;
        self
    }

    /// Sets the model id.
    ///
    /// You can use the predefined models (e.g.,
    /// [`OpenAIModel::Gpt4oMini`](crate::llm::OpenAIModel::Gpt4oMini)) or provide a custom model
    /// name with a raw string.
    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
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

    /// Returns a [`OpenAIResponses`] that uses this [`OpenAIResponsesBuilder`] configuration.
    pub fn build(self) -> OpenAIResponses<C> {
        let http_client = self.http_client.unwrap_or_default();
        let client = openai_responses::Client::build(http_client, self.api_config);
        OpenAIResponses::new(client, self.model, self.default_options, self.pricing)
    }
}

impl OpenAIResponsesBuilder<OpenAIConfig> {
    /// Sets the API base.
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_config = self.api_config.with_api_base(api_base);
        self
    }

    /// Sets the API key for authentication.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_config = self.api_config.with_api_key(api_key);
        self
    }
}

impl<C: Config + Default> Default for OpenAIResponsesBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}
