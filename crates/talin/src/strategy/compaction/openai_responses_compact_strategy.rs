use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use talin_provider::openai_responses::Config;

use super::{CompactionStrategy, Compactor, CustomCompactStrategy, RequestParts, TokenBudget};
use crate::llm::openai_responses::helper::is_system_item;
use crate::llm::openai_responses::types::{CompactResponseRequest, InputItem, InputParam, Item};
use crate::llm::{LlmError, LlmExt, OpenAIResponses};
use crate::schemas::WithUsage;
use crate::transcript::AgentTranscript;

/// Produces the Responses API's complete native replacement vector.
///
/// Endpoint support (live-tested 2026-10): OpenAI Responses, and xAI (`https://api.x.ai/v1`),
/// whose summary drops facts that only appear in tool output. OpenRouter has no
/// `/responses/compact` (HTTP 404).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIResponsesCompactor {
    /// Optional instructions for the summarizer.
    pub instructions: Option<String>,
}

#[async_trait]
impl<C: Config + Clone + Send + Sync + 'static> Compactor<OpenAIResponses<C>>
    for OpenAIResponsesCompactor
{
    async fn compact(
        &mut self,
        llm: &OpenAIResponses<C>,
        context: (Vec<String>, Vec<Vec<InputItem>>),
        parts: &RequestParts<OpenAIResponses<C>>,
    ) -> Result<WithUsage<Option<Vec<InputItem>>>, LlmError> {
        let request = parts.build(llm, context);
        let system_items = match &request.input {
            InputParam::Items(items) => {
                items.iter().take_while(|item| is_system_item(item)).count()
            }
            InputParam::Text(_) => 0,
        };
        let mut response = llm
            .compact(CompactResponseRequest {
                model: request.model.ok_or_else(|| {
                    LlmError::InvalidRequest("compaction requires a model".into())
                })?,
                input: Some(request.input),
                instructions: self.instructions.clone(),
                ..Default::default()
            })
            .await?;
        let usage = llm.compute_usage(Some(response.usage.clone()));
        if !response
            .output
            .iter()
            .any(|item| matches!(item, InputItem::Item(Item::Compaction(_))))
        {
            return Err(LlmError::invalid_response(
                "compaction response contains no compaction item",
            ));
        }
        // The output echoes the system prompt sent with the input; requests prepend the live one.
        let echoed = response
            .output
            .iter()
            .take(system_items)
            .take_while(|item| is_system_item(item))
            .count();
        response.output.drain(..echoed);
        let native = response.into();
        Ok(WithUsage::new(Some(native), usage))
    }
}

/// Compacts an older prefix through `/responses/compact`, retaining a recent token budget.
///
/// Endpoint support (live-tested 2026-10): OpenAI Responses. OpenRouter has neither
/// `/responses/input_tokens` nor `/responses/compact` (HTTP 404); xAI has no
/// `/responses/input_tokens`, so its token budget cannot be evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponsesCompactStrategy {
    #[serde(flatten)]
    /// Trigger and retained-context token budgets.
    pub budget: TokenBudget,
    #[serde(flatten)]
    /// Provider-specific summarization settings.
    pub compactor: OpenAIResponsesCompactor,
}

impl OpenAIResponsesCompactStrategy {
    /// Creates a strategy with a trigger and a target budget for recent history.
    pub fn new(threshold: u32, retain_tokens: u32) -> Self {
        Self {
            budget: TokenBudget::new(threshold, retain_tokens),
            compactor: Default::default(),
        }
    }
}

#[async_trait]
impl<C: Config + Clone + Send + Sync + 'static> CompactionStrategy<OpenAIResponses<C>>
    for OpenAIResponsesCompactStrategy
{
    async fn compact(
        &mut self,
        llm: &OpenAIResponses<C>,
        transcript: &mut AgentTranscript<Vec<String>, Vec<InputItem>>,
        parts: &RequestParts<OpenAIResponses<C>>,
    ) -> Result<WithUsage<bool>, LlmError> {
        let mut strategy = CustomCompactStrategy::new(self.compactor.clone(), self.budget.clone());
        strategy.compact(llm, transcript, parts).await
    }
}
