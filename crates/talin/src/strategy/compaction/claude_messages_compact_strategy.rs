use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{CompactionStrategy, Compactor, CustomCompactStrategy, RequestParts, TokenBudget};
use crate::llm::claude_messages::ClaudeSystemMessage;
use crate::llm::claude_messages::types::{
    CompactionConfig, ContentBlock, MessageParam, ToolChoice,
};
use crate::llm::{ClaudeMessages, Llm, LlmError, LlmExt};
use crate::schemas::WithUsage;
use crate::transcript::AgentTranscript;

/// Produces a signed Claude compaction using the session's system prompt and tools.
///
/// Endpoint support (live-tested 2026-10): Anthropic Messages. OpenRouter Messages silently drops
/// the `compaction` request field, so the call returns no summary and fails.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeMessagesCompactor {
    /// Optional instructions for the summarizer.
    pub instructions: Option<String>,
    /// Overrides the configured `max_tokens` for the summarization call.
    pub max_tokens: Option<u32>,
}

#[async_trait]
impl Compactor<ClaudeMessages> for ClaudeMessagesCompactor {
    async fn compact(
        &mut self,
        llm: &ClaudeMessages,
        context: (ClaudeSystemMessage, Vec<MessageParam>),
        parts: &RequestParts<ClaudeMessages>,
    ) -> Result<WithUsage<Option<MessageParam>>, LlmError> {
        let mut request = parts.build(llm, context);
        let mut compaction = CompactionConfig::new();
        compaction.instructions = self.instructions.clone();
        request.compaction = Some(compaction);
        request.stop_sequences = None;
        if matches!(
            request.tool_choice,
            Some(ToolChoice::Any(_) | ToolChoice::Tool(_))
        ) {
            request.tool_choice = None;
        }
        if let Some(output) = &mut request.output_config {
            output.format = None;
        }
        if let Some(max_tokens) = self.max_tokens {
            request.max_tokens = max_tokens;
        }
        let response = llm.send_request(request).await?;
        let usage = llm.compute_usage(Some(response.usage.clone()));
        // The response shape is undocumented beyond its block types, so only the latest
        // compaction block carrying a summary is kept; any other content is discarded.
        let summary = response.content.into_iter().rev().find(|block| {
            matches!(block, ContentBlock::Compaction(block) if
                block.content.as_ref().is_some_and(|text| !text.is_empty()) ||
                block.encrypted_content.as_ref().is_some_and(|text| !text.is_empty()))
        });
        // Retrying the same input and budget would fail again on every step.
        let Some(summary) = summary else {
            return Err(LlmError::invalid_response(format!(
                "on-demand compaction returned no summary (stop reason: {:?})",
                response.stop_reason
            )));
        };
        Ok(WithUsage::new(Some(vec![summary].into()), usage))
    }
}

/// Compacts an older prefix on demand and retains recent native messages verbatim.
/// Requires the `compact-2026-09-04` beta header on the configured Claude client.
///
/// Endpoint support (live-tested 2026-10): Anthropic Messages. OpenRouter Messages has no
/// `/messages/count_tokens` (HTTP 404) and drops the `compaction` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessagesCompactStrategy {
    #[serde(flatten)]
    /// Trigger and retained-context token budgets.
    pub budget: TokenBudget,
    #[serde(flatten)]
    /// Provider-specific summarization settings.
    pub compactor: ClaudeMessagesCompactor,
}

impl ClaudeMessagesCompactStrategy {
    /// Creates a strategy with a trigger and a target budget for recent history.
    pub fn new(threshold: u32, retain_tokens: u32) -> Self {
        Self {
            budget: TokenBudget::new(threshold, retain_tokens),
            compactor: Default::default(),
        }
    }
}

#[async_trait]
impl CompactionStrategy<ClaudeMessages> for ClaudeMessagesCompactStrategy {
    async fn compact(
        &mut self,
        llm: &ClaudeMessages,
        transcript: &mut AgentTranscript<ClaudeSystemMessage, MessageParam>,
        parts: &RequestParts<ClaudeMessages>,
    ) -> Result<WithUsage<bool>, LlmError> {
        let mut strategy = CustomCompactStrategy::new(self.compactor.clone(), self.budget.clone());
        strategy.compact(llm, transcript, parts).await
    }
}
