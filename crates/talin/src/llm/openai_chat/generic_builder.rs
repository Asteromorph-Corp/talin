use reqwest::Client;
use talin_provider::openai_chat;
use talin_provider::openai_chat::{Config, OpenAIConfig};

use super::GenericChat;
use crate::llm::LlmRequestOptions;
use crate::llm::instructor::{DefaultInstructor, Instructor};
use crate::llm::models::OpenAIModel;
use crate::llm::openai_chat::{ChatOptions, ChatPricing};

/// A builder for constructing a [`GenericChat`].
pub struct GenericChatBuilder<C: Config = OpenAIConfig> {
    /// The HTTP client to use.
    http_client: Option<Client>,
    /// The API configuration.
    api_config: C,
    /// The model id.
    model: String,
    /// The [`Instructor`] used to create tool use instruction and parse tool calls.
    instructor: Box<dyn Instructor>,
    /// The default call options for the LLM.
    default_options: ChatOptions,
    /// The pricing for the configured model.
    pricing: Option<ChatPricing>,
}

impl GenericChatBuilder {
    /// Constructs a new [`GenericChatBuilder`].
    ///
    /// This is the same as calling [`GenericChat::builder`].
    #[must_use]
    pub fn new() -> Self {
        GenericChatBuilder {
            api_config: OpenAIConfig::default(),
            model: OpenAIModel::Gpt4oMini.to_string(),
            instructor: Box::new(DefaultInstructor),
            default_options: ChatOptions::default(),
            http_client: None,
            pricing: None,
        }
    }
}

impl<C: Config> GenericChatBuilder<C> {
    /// Sets the HTTP client to use.
    pub fn with_http_client(mut self, http_client: Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Sets the API configuration.
    pub fn with_api_config<C2: Config>(self, api_config: C2) -> GenericChatBuilder<C2> {
        GenericChatBuilder {
            http_client: self.http_client,
            api_config,
            model: self.model,
            instructor: self.instructor,
            default_options: self.default_options,
            pricing: self.pricing,
        }
    }

    /// Sets the model id.
    ///
    /// You can use the predefined models (e.g.,
    /// [`OpenAIModel::Gpt4oMini`] or provide a custom model
    /// name with a raw string.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the instructor.
    ///
    /// Instructor is used to used to create tool use instruction and parse tool calls.
    pub fn with_instructor(mut self, instructor: impl Instructor + 'static) -> Self {
        self.instructor = Box::new(instructor);
        self
    }

    /// Configures the call options to be used in subsequent requests, merging the given options
    /// into any already set.
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

    /// Returns a [`GenericChat`] that uses this [`GenericChatBuilder`] configuration.
    pub fn build(self) -> GenericChat<C> {
        let http_client = self.http_client.unwrap_or_default();
        let client = openai_chat::Client::build(http_client, self.api_config);
        GenericChat::new(
            client,
            self.model,
            self.instructor,
            self.default_options,
            self.pricing,
        )
    }
}

impl GenericChatBuilder<OpenAIConfig> {
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

impl Default for GenericChatBuilder {
    fn default() -> Self {
        Self::new()
    }
}
