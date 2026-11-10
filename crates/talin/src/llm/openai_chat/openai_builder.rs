use reqwest::Client;
use talin_provider::openai_chat;
use talin_provider::openai_chat::{Config, OpenAIConfig};

use super::OpenAIChat;
use crate::llm::LlmRequestOptions;
use crate::llm::openai_chat::{ChatOptions, ChatPricing};

/// A builder for constructing a [`OpenAIChat`].
pub struct OpenAIChatBuilder<C: Config = OpenAIConfig> {
    /// The HTTP client to use.
    http_client: Option<Client>,
    /// The API configuration.
    api_config: C,
    /// The model id.
    model: String,
    /// The default call options for the LLM.
    default_options: ChatOptions,
    /// The pricing for the configured model.
    pricing: Option<ChatPricing>,
}

impl OpenAIChatBuilder {
    /// Constructs a new [`OpenAIChatBuilder`].
    ///
    /// This is the same as calling [`OpenAIChat::builder`].
    #[must_use]
    pub fn new() -> Self {
        OpenAIChatBuilder {
            api_config: OpenAIConfig::default(),
            model: "gpt-3.5-turbo".to_string(),
            default_options: ChatOptions::default(),
            http_client: None,
            pricing: None,
        }
    }
}

impl<C: Config> OpenAIChatBuilder<C> {
    /// Sets the HTTP client to use.
    #[must_use]
    pub fn with_http_client(mut self, http_client: Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Sets the API configuration.
    pub fn with_api_config<C2: Config>(self, api_config: C2) -> OpenAIChatBuilder<C2> {
        OpenAIChatBuilder {
            http_client: self.http_client,
            api_config,
            model: self.model,
            default_options: self.default_options,
            pricing: self.pricing,
        }
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
    /// Accepts either [`ChatOptions`] or the canonical [`LlmOptions`](crate::llm::LlmOptions).
    pub fn with_default_options(mut self, options: impl Into<ChatOptions>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }

    /// Sets the pricing for the configured model.
    pub fn with_pricing(mut self, pricing: ChatPricing) -> Self {
        self.pricing = Some(pricing);
        self
    }

    /// Returns a [`OpenAIChat`] that uses this [`OpenAIChatBuilder`] configuration.
    pub fn build(self) -> OpenAIChat<C> {
        let http_client = self.http_client.unwrap_or_default();
        let client = openai_chat::Client::build(http_client, self.api_config);
        OpenAIChat::new(client, self.model, self.default_options, self.pricing)
    }
}

impl OpenAIChatBuilder<OpenAIConfig> {
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

impl Default for OpenAIChatBuilder {
    fn default() -> Self {
        Self::new()
    }
}
