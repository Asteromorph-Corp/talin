use serde::Deserialize;

use crate::claude_messages::types::{
    Container, ContentBlock, IterationUsage, MessagesResponse, OutputTokensDetails,
    RefusalStopDetails, ServerToolUsage, StopReason, TextCitation,
};
use crate::utils::tagged_enum;

tagged_enum! {
    /// A streamed chunk of a messages response.
    ///
    /// See [streaming](https://docs.claude.com/en/docs/build-with-claude/streaming).
    #[derive(Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum RawMessageStreamEvent {
        /// The initial event with an empty message.
        MessageStart(Box<RawMessageStartEvent>),
        /// Top-level changes in the message object.
        MessageDelta(RawMessageDeltaEvent),
        /// The final event in the stream.
        MessageStop,
        /// The content block.
        ContentBlockStart(RawContentBlockStartEvent),
        /// The changes in the content block.
        ContentBlockDelta(RawContentBlockDeltaEvent),
        /// The final event for a content block indicating its completion.
        ContentBlockStop(RawContentBlockStopEvent),
    }
}

/// The message with empty content.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RawMessageStartEvent {
    /// The initial message object.
    pub message: MessagesResponse,
}

/// Top-level changes to the message object.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RawMessageDeltaEvent {
    /// The changes in the message object.
    pub delta: ResponseMessageDelta,

    /// The changes in usage information.
    pub usage: MessageDeltaUsage,
}

/// A content block.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RawContentBlockStartEvent {
    /// The index of the content block.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub index: u32,

    /// The initial content block.
    pub content_block: ContentBlock,
}

/// The changes in the content block.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RawContentBlockDeltaEvent {
    /// The index of the content block.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub index: u32,

    /// The changes in the content block.
    pub delta: ResponseMessageContentItemDelta,
}

/// The final event for a content block indicating its completion.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RawContentBlockStopEvent {
    /// The index of the content block.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub index: u32,
}

/// The changes in usage information.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MessageDeltaUsage {
    /// The cumulative number of input tokens used to crate the cache entry.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema, required = true))]
    pub cache_creation_input_tokens: Option<u32>,

    /// The cumulative number of input tokens read from the cache.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema, required = true))]
    pub cache_read_input_tokens: Option<u32>,

    /// The cumulative number of input tokens which were used.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema, required = true))]
    pub input_tokens: Option<u32>,

    /// The cumulative number of output tokens which were used.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub output_tokens: u32,

    /// Per-iteration usage, including compaction work.
    pub iterations: Option<Vec<IterationUsage>>,

    /// Breakdown of output tokens by category.
    #[cfg_attr(test, schema(required = true))]
    pub output_tokens_details: Option<OutputTokensDetails>,

    /// The number of server tool requests.
    #[cfg_attr(test, schema(required = true))]
    pub server_tool_use: Option<ServerToolUsage>,
}

/// Top-level changes to the message object.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct ResponseMessageDelta {
    /// Information about the container used in the request (for the code execution tool).
    #[cfg_attr(test, schema(required = true))]
    pub container: Option<Container>,

    /// The reason that Claude stopped.
    ///
    /// See [`MessagesResponse::stop_reason`](crate::claude_messages::types::MessagesResponse::stop_reason)
    #[cfg_attr(test, schema(required = true))]
    pub stop_reason: Option<StopReason>,

    /// Which custom stop sequence was generated, if any.
    ///
    /// See [`MessagesResponse::stop_sequence`](crate::claude_messages::types::MessagesResponse::stop_sequence)
    #[cfg_attr(test, schema(required = true))]
    pub stop_sequence: Option<String>,

    /// Structured information about why the model stopped.
    ///
    /// See [`MessagesResponse::stop_details`](crate::claude_messages::types::MessagesResponse::stop_details)
    #[cfg_attr(test, schema(required = true))]
    pub stop_details: Option<RefusalStopDetails>,
}

tagged_enum! {
    /// The changes in the content block.
    #[derive(Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ResponseMessageContentItemDelta {
        /// The changes in the [text content block](crate::claude_messages::types::TextBlock).
        TextDelta(TextDelta),
        /// The changes in the `input` field of the [tool use content
        /// block](crate::claude_messages::types::ToolUseBlock).
        InputJsonDelta(InputJsonDelta),
        /// The changes in the `citations` field of the content block.
        CitationsDelta(CitationsDelta),
        /// The changes in the `thinking` field of the [thinking content
        /// block](crate::claude_messages::types::ThinkingBlock).
        ThinkingDelta(ThinkingDelta),
        /// The changes in the `signature` field of the [thinking content
        /// block](crate::claude_messages::types::ThinkingBlock).
        SignatureDelta(SignatureDelta),
        /// The complete summary generated by automatic compaction.
        CompactionDelta(CompactionDelta),
    }
}

/// The changes in the [text content block](crate::claude_messages::types::TextBlock).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TextDelta {
    /// The text delta.
    pub text: String,
}

/// The changes in the `input` field of the [tool use content
/// block](crate::claude_messages::types::ToolUseBlock).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InputJsonDelta {
    /// The partial JSON string.
    pub partial_json: String,
}

/// The changes in the `citations` field of the content block.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationsDelta {
    /// The citation delta.
    pub citation: TextCitation,
}

/// The changes in the `thinking` field of the [thinking content
/// block](crate::claude_messages::types::ThinkingBlock).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ThinkingDelta {
    /// The thinking delta.
    pub thinking: String,
}

/// The changes in the `signature` field of the [thinking content
/// block](crate::claude_messages::types::ThinkingBlock).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SignatureDelta {
    /// The signature delta.
    pub signature: String,
}

/// A complete compaction summary sent as one streaming delta.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CompactionDelta {
    /// The summary, or `null` when compaction failed.
    pub content: Option<String>,
}

impl RawMessageStreamEvent {
    /// Returns the event type as a string.
    pub fn event_type(&self) -> &'static str {
        match self {
            RawMessageStreamEvent::MessageStart(_) => "message_start",
            RawMessageStreamEvent::MessageDelta(_) => "message_delta",
            RawMessageStreamEvent::MessageStop => "message_stop",
            RawMessageStreamEvent::ContentBlockStart(_) => "content_block_start",
            RawMessageStreamEvent::ContentBlockDelta(_) => "content_block_delta",
            RawMessageStreamEvent::ContentBlockStop(_) => "content_block_stop",
            RawMessageStreamEvent::Unknown(_) => "unknown",
        }
    }
}

impl RawMessageStartEvent {
    /// Creates a new [`RawMessageStartEvent`] from the given message.
    pub fn new(message: MessagesResponse) -> Self {
        Self { message }
    }
}

impl RawMessageDeltaEvent {
    /// Creates a new [`RawMessageDeltaEvent`] from the given delta and usage.
    pub fn new(delta: ResponseMessageDelta, usage: MessageDeltaUsage) -> Self {
        Self { delta, usage }
    }
}

impl RawContentBlockStartEvent {
    /// Creates a new [`RawContentBlockStartEvent`] from the given index and content block.
    pub fn new(index: u32, content_block: ContentBlock) -> Self {
        Self {
            index,
            content_block,
        }
    }
}

impl RawContentBlockDeltaEvent {
    /// Creates a new [`RawContentBlockDeltaEvent`] from the given index and delta.
    pub fn new(index: u32, delta: ResponseMessageContentItemDelta) -> Self {
        Self { index, delta }
    }
}

impl RawContentBlockStopEvent {
    /// Creates a new [`RawContentBlockStopEvent`] from the given index.
    pub fn new(index: u32) -> Self {
        Self { index }
    }
}

impl MessageDeltaUsage {
    /// Creates a new [`MessageDeltaUsage`] with the given output tokens.
    pub fn new(output_tokens: u32) -> Self {
        Self {
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            input_tokens: None,
            output_tokens,
            iterations: None,
            output_tokens_details: None,
            server_tool_use: None,
        }
    }

    /// Sets the [`cache_creation_input_tokens`](Self::cache_creation_input_tokens) for the usage.
    pub fn with_cache_creation_input_tokens(mut self, tokens: u32) -> Self {
        self.cache_creation_input_tokens = Some(tokens);
        self
    }

    /// Sets the [`cache_read_input_tokens`](Self::cache_read_input_tokens) for the usage.
    pub fn with_cache_read_input_tokens(mut self, tokens: u32) -> Self {
        self.cache_read_input_tokens = Some(tokens);
        self
    }

    /// Sets the [`input_tokens`](Self::input_tokens) for the usage.
    pub fn with_input_tokens(mut self, tokens: u32) -> Self {
        self.input_tokens = Some(tokens);
        self
    }

    /// Sets the [`output_tokens_details`](Self::output_tokens_details) for the usage.
    pub fn with_output_tokens_details(mut self, details: OutputTokensDetails) -> Self {
        self.output_tokens_details = Some(details);
        self
    }

    /// Sets the [`server_tool_use`](Self::server_tool_use) for the usage.
    pub fn with_server_tool_use(mut self, usage: ServerToolUsage) -> Self {
        self.server_tool_use = Some(usage);
        self
    }
}

impl ResponseMessageDelta {
    /// Creates a new [`ResponseMessageDelta`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`stop_reason`](Self::stop_reason) for the message.
    pub fn with_stop_reason(mut self, stop_reason: StopReason) -> Self {
        self.stop_reason = Some(stop_reason);
        self
    }

    /// Sets the [`stop_sequence`](Self::stop_sequence) that was generated.
    pub fn with_stop_sequence(mut self, stop_sequence: impl Into<String>) -> Self {
        self.stop_sequence = Some(stop_sequence.into());
        self
    }

    /// Sets the [`stop_details`](Self::stop_details) for the message.
    pub fn with_stop_details(mut self, stop_details: RefusalStopDetails) -> Self {
        self.stop_details = Some(stop_details);
        self
    }
}

impl TextDelta {
    /// Creates a new [`TextDelta`] from the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl InputJsonDelta {
    /// Creates a new [`InputJsonDelta`] from the given partial JSON.
    pub fn new(partial_json: impl Into<String>) -> Self {
        Self {
            partial_json: partial_json.into(),
        }
    }
}

impl CitationsDelta {
    /// Creates a new [`CitationsDelta`] from the given citation.
    pub fn new(citation: TextCitation) -> Self {
        Self { citation }
    }
}

impl ThinkingDelta {
    /// Creates a new [`ThinkingDelta`] from the given thinking text.
    pub fn new(thinking: impl Into<String>) -> Self {
        Self {
            thinking: thinking.into(),
        }
    }
}

impl SignatureDelta {
    /// Creates a new [`SignatureDelta`] from the given signature.
    pub fn new(signature: impl Into<String>) -> Self {
        Self {
            signature: signature.into(),
        }
    }
}
