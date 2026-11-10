use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{Compactor, RequestParts};
use crate::llm::{
    Llm, LlmError, LlmExt, LlmRequest, LlmRequestItem, LlmRequestOptions, LlmResponse,
};
use crate::schemas::{BodyMessage, ContentExt, LlmStep, UserMessage, WithUsage};

/// Summarizes with an ordinary generation request: appends `instructions` as a user message to the
/// selected history, forbids tool calls, and stores the reply's native content as the summary.
/// The request keeps the session's tools but uses default call options, so strategy settings such
/// as a structured response format do not shape the summary; the LLM's own defaults still apply.
///
/// Works with every LLM, including endpoints without native compaction such as Chat Completions,
/// OpenRouter and xAI (live-tested 2026-10 with [`MessageCount`](super::MessageCount) and
/// [`CompactionCutoff`](super::CompactionCutoff)). Summary quality depends on the model and the
/// instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionCompactor {
    /// The summarization request, sent as the final user message.
    pub instructions: String,
}

impl InstructionCompactor {
    /// Creates a compactor with the given summarization instructions.
    pub fn new(instructions: impl Into<String>) -> Self {
        Self {
            instructions: instructions.into(),
        }
    }
}

#[async_trait]
impl<L: Llm> Compactor<L> for InstructionCompactor {
    async fn compact(
        &mut self,
        llm: &L,
        (system, mut items): (L::System, Vec<L::Item>),
        parts: &RequestParts<L>,
    ) -> Result<WithUsage<Option<L::Item>>, LlmError> {
        let instructions = UserMessage::new(self.instructions.clone());
        items.push(L::Item::from_message(BodyMessage::User(instructions))?);
        let parts = RequestParts::<L> {
            tools: parts.tools.clone(),
            builtin_tools: parts.builtin_tools.clone(),
            mcp_servers: parts.mcp_servers.clone(),
            options: L::Options::from_default(),
        };
        let request = parts.build(llm, (system, items)).force_final();
        let response = llm.send_request(request).await?;
        let usage = llm.compute_usage(response.get_usage());
        // A truncated or tool-calling reply is not a summary to keep in history.
        if let Some(truncation) = response.truncation() {
            return Err(LlmError::Truncated(truncation));
        }
        let (step, native) = llm.parse_response(response)?.into_parts();
        match step {
            LlmStep::Finish(message)
                if message.content.to_text().is_some_and(|t| !t.is_empty()) =>
            {
                Ok(WithUsage::new(Some(native), usage))
            }
            LlmStep::Finish(_) => Err(LlmError::invalid_response("summarization returned no text")),
            LlmStep::ToolCall(_) => Err(LlmError::invalid_response(
                "summarization returned a tool call instead of a summary",
            )),
        }
    }
}
