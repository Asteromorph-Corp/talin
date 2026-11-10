use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::ResponseModality;
use crate::utils::string_enum;

/// Statistics on the interaction request's token usage.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct Usage {
    /// A breakdown of cached token usage by modality.
    #[cfg_attr(test, schema(nullable = false))]
    pub cached_tokens_by_modality: Option<Vec<ModalityTokens>>,

    /// Grounding tool count.
    #[cfg_attr(test, schema(nullable = false))]
    pub grounding_tool_count: Option<Vec<GroundingToolCount>>,

    /// A breakdown of input token usage by modality.
    #[cfg_attr(test, schema(nullable = false))]
    pub input_tokens_by_modality: Option<Vec<ModalityTokens>>,

    /// A breakdown of output token usage by modality.
    #[cfg_attr(test, schema(nullable = false))]
    pub output_tokens_by_modality: Option<Vec<ModalityTokens>>,

    /// A breakdown of tool-use token usage by modality.
    #[cfg_attr(test, schema(nullable = false))]
    pub tool_use_tokens_by_modality: Option<Vec<ModalityTokens>>,

    /// Number of input tokens used in the first turn of this interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub initial_input_tokens: Option<i32>,

    /// Number of tokens in the cached part of the prompt (the cached content).
    #[cfg_attr(test, schema(nullable = false))]
    pub total_cached_tokens: Option<i32>,

    /// Number of tokens in the prompt (context).
    #[cfg_attr(test, schema(nullable = false))]
    pub total_input_tokens: Option<i32>,

    /// Total number of tokens across all the generated responses.
    #[cfg_attr(test, schema(nullable = false))]
    pub total_output_tokens: Option<i32>,

    /// Number of tokens of thoughts for thinking models.
    #[cfg_attr(test, schema(nullable = false))]
    pub total_thought_tokens: Option<i32>,

    /// Total token count for the interaction request (prompt + responses + other internal
    /// tokens).
    #[cfg_attr(test, schema(nullable = false))]
    pub total_tokens: Option<i32>,

    /// Number of tokens present in tool-use prompt(s).
    #[cfg_attr(test, schema(nullable = false))]
    pub total_tool_use_tokens: Option<i32>,
}

/// The token count for a single response modality.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ModalityTokens {
    /// The modality associated with the token count.
    #[cfg_attr(test, schema(nullable = false))]
    pub modality: Option<ResponseModality>,

    /// Number of tokens for the modality.
    #[cfg_attr(test, schema(nullable = false))]
    pub tokens: Option<i32>,
}

/// The number of grounding tool counts for a given grounding tool type.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GroundingToolCount {
    /// The number of grounding tool counts.
    #[cfg_attr(test, schema(nullable = false))]
    pub count: Option<i32>,

    /// The grounding tool type associated with the count.
    #[serde(rename = "type")]
    #[cfg_attr(test, schema(nullable = false))]
    pub grounding_type: Option<GroundingToolType>,
}

string_enum! {
    /// The grounding tool type associated with a [`GroundingToolCount`].
    #[serde(rename_all = "snake_case")]
    pub enum GroundingToolType {
        /// Grounding with Google Web Search and Image Search, & Web Grounding for Enterprise.
        GoogleSearch,

        /// Grounding with Google Maps.
        GoogleMaps,
    }
}
