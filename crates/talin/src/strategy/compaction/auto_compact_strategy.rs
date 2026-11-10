use std::convert::Infallible;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use talin_provider::claude_messages::types::MessagesRequest;
use talin_provider::openai_responses::Config;
use talin_provider::openai_responses::types::CreateResponse;

use super::{ClaudeMessagesAutoCompactStrategy, OpenAIResponsesAutoCompactStrategy};
use crate::chain::OutputCtor;
use crate::llm::{ClaudeMessages, OpenAIResponses};
use crate::strategy::Strategy;

/// Enables threshold compaction for [`ClaudeMessages`] or [`OpenAIResponses`].
/// Full history remains stored; subsequent requests render from the latest successful compaction.
/// Use [`ClaudeMessagesAutoCompactStrategy`] to configure summary instructions.
///
/// Endpoint support (live-tested 2026-10): Anthropic Messages, OpenAI Responses and OpenRouter
/// Messages compact. OpenRouter Responses and xAI ignore `context_management`, so nothing is
/// compacted.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutoCompactStrategy {
    /// The input token count that triggers compaction.
    pub threshold: u32,
}

impl AutoCompactStrategy {
    /// Creates a strategy with an explicit token threshold.
    pub fn new(threshold: u32) -> Self {
        Self { threshold }
    }
}

#[async_trait]
impl<O: OutputCtor> Strategy<ClaudeMessages, O> for AutoCompactStrategy {
    type Output = ();
    type Error = Infallible;

    async fn process_request(
        &mut self,
        request: MessagesRequest,
    ) -> Result<MessagesRequest, Self::Error> {
        let mut strategy = ClaudeMessagesAutoCompactStrategy::new(self.threshold, None);
        Strategy::<ClaudeMessages, O>::process_request(&mut strategy, request).await
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

#[async_trait]
impl<C, O> Strategy<OpenAIResponses<C>, O> for AutoCompactStrategy
where
    C: Config + Clone + Send + Sync + 'static,
    O: OutputCtor,
{
    type Output = ();
    type Error = Infallible;

    async fn process_request(
        &mut self,
        request: CreateResponse,
    ) -> Result<CreateResponse, Self::Error> {
        let mut strategy = OpenAIResponsesAutoCompactStrategy::new(self.threshold);
        Strategy::<OpenAIResponses<C>, O>::process_request(&mut strategy, request).await
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
