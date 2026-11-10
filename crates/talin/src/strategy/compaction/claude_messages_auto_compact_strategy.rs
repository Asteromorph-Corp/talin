use std::convert::Infallible;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use talin_provider::claude_messages::types::{
    CompactEdit, ContextManagementEdit, InputTokensThreshold, MessagesRequest,
};

use crate::chain::OutputCtor;
use crate::llm::ClaudeMessages;
use crate::strategy::Strategy;

/// Enables threshold compaction for [`ClaudeMessages`], with optional summary instructions.
/// Generation continues after compaction in the same request.
/// Full history remains stored; subsequent requests render from the latest successful compaction.
///
/// Endpoint support (live-tested 2026-10): Anthropic Messages and OpenRouter Messages.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaudeMessagesAutoCompactStrategy {
    /// The input token count that triggers compaction.
    pub threshold: u32,
    /// Optional instructions for the summary.
    pub instructions: Option<String>,
}

impl ClaudeMessagesAutoCompactStrategy {
    /// Creates a strategy with an explicit token threshold and optional summary instructions.
    pub fn new(threshold: u32, instructions: Option<String>) -> Self {
        Self {
            threshold,
            instructions,
        }
    }
}

#[async_trait]
impl<O: OutputCtor> Strategy<ClaudeMessages, O> for ClaudeMessagesAutoCompactStrategy {
    type Output = ();
    type Error = Infallible;

    async fn process_request(
        &mut self,
        mut request: MessagesRequest,
    ) -> Result<MessagesRequest, Self::Error> {
        let edits = request
            .context_management
            .get_or_insert_default()
            .edits
            .get_or_insert_default();
        edits.retain(|edit| !matches!(edit, ContextManagementEdit::Compact(_)));
        let mut edit = CompactEdit::new()
            .with_trigger(InputTokensThreshold::new(self.threshold))
            .with_pause_after_compaction(false);
        edit.instructions = self.instructions.clone();
        edits.push(ContextManagementEdit::Compact(edit));
        Ok(request)
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
