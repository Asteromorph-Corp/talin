use secrecy::SecretString;
use talin_provider::sglang_generate::{self, ClientConfig};

use crate::llm::LlmRequestOptions;
use crate::llm::instructor::{CompletionInstructor, DefaultInstructor};
use crate::llm::sglang_generate::{GenerateOptions, GeneratePricing, SglangGenerate};

/// A builder for constructing a [`SglangGenerate`].
pub struct SglangGenerateBuilder {
    /// The HTTP client to use for requests.
    http_client: Option<reqwest::Client>,
    /// The base URL of the SGLang server (e.g. `http://localhost:30000`).
    api_base: String,
    /// The API key to use for authentication with the SGLang server, if any.
    api_key: Option<SecretString>,
    /// The model name to report in usage records.
    model: String,
    /// The instructor used to format the prompt and parse tool calls.
    instructor: Box<dyn CompletionInstructor>,
    /// The default call options for the LLM.
    default_options: GenerateOptions,
    /// The pricing for the configured model.
    pricing: Option<GeneratePricing>,
}

impl SglangGenerateBuilder {
    /// Constructs a new [`SglangGenerateBuilder`] targeting the given server base URL.
    ///
    /// This is the same as calling [`SglangGenerate::builder`].
    #[must_use]
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            http_client: None,
            api_base: api_base.into(),
            api_key: None,
            model: "sglang".to_string(),
            instructor: Box::new(DefaultInstructor),
            default_options: GenerateOptions::default(),
            pricing: None,
        }
    }

    /// Sets the HTTP client to use for requests.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Sets the base URL of the SGLang server.
    pub fn with_api_base(mut self, base_url: impl Into<String>) -> Self {
        self.api_base = base_url.into();
        self
    }

    /// Sets the api key to use for authentication with the SGLang server.
    pub fn with_api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets the model name reported in usage records.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the instructor.
    ///
    /// The instructor decides how the conversation is formatted into a single prompt and how tool
    /// calls are instructed and parsed. Defaults to [`DefaultInstructor`].
    pub fn with_instructor(mut self, instructor: impl CompletionInstructor + 'static) -> Self {
        self.instructor = Box::new(instructor);
        self
    }

    /// Configures the default call options, merging the given options into any already set.
    ///
    /// Accepts either [`GenerateOptions`] or the canonical [`LlmOptions`](crate::llm::LlmOptions).
    pub fn with_default_options(mut self, options: impl Into<GenerateOptions>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }

    /// Sets the pricing for the configured model.
    pub fn with_pricing(mut self, pricing: GeneratePricing) -> Self {
        self.pricing = Some(pricing);
        self
    }

    /// Returns a [`SglangGenerate`] that uses this [`SglangGenerateBuilder`] configuration.
    pub fn build(self) -> SglangGenerate {
        let http_client = self.http_client.unwrap_or_default();

        let mut config = ClientConfig::new(self.api_base);
        if let Some(api_key) = self.api_key {
            config = config.with_api_key(api_key);
        }

        let client = sglang_generate::Client::with_config(config).with_http_client(http_client);

        SglangGenerate::new(
            client,
            self.model,
            self.instructor,
            self.default_options,
            self.pricing,
        )
    }
}
