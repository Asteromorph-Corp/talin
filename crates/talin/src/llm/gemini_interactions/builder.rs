use secrecy::SecretString;
use talin_provider::gemini_interactions::{self, ClientConfig};

use crate::llm::LlmRequestOptions;
use crate::llm::gemini_interactions::{
    GeminiInteractions, InteractionsOptions, InteractionsPricing,
};

/// A builder for constructing [`GeminiInteractions`].
pub struct GeminiInteractionsBuilder {
    http_client: Option<reqwest::Client>,
    api_base: String,
    api_key: Option<SecretString>,
    api_version: Option<String>,
    model: Option<String>,
    default_options: InteractionsOptions,
    pricing: Option<InteractionsPricing>,
}

impl GeminiInteractionsBuilder {
    /// Constructs a new [`GeminiInteractionsBuilder`].
    ///
    /// This is the same as calling [`GeminiInteractions::builder`].
    #[must_use]
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            http_client: None,
            api_base: api_base.into(),
            api_key: None,
            api_version: None,
            model: None,
            default_options: InteractionsOptions::default(),
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

    /// Sets the API version.
    pub fn with_api_version(mut self, api_version: impl Into<String>) -> Self {
        self.api_version = Some(api_version.into());
        self
    }

    /// Sets the model id.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Configures the default call options, merging the given options into any already set.
    ///
    /// Accepts either [`InteractionsOptions`] or the canonical
    /// [`LlmOptions`](crate::llm::LlmOptions).
    pub fn with_default_options(mut self, options: impl Into<InteractionsOptions>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }

    /// Sets the pricing for the configured model.
    pub fn with_pricing(mut self, pricing: InteractionsPricing) -> Self {
        self.pricing = Some(pricing);
        self
    }

    /// Returns a [`GeminiInteractions`] instance that uses this builder configuration.
    pub fn build(self) -> GeminiInteractions {
        let http_client = self.http_client.unwrap_or_default();

        let mut config = ClientConfig::new(self.api_base);
        if let Some(api_key) = self.api_key {
            config = config.with_api_key(api_key);
        }

        if let Some(api_version) = self.api_version {
            config = config.with_api_version(api_version);
        }

        let client = gemini_interactions::Client::with_config(config).with_http_client(http_client);

        let model = self
            .model
            .unwrap_or_else(|| "models/gemini-3.5-flash".to_string());

        GeminiInteractions::new(client, model, self.default_options, self.pricing)
    }
}

impl Default for GeminiInteractionsBuilder {
    fn default() -> Self {
        Self::new("https://generativelanguage.googleapis.com")
    }
}
