use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::gemini_generate_content::types::{Content, SafetyRating};

mod citation_metadata;
mod finish_reason;
mod grounding_attribution;
mod grounding_metadata;
mod logprobs_result;
mod url_context_metadata;

pub use citation_metadata::*;
pub use finish_reason::*;
pub use grounding_attribution::*;
pub use grounding_metadata::*;
pub use logprobs_result::*;
pub use url_context_metadata::*;

/// A response candidate generated from the model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Candidate {
    /// Output only. Generated content returned from the model.
    pub content: Option<Content>,

    /// Optional. Output only. The reason why the model stopped generating tokens.
    pub finish_reason: Option<FinishReason>,

    /// List of ratings for the safety of a response candidate.
    pub safety_ratings: Option<Vec<SafetyRating>>,

    /// Output only. Citation information for model-generated candidate.
    pub citation_metadata: Option<CitationMetadata>,

    /// Output only. Token count for this candidate.
    pub token_count: Option<u32>,

    /// Output only. Attribution information for sources that contributed to a grounded answer.
    pub grounding_attributions: Option<Vec<GroundingAttribution>>,

    /// Output only. Grounding metadata for the candidate.
    pub grounding_metadata: Option<GroundingMetadata>,

    /// Output only. Average log probability score of the candidate.
    pub avg_logprobs: Option<f64>,

    /// Output only. Log-likelihood scores for the response tokens and top tokens.
    pub logprobs_result: Option<LogprobsResult>,

    /// Output only. Metadata related to url context retrieval tool.
    pub url_context_metadata: Option<UrlContextMetadata>,

    /// Output only. Index of the candidate in the list of response candidates.
    pub index: Option<u32>,

    /// Optional. Output only. Details the reason why the model stopped generating tokens.
    pub finish_message: Option<String>,
}
