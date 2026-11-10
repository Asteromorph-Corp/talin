use secrecy::SecretString;
use talin_provider::claude_messages::{self, AnthropicConfig, VertexConfig};
use talin_provider::vertex::VertexTokenProvider;

use crate::llm::LlmRequestOptions;
use crate::llm::claude_messages::{ClaudeMessages, MessagesOptions, MessagesPricing};

const DEFAULT_API_BASE: &str = "https://api.anthropic.com/v1";

/// Provider-agnostic configuration shared by every Claude builder.
#[derive(Default)]
struct ClaudeCommon {
    http_client: Option<reqwest::Client>,
    api_base: Option<String>,
    model: Option<String>,
    anthropic_beta: Option<Vec<String>>,
    anthropic_version: Option<String>,
    default_options: MessagesOptions,
    pricing: Option<MessagesPricing>,
}

/// A builder for constructing [`ClaudeMessages`].
pub struct ClaudeMessagesBuilder {
    common: ClaudeCommon,
    api_key: Option<SecretString>,
}

/// A builder for constructing [`ClaudeMessages`] for Claude models on Vertex AI.
pub struct VertexClaudeMessagesBuilder {
    common: ClaudeCommon,
    project_id: String,
    location: Option<String>,
    token_provider: Option<Box<dyn VertexTokenProvider>>,
}

/// Generates the setters shared by every builder that embeds a [`ClaudeCommon`] as `common`.
macro_rules! common_builder_methods {
    () => {
        /// Sets the HTTP client to use for requests.
        pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
            self.common.http_client = Some(client);
            self
        }

        /// Sets the API base.
        pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
            self.common.api_base = Some(api_base.into());
            self
        }

        /// Sets the model id.
        pub fn with_model(mut self, model: impl Into<String>) -> Self {
            self.common.model = Some(model.into());
            self
        }

        /// Sets the Anthropic beta version to use.
        pub fn with_anthropic_beta(mut self, beta: impl Into<String>) -> Self {
            self.common
                .anthropic_beta
                .get_or_insert_default()
                .push(beta.into());
            self
        }

        /// Sets the Anthropic beta versions to use.
        pub fn with_anthropic_betas(mut self, betas: impl IntoIterator<Item = String>) -> Self {
            self.common
                .anthropic_beta
                .get_or_insert_default()
                .extend(betas);
            self
        }

        /// Sets the Anthropic API version to use.
        pub fn with_anthropic_version(mut self, version: impl Into<String>) -> Self {
            self.common.anthropic_version = Some(version.into());
            self
        }

        /// Configures the default call options, merging the given options into any already set.
        ///
        /// Accepts either [`MessagesOptions`] or the canonical
        /// [`LlmOptions`](crate::llm::LlmOptions).
        pub fn with_default_options(mut self, options: impl Into<MessagesOptions>) -> Self {
            self.common.default_options = self.common.default_options.merge(options.into());
            self
        }

        /// Sets the pricing for the configured model.
        pub fn with_pricing(mut self, pricing: MessagesPricing) -> Self {
            self.common.pricing = Some(pricing);
            self
        }
    };
}

impl ClaudeCommon {
    fn new(api_base: impl Into<String>) -> Self {
        Self {
            http_client: None,
            api_base: Some(api_base.into()),
            model: None,
            anthropic_beta: None,
            anthropic_version: None,
            default_options: MessagesOptions::default(),
            pricing: None,
        }
    }
}

impl ClaudeMessagesBuilder {
    /// Constructs a new [`ClaudeMessagesBuilder`].
    ///
    /// This is the same as calling [`ClaudeMessages::builder`].
    #[must_use]
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            common: ClaudeCommon::new(api_base),
            api_key: None,
        }
    }

    common_builder_methods!();

    /// Sets the API key for authentication.
    pub fn with_api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Returns a [`VertexClaudeMessagesBuilder`] instance that uses this [`ClaudeMessagesBuilder`]
    /// configurations.
    pub fn vertex(mut self, project_id: impl Into<String>) -> VertexClaudeMessagesBuilder {
        if self.common.api_base.as_deref() == Some(DEFAULT_API_BASE) {
            self.common.api_base = None;
        }

        VertexClaudeMessagesBuilder {
            common: self.common,
            project_id: project_id.into(),
            location: None,
            token_provider: None,
        }
    }

    /// Returns a [`ClaudeMessages`] instance that uses this [`ClaudeMessagesBuilder`]
    /// configurations.
    pub fn build(self) -> ClaudeMessages {
        let http_client = self.common.http_client.unwrap_or_default();

        let api_base = self
            .common
            .api_base
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string());

        let mut config = AnthropicConfig::new(api_base);
        if let Some(api_key) = self.api_key {
            config = config.with_api_key(api_key);
        }
        if let Some(anthropic_beta) = self.common.anthropic_beta {
            config = config.with_anthropic_beta(anthropic_beta);
        }
        if let Some(anthropic_version) = self.common.anthropic_version {
            config = config.with_anthropic_version(anthropic_version);
        }

        let client = claude_messages::Client::with_config(config).with_http_client(http_client);

        let model = self
            .common
            .model
            .unwrap_or_else(|| "claude-3-7-sonnet-20250219".to_string());

        ClaudeMessages::new(
            client,
            model,
            self.common.default_options,
            self.common.pricing,
        )
    }
}

impl VertexClaudeMessagesBuilder {
    /// Creates a new [`VertexClaudeMessagesBuilder`] for the given project id.
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            common: ClaudeCommon::default(),
            project_id: project_id.into(),
            location: None,
            token_provider: None,
        }
    }

    common_builder_methods!();

    /// Sets the location for the Vertex AI API.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Sets the token provider for the Vertex AI API.
    pub fn with_token_provider(
        mut self,
        token_provider: impl VertexTokenProvider + 'static,
    ) -> Self {
        self.token_provider = Some(Box::new(token_provider));
        self
    }

    /// Returns a [`ClaudeMessages`] instance that uses this [`VertexClaudeMessagesBuilder`]
    /// configurations.
    pub fn build(self) -> ClaudeMessages {
        let http_client = self.common.http_client.unwrap_or_default();

        let mut config = VertexConfig::new(self.project_id);
        if let Some(api_base) = self.common.api_base {
            config = config.with_api_base(api_base);
        }

        if let Some(token_provider) = self.token_provider {
            config = config.with_token_provider(token_provider);
        }
        if let Some(anthropic_beta) = self.common.anthropic_beta {
            config = config.with_anthropic_beta(anthropic_beta);
        }
        if let Some(anthropic_version) = self.common.anthropic_version {
            config = config.with_anthropic_version(anthropic_version);
        }
        if let Some(location) = self.location {
            config = config.with_location(location);
        }

        let client = claude_messages::Client::with_config(config).with_http_client(http_client);

        let model = self
            .common
            .model
            .unwrap_or_else(|| "claude-sonnet-5".to_string());

        ClaudeMessages::new(
            client,
            model,
            self.common.default_options,
            self.common.pricing,
        )
    }
}

impl Default for ClaudeMessagesBuilder {
    fn default() -> Self {
        Self::new(DEFAULT_API_BASE)
    }
}
