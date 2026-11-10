use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Metadata on the generation request's token usage.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UsageMetadata {
    /// Number of tokens in the prompt.
    pub prompt_token_count: Option<u32>,

    /// Number of tokens in the cached part of the prompt.
    pub cached_content_token_count: Option<u32>,

    /// Total number of tokens across all the generated response candidates.
    pub candidates_token_count: Option<u32>,

    /// Output only. Number of tokens present in tool-use prompt(s).
    pub tool_use_prompt_token_count: Option<u32>,

    /// Output only. Number of tokens of thoughts for thinking models.
    pub thoughts_token_count: Option<u32>,

    /// Total token count for the generation request (prompt + response candidates).
    pub total_token_count: Option<u32>,

    /// Output only. List of modalities that were processed in the request input.
    pub prompt_tokens_details: Option<Vec<super::ModalityTokenCount>>,

    /// Output only. List of modalities of the cached content in the request input.
    pub cache_tokens_details: Option<Vec<super::ModalityTokenCount>>,

    /// Output only. List of modalities that were returned in the response.
    pub candidates_tokens_details: Option<Vec<super::ModalityTokenCount>>,

    /// Output only. List of modalities that were processed for tool-use request inputs.
    pub tool_use_prompt_tokens_details: Option<Vec<super::ModalityTokenCount>>,

    /// Output only. The service tier used to process the request.
    pub service_tier: Option<super::ServiceTier>,
}
