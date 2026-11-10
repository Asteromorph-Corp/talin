use std::any::type_name;

use async_trait::async_trait;

use crate::llm::{Llm, LlmError, UnsupportedContent};
use crate::mcp::McpServerHttp;
use crate::schemas::WithUsage;
use crate::tools::ToolSpec;
use crate::transcript::AgentTranscript;

mod auto_compact_strategy;
mod claude_messages_auto_compact_strategy;
mod claude_messages_compact_strategy;
mod custom_compact_strategy;
mod instruction_compactor;
mod openai_responses_auto_compact_strategy;
mod openai_responses_compact_strategy;
mod selection;

pub use auto_compact_strategy::AutoCompactStrategy;
pub use claude_messages_auto_compact_strategy::ClaudeMessagesAutoCompactStrategy;
pub use claude_messages_compact_strategy::{
    ClaudeMessagesCompactStrategy, ClaudeMessagesCompactor,
};
pub use custom_compact_strategy::{Compactor, CustomCompactStrategy};
pub use instruction_compactor::InstructionCompactor;
pub use openai_responses_auto_compact_strategy::OpenAIResponsesAutoCompactStrategy;
pub use openai_responses_compact_strategy::{
    OpenAIResponsesCompactStrategy, OpenAIResponsesCompactor,
};
use selection::compaction_range;
pub use selection::{CompactionCutoff, CompactionSelector, MessageCount, TokenBudget};

/// Native request operations used by compaction strategies.
#[async_trait]
pub trait CompactionModel: Llm {
    /// Counts the supplied request without generating output.
    /// The built-in endpoint implementations are Claude Messages and OpenAI Responses.
    /// Other models can use a custom selector or [`MessageCount`] without token counting.
    async fn count_tokens(&self, _request: &Self::Request) -> Result<u32, LlmError> {
        Err(UnsupportedContent::new(type_name::<Self>(), "token counting").into())
    }
}

/// The parts of a request other than its conversation: the session's tools and call options.
pub struct RequestParts<L: Llm> {
    /// Function and MCP tools available to the agent.
    pub tools: Vec<ToolSpec>,
    /// Provider built-in tools.
    pub builtin_tools: Vec<L::BuiltinTool>,
    /// Remote MCP servers the provider connects to itself.
    pub mcp_servers: Vec<McpServerHttp>,
    /// Call options.
    pub options: L::Options,
}

impl<L: Llm> RequestParts<L> {
    /// Builds a request for lowered transcript content.
    pub fn build(&self, llm: &L, (system, items): (L::System, Vec<L::Item>)) -> L::Request {
        llm.build_request(
            system,
            items,
            self.tools.clone(),
            self.builtin_tools.clone(),
            self.mcp_servers.clone(),
            self.options.clone(),
        )
    }
}

/// A session's policy for compacting history before generation.
/// Runs before the step's request is built, and is skipped during truncated-output continuation.
#[async_trait]
pub trait CompactionStrategy<L: Llm>: Send + Sync {
    /// Compacts selected history without removing original entries or changing their UUIDs.
    /// Returns whether the transcript changed, together with compaction usage.
    async fn compact(
        &mut self,
        llm: &L,
        transcript: &mut AgentTranscript<L::System, L::Item>,
        parts: &RequestParts<L>,
    ) -> Result<WithUsage<bool>, LlmError>;
}
