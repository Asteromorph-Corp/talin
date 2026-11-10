use std::convert::Infallible;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use talin_provider::openai_responses::Config;
use talin_provider::openai_responses::types::{
    ContextManagementParam, ContextManagementParamType, CreateResponse,
};

use crate::chain::OutputCtor;
use crate::llm::OpenAIResponses;
use crate::strategy::Strategy;

/// Enables threshold compaction for [`OpenAIResponses`].
/// Full history remains stored; subsequent requests render from the latest successful compaction.
///
/// Endpoint support (live-tested 2026-10): OpenAI Responses. OpenRouter Responses and xAI accept
/// but ignore `context_management`, so nothing is compacted.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIResponsesAutoCompactStrategy {
    /// The input token count that triggers compaction.
    pub threshold: u32,
}

impl OpenAIResponsesAutoCompactStrategy {
    /// Creates a strategy with an explicit token threshold.
    pub fn new(threshold: u32) -> Self {
        Self { threshold }
    }
}

#[async_trait]
impl<C, O> Strategy<OpenAIResponses<C>, O> for OpenAIResponsesAutoCompactStrategy
where
    C: Config + Clone + Send + Sync + 'static,
    O: OutputCtor,
{
    type Output = ();
    type Error = Infallible;

    async fn process_request(
        &mut self,
        mut request: CreateResponse,
    ) -> Result<CreateResponse, Self::Error> {
        let edits = request.context_management.get_or_insert_default();
        edits.retain(|edit| edit.r#type != ContextManagementParamType::Compaction);
        edits.push(ContextManagementParam {
            r#type: ContextManagementParamType::Compaction,
            compact_threshold: Some(self.threshold),
        });
        Ok(request)
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
