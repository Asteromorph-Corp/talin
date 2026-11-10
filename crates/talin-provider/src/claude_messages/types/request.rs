use serde::Serialize;
use serde_with::skip_serializing_none;

#[cfg(feature = "mcp-client-2025-11-20")]
use crate::claude_messages::types::McpServer;
use crate::claude_messages::types::{
    CacheControlEphemeral, CompactionConfig, ContentBlockParam, ContextManagementConfig,
    DiagnosticsParam, MessageCreateParamsContainer, Metadata, Model, OutputConfig,
    ServiceTierParam, Speed, SystemBlockParam, TextOrItems, ThinkingConfigParam, ToolChoice,
    ToolUnion,
};

/// Request body for the [Messages](https://docs.claude.com/en/api/messages) endpoint.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone)]
#[non_exhaustive]
pub struct MessagesRequest {
    /// The model that will complete your prompt.
    ///
    /// See [`model`](https://docs.claude.com/en/api/messages#body-model)
    pub model: Model,

    /// Input messages.
    ///
    /// See [`messages`](https://docs.claude.com/en/api/messages#body-messages)
    pub messages: Vec<MessageParam>,

    /// The maximum number of tokens to generate before stopping.
    ///
    /// See [`max_tokens`](https://docs.claude.com/en/api/messages#body-max-tokens)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub max_tokens: u32,

    /// Cache control configuration for the request.
    #[cfg_attr(test, schema(inline = false))]
    pub cache_control: Option<CacheControlEphemeral>,

    /// [On-demand compaction](https://platform.claude.com/docs/en/api/http/beta/messages#beta_compaction_config).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction: Option<CompactionConfig>,

    /// [Context management](https://platform.claude.com/docs/en/api/http/beta/messages#beta_context_management_config).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_management: Option<ContextManagementConfig>,

    /// Container identifier for reuse across requests.
    ///
    /// See [`container`](https://docs.claude.com/en/api/messages#body-container)
    pub container: Option<MessageCreateParamsContainer>,

    /// Request-level diagnostics, such as prompt-cache divergence reporting.
    pub diagnostics: Option<DiagnosticsParam>,

    /// The geographic region to use for inference.
    pub inference_geo: Option<String>,

    /// MCP servers to be utilized in this request.
    ///
    /// See [`mcp_servers`](https://docs.claude.com/en/api/messages#body-mcp-servers)
    #[cfg(feature = "mcp-client-2025-11-20")]
    #[cfg_attr(test, schema(nullable = false))]
    pub mcp_servers: Option<Vec<McpServer>>,

    /// An object describing metadata about the request.
    ///
    /// See [`metadata`](https://docs.claude.com/en/api/messages#body-metadata)
    #[cfg_attr(test, schema(nullable = false))]
    pub metadata: Option<Metadata>,

    /// Determines whether to use priority capacity (if available) or standard capacity for this
    /// request.
    ///
    /// See [`service_tier`](https://docs.claude.com/en/api/messages#body-service-tier)
    #[cfg_attr(test, schema(nullable = false))]
    pub service_tier: Option<ServiceTierParam>,

    /// The inference speed mode for this request.
    ///
    /// See [`speed`](https://docs.claude.com/en/api/messages#body-speed)
    pub speed: Option<Speed>,

    /// Custom text sequences that will cause the model to stop generating.
    ///
    /// See [`stop_sequences`](https://docs.claude.com/en/api/messages#body-stop-sequences)
    #[cfg_attr(test, schema(nullable = false))]
    pub stop_sequences: Option<Vec<String>>,

    /// Whether to incrementally stream the response using server-sent events.
    ///
    /// See [`stream`](https://docs.claude.com/en/api/messages#body-stream)
    #[cfg_attr(test, schema(nullable = false))]
    pub stream: Option<bool>,

    /// System prompt.
    ///
    /// See [`system`](https://docs.claude.com/en/api/messages#body-system)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::text_or_system_block_params_schema))]
    pub system: Option<TextOrItems<SystemBlockParam>>,

    /// Configuration for enabling Claude's extended thinking.
    ///
    /// See [`thinking`](https://docs.claude.com/en/api/messages#body-thinking)
    #[cfg_attr(test, schema(nullable = false))]
    pub thinking: Option<ThinkingConfigParam>,

    /// Definition of tools that the model may use.
    ///
    /// See [`tools`](https://docs.claude.com/en/api/messages#body-tools)
    #[cfg_attr(test, schema(nullable = false))]
    pub tools: Option<Vec<ToolUnion>>,

    /// How the model should use the provided tools. The model can use a specific tool, any
    /// available tool, decide by itself, or not use tools at all.
    #[cfg_attr(test, schema(nullable = false))]
    pub tool_choice: Option<ToolChoice>,

    /// The output configuration (effort level and/or format).
    #[cfg_attr(test, schema(nullable = false))]
    pub output_config: Option<OutputConfig>,
}

/// A single input message.
///
/// See [`messages`](https://docs.claude.com/en/api/messages#body-messages)
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "role", rename_all = "snake_case")]
#[non_exhaustive]
pub enum MessageParam {
    /// A message from the user.
    User(UserMessageParam),

    /// A message from the assistant.
    Assistant(AssistantMessageParam),

    /// A system message.
    System(SystemMessageParam),
}

/// A message from the user.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone)]
#[non_exhaustive]
pub struct UserMessageParam {
    /// The message content.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::text_or_content_block_params_schema))]
    pub content: TextOrItems<ContentBlockParam>,
}

/// A message from the assistant.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone)]
#[non_exhaustive]
pub struct AssistantMessageParam {
    /// The message content.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::text_or_content_block_params_schema))]
    pub content: TextOrItems<ContentBlockParam>,
}

/// A system message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone)]
#[non_exhaustive]
pub struct SystemMessageParam {
    /// The message content.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::text_or_system_block_params_schema))]
    pub content: TextOrItems<SystemBlockParam>,
}

impl MessagesRequest {
    /// Creates a new [`MessagesRequest`].
    pub fn new(model: impl Into<Model>, messages: Vec<MessageParam>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            messages,
            max_tokens,
            cache_control: None,
            compaction: None,
            context_management: None,
            container: None,
            diagnostics: None,
            inference_geo: None,
            metadata: None,
            service_tier: None,
            speed: None,
            stop_sequences: None,
            stream: None,
            system: None,
            thinking: None,
            tools: None,
            tool_choice: None,
            #[cfg(feature = "mcp-client-2025-11-20")]
            mcp_servers: None,
            output_config: None,
        }
    }

    /// Sets the [`cache_control`](Self::cache_control) for the request.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Sets the on-demand compaction configuration.
    pub fn with_compaction(mut self, compaction: CompactionConfig) -> Self {
        self.compaction = Some(compaction);
        self
    }

    /// Sets the context management configuration.
    pub fn with_context_management(mut self, context_management: ContextManagementConfig) -> Self {
        self.context_management = Some(context_management);
        self
    }

    /// Sets the [`container`](Self::container) for the request.
    pub fn with_container(mut self, container: MessageCreateParamsContainer) -> Self {
        self.container = Some(container);
        self
    }

    /// Sets the [`diagnostics`](Self::diagnostics) for the request.
    pub fn with_diagnostics(mut self, diagnostics: DiagnosticsParam) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    /// Sets the [`inference_geo`](Self::inference_geo) for the request.
    pub fn with_inference_geo(mut self, inference_geo: impl Into<String>) -> Self {
        self.inference_geo = Some(inference_geo.into());
        self
    }

    /// Sets the [`mcp_servers`](Self::mcp_servers) for the request.
    #[cfg(feature = "mcp-client-2025-11-20")]
    pub fn with_mcp_servers(mut self, mcp_servers: Vec<McpServer>) -> Self {
        self.mcp_servers = Some(mcp_servers);
        self
    }

    /// Sets the [`metadata`](Self::metadata) for the request.
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the [`service_tier`](Self::service_tier) for the request.
    pub fn with_service_tier(mut self, service_tier: ServiceTierParam) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    /// Sets the [`speed`](Self::speed) for the request.
    pub fn with_speed(mut self, speed: Speed) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Sets the [`stop_sequences`](Self::stop_sequences) for the request.
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Sets whether to [`stream`](Self::stream) the response.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets the [`system`](Self::system) prompt for the request.
    pub fn with_system(mut self, system: impl Into<TextOrItems<SystemBlockParam>>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Sets the [`thinking`](Self::thinking) configuration for the request.
    pub fn with_thinking(mut self, thinking: ThinkingConfigParam) -> Self {
        self.thinking = Some(thinking);
        self
    }

    /// Sets the [`tools`](Self::tools) for the request.
    pub fn with_tools(mut self, tools: Vec<ToolUnion>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the [`tool_choice`](Self::tool_choice) for the request.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the [`output_config`](Self::output_config) for the request.
    pub fn with_output_config(mut self, output_config: OutputConfig) -> Self {
        self.output_config = Some(output_config);
        self
    }
}

impl MessageParam {
    /// Creates a user [`MessageParam`] with the given content.
    pub fn user(content: impl Into<TextOrItems<ContentBlockParam>>) -> Self {
        Self::User(UserMessageParam {
            content: content.into(),
        })
    }

    /// Creates an assistant [`MessageParam`] with the given content.
    pub fn assistant(content: impl Into<TextOrItems<ContentBlockParam>>) -> Self {
        Self::Assistant(AssistantMessageParam {
            content: content.into(),
        })
    }

    /// Creates a system [`MessageParam`] with the given content.
    pub fn system(content: impl Into<TextOrItems<SystemBlockParam>>) -> Self {
        Self::System(SystemMessageParam {
            content: content.into(),
        })
    }

    /// Returns the content of a user or assistant message, or `None` for a system message.
    pub fn content(&self) -> Option<&TextOrItems<ContentBlockParam>> {
        match self {
            Self::User(m) => Some(&m.content),
            Self::Assistant(m) => Some(&m.content),
            Self::System(_) => None,
        }
    }

    /// Returns the content of a user or assistant message mutably, or `None` for a system message.
    pub fn content_mut(&mut self) -> Option<&mut TextOrItems<ContentBlockParam>> {
        match self {
            Self::User(m) => Some(&mut m.content),
            Self::Assistant(m) => Some(&mut m.content),
            Self::System(_) => None,
        }
    }

    /// Takes the content of a user or assistant message, or `None` for a system message.
    pub fn into_content(self) -> Option<TextOrItems<ContentBlockParam>> {
        match self {
            Self::User(m) => Some(m.content),
            Self::Assistant(m) => Some(m.content),
            Self::System(_) => None,
        }
    }
}
