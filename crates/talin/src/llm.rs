//! A module for LLM wrappers and related functionality.

use async_trait::async_trait;
use indexmap::IndexMap;

use crate::mcp::McpServerHttp;
use crate::schemas::{
    BodyMessage, LlmStep, Message, ModelKey, Prompt, ToolErrorMessage, Usage, UsageLines, WithUsage,
};
use crate::tools::ToolSpec;
use crate::transcript::{AgentTranscript, Entry, ModelRecord, TruncatedRecord};

mod error;
pub(crate) mod merge;
mod models;
mod options;
mod preset;

pub mod claude_messages;
pub mod gemini_generate_content;
pub mod gemini_interactions;
pub mod instructor;
pub mod openai_chat;
pub mod openai_responses;
pub mod sglang_generate;
pub mod xai_responses;

/// Re-exports of the Vertex AI provider utilities.
pub mod vertex {
    pub use talin_provider::vertex::*;
}

pub use claude_messages::ClaudeMessages;
pub use error::{ContinuationError, LlmError, Truncation, UnsupportedContent};
pub use gemini_generate_content::GeminiGenerateContent;
pub use gemini_interactions::GeminiInteractions;
pub use models::OpenAIModel;
pub use openai_chat::{GenericChat, OpenAIChat};
pub use openai_responses::OpenAIResponses;
pub use options::LlmOptions;
pub use preset::LlmPreset;
pub use sglang_generate::SglangGenerate;
pub use xai_responses::XaiResponses;

pub use crate::any::{AnyPreset, UnknownModel};

/// The request type of the [`Llm`] `L`.
pub type RequestOf<L> = <L as Llm>::Request;
/// The response type of the [`Llm`] `L`.
pub type ResponseOf<L> = <L as Llm>::Response;
/// The request item type of the [`Llm`] `L`.
pub type ItemOf<L> = <L as Llm>::Item;
/// The system prompt type of the [`Llm`] `L`.
pub type SystemOf<L> = <L as Llm>::System;
/// The call options type of the [`Llm`] `L`.
pub type OptionsOf<L> = <L as Llm>::Options;
/// The built-in tool type of the [`Llm`] `L`.
pub type BuiltinToolOf<L> = <L as Llm>::BuiltinTool;

mod sealed {
    pub trait Sealed {}
}

/// A common interface for interacting with LLM backends.
#[async_trait]
pub trait Llm: Clone + Send + Sync {
    /// The provider-specific request type.
    type Request: LlmRequest;
    /// The provider-specific response type.
    type Response: LlmResponse<Usage = Self::Usage>;

    /// The provider-specific system prompt representation.
    type System: LlmRequestSystem;
    /// The provider-specific request item type.
    type Item: LlmRequestItem;
    /// The provider-specific built-in (server-side) tool type.
    type BuiltinTool: LlmBuiltinTool;
    /// The provider-specific call options.
    type Options: LlmRequestOptions;

    /// The pricing specification for the provider.
    type Pricing: LlmPricing<Usage = Self::Usage>;
    /// The usage information reported by the provider.
    type Usage: LlmUsage;

    /// Selects the transcript content to include in a request.
    ///
    /// By default, starts after the latest boundary compaction marker and retains the system
    /// prompt. Advisory markers are ignored: backends omit context before native compaction items
    /// when building the request.
    fn materialize(
        &self,
        transcript: AgentTranscript<Self::System, Self::Item>,
    ) -> (Self::System, Vec<Self::Item>) {
        let (system, entries) = transcript.into_parts();
        let start = entries
            .iter()
            .rposition(|entry| matches!(entry, Entry::CompactionMarker { boundary: true }))
            .map_or(0, |index| index + 1);
        let items = entries
            .into_iter()
            .skip(start)
            .filter_map(Entry::into_native)
            .collect();
        (system.into_parts().1, items)
    }

    /// Builds a provider-specific request from the canonical request pieces.
    fn build_request(
        &self,
        system: Self::System,
        items: Vec<Self::Item>,
        tools: Vec<ToolSpec>,
        builtin_tools: Vec<Self::BuiltinTool>,
        mcp_servers: Vec<McpServerHttp>,
        options: Self::Options,
    ) -> Self::Request;

    /// Sends the request to the provider and returns its raw response.
    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError>;

    /// Converts a provider-specific response into a transcript [`ModelRecord`].
    fn parse_response(&self, response: Self::Response)
    -> Result<ModelRecord<Self::Item>, LlmError>;

    /// Builds a follow-up request to continue a truncated model response.
    ///
    /// > Note: Providers using native structured output should reject continuation when a response
    /// > schema is set. The model is required to produce a valid schema-conforming output, so it
    /// > cannot reliably resume the truncated document and is likely to hit the token limit again.
    fn build_continuation(
        &self,
        _request: Self::Request,
        _partial: TruncatedRecord<Self::Item>,
    ) -> Result<Self::Request, ContinuationError> {
        Err(ContinuationError::Unsupported)
    }

    /// Reconstructs the complete model record from a continuation response.
    ///
    /// By default, the parsed continuation is appended to partial. Providers whose models
    /// regenerate the response from the beginning should override this method and return the
    /// continuation alone.
    fn parse_continuation(
        &self,
        response: Self::Response,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        let continuation = self.parse_response(response)?;
        Ok(partial.concat(continuation)?)
    }

    /// Whether the provider supports MCP servers. Defaults to `true`.
    fn supports_mcp(&self) -> bool {
        true
    }
    /// Returns the [`ModelKey`] identifying this model.
    fn model_key(&self) -> ModelKey;
    /// Returns the pricing specification for the model, if known.
    fn pricing(&self) -> Option<Self::Pricing>;

    /// Returns a copy of this LLM with the given options merged into its default call options.
    fn with_default_options(self, options: impl Into<Self::Options>) -> Self;
}

/// Extension methods for [`Llm`], implemented for every [`Llm`].
///
/// This trait is sealed and cannot be implemented outside this crate.
#[async_trait]
pub trait LlmExt: Llm + sealed::Sealed {
    /// Computes canonical usage using this model's configured pricing.
    fn compute_usage(&self, usage: Option<Self::Usage>) -> Usage {
        let lines = match (usage, self.pricing()) {
            (Some(usage), Some(pricing)) => pricing.compute_usage_lines(usage),
            (Some(usage), None) => Self::Pricing::default_usage_lines(usage),
            (None, _) => UsageLines::default(),
        };
        Usage(IndexMap::from_iter([(self.model_key(), lines)]))
    }

    /// Generates a single step from the prompt, along with its token usage.
    ///
    /// Fails with [`LlmError::Truncated`] if the response was cut off before completion.
    async fn generate(
        &self,
        prompt: Prompt,
        tools: Vec<ToolSpec>,
        builtin_tools: Vec<Self::BuiltinTool>,
        mcp_servers: Vec<McpServerHttp>,
        options: Self::Options,
    ) -> Result<WithUsage<LlmStep>, LlmError> {
        let (system, messages) = Message::partition(prompt.to_messages());
        let system = Self::System::from_contents(system);
        let items = messages
            .into_iter()
            .map(Self::Item::from_message)
            .collect::<Result<Vec<_>, _>>()?;

        let request = self.build_request(system, items, tools, builtin_tools, mcp_servers, options);
        let response = self.send_request(request).await?;

        // Continuing from a truncated response requires the agent's transcript.
        if let Some(truncation) = response.truncation() {
            return Err(LlmError::Truncated(truncation));
        }

        let usage = self.compute_usage(response.get_usage());

        let record = self.parse_response(response)?;

        Ok(WithUsage {
            content: record.canonical,
            usage,
        })
    }

    /// Sends `input` as a single user prompt with default options and returns the response text,
    /// excluding thoughts.
    async fn invoke(&self, input: &str) -> Result<String, LlmError> {
        let prompt = Prompt::single(input);
        let options = Self::Options::from_default();
        let result = self
            .generate(prompt, Vec::new(), Vec::new(), Vec::new(), options)
            .await?
            .content
            .drop_thought()
            .to_string();
        Ok(result)
    }
}

impl<T: Llm> sealed::Sealed for T {}
impl<T: Llm> LlmExt for T {}

/// Provider-specific request for an [`Llm`].
pub trait LlmRequest: Sized + Send + Sync + 'static {
    /// Remove the `tool_choice` option from request.
    ///
    /// Used to prevent using the same tool choice for all subsequent requests.
    fn clear_tool_choice(self) -> Self;

    /// Configures the request so the model is forced to emit a final text answer.
    ///
    /// Drops all function tools and applies the provider's mechanism for refusing tool use
    /// (e.g. `tool_choice = none`, removing `tool_config`, scrubbing the last assistant message,
    /// etc.). Built-in/provider-side tools may be retained at the implementor's discretion.
    fn force_final(self) -> Self;
}

/// Provider-specific response for an [`Llm`].
pub trait LlmResponse: Sized + Send + Sync + 'static {
    /// The usage information reported in the response.
    type Usage: LlmUsage;

    /// Extracts usage information from the response, if available.
    fn get_usage(&self) -> Option<Self::Usage>;

    /// Why generation stopped before the model finished its output, if it did.
    ///
    /// The response still holds a valid prefix of the intended output, so it is parsed as usual.
    /// A trailing tool call, if any, has incomplete arguments and must not be executed.
    fn truncation(&self) -> Option<Truncation> {
        None
    }
}

/// Provider-specific system prompt representation for an [`Llm`].
pub trait LlmRequestSystem: Clone + Send + Sync + 'static {
    /// Constructs a new system item with the given contents.
    fn from_contents(contents: Vec<String>) -> Self;

    /// Constructs a new system item with the given contents, using the original provider-specific
    /// system item as a template.
    fn from_contents_with(contents: Vec<String>, base: Self) -> Self;
}
impl LlmRequestSystem for Vec<String> {
    fn from_contents(contents: Vec<String>) -> Self {
        contents
    }
    fn from_contents_with(contents: Vec<String>, _: Self) -> Self {
        Self::from_contents(contents)
    }
}

/// Provider-specific request item for an [`Llm`].
pub trait LlmRequestItem: Clone + Send + Sync + 'static {
    /// Constructs a request item from the given message.
    ///
    /// Fails with [`UnsupportedContent`] when the message carries content the provider cannot
    /// represent.
    fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent>;

    /// Updates a request item from the given message, using the original provider-specific items as
    /// a base.
    fn from_message_with(
        message: BodyMessage,
        base: Self,
        retain_thought: bool,
    ) -> Result<Self, UnsupportedContent>;

    /// Whether this item carries a provider compaction that replaces earlier context.
    ///
    /// The runner places an advisory `CompactionMarker` before a model record for which this
    /// returns true.
    fn contains_native_compaction(&self) -> bool {
        false
    }

    /// Appends a continuation's native content without modifying its text parts.
    fn concat(self, _continuation: Self) -> Result<Self, UnsupportedContent> {
        Err(UnsupportedContent::new(
            std::any::type_name::<Self>(),
            "native continuation",
        ))
    }

    /// Constructs a request item representing an error from a tool call.
    ///
    /// `None` can be returned if the provider does not support tool errors as request items.
    fn from_tool_error(error: &ToolErrorMessage) -> Option<Self>;
}
impl LlmRequestItem for BodyMessage {
    fn concat(self, mut continuation: Self) -> Result<Self, UnsupportedContent> {
        let Self::Assistant(prefix) = self else {
            return Err(UnsupportedContent::new(
                "canonical messages",
                "non-text continuation",
            ));
        };
        let content = match &mut continuation {
            Self::Assistant(message) => &mut message.content,
            Self::ToolCall(message) => &mut message.content,
            _ => {
                return Err(UnsupportedContent::new(
                    "canonical messages",
                    "non-model continuation",
                ));
            }
        };
        let mut parts = prefix.content;
        parts.append(content);
        *content = parts;
        Ok(continuation)
    }

    fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent> {
        Ok(message)
    }
    fn from_message_with(
        message: BodyMessage,
        _: Self,
        _: bool,
    ) -> Result<Self, UnsupportedContent> {
        Self::from_message(message)
    }
    fn from_tool_error(message: &ToolErrorMessage) -> Option<Self> {
        Some(BodyMessage::Tool(message.as_output()))
    }
}

/// Provider-specific built-in (server-side) tools for an [`Llm`].
pub trait LlmBuiltinTool: Clone + Send + Sync + 'static {}
impl LlmBuiltinTool for () {}

/// Provider-specific call options for an [`Llm`].
pub trait LlmRequestOptions: Clone + Send + Sync + 'static {
    /// Constructs provider-specific options from the canonical [`LlmOptions`].
    ///
    /// Provider-specific options that have no canonical counterpart take their default values.
    fn from_options(options: LlmOptions) -> Self;

    /// Constructs provider-specific options with default values for all fields.
    fn from_default() -> Self {
        Self::from_options(LlmOptions::default())
    }

    /// Merges another option into this one.
    ///
    /// For each option field, the incoming option must be preferred for each value when set.
    /// Otherwise, the current option should be retained.
    fn merge(self, other: Self) -> Self;
}
impl LlmRequestOptions for LlmOptions {
    fn from_options(options: LlmOptions) -> Self {
        options
    }

    fn merge(self, other: Self) -> Self {
        self.merge(other)
    }
}

/// Pricing specification for an [`Llm`].
pub trait LlmPricing: Send + Sync + 'static {
    /// The usage information this pricing is computed from.
    type Usage: LlmUsage;

    /// Builds usage lines without prices, used when no pricing is configured.
    fn default_usage_lines(usage: Self::Usage) -> UsageLines;
    /// Builds usage lines carrying prices computed from this pricing.
    fn compute_usage_lines(&self, usage: Self::Usage) -> UsageLines;
}

/// Token usage information for an [`Llm`].
pub trait LlmUsage: Send + Sync + 'static {}

/// Warns that `mcp_servers` are dropped because the provider has no native MCP support.
pub(crate) fn warn_unsupported_mcp(provider: &str, mcp_servers: &[McpServerHttp]) {
    if !mcp_servers.is_empty() {
        log::warn!("{provider} does not support MCP servers, they will be ignored");
    }
}
