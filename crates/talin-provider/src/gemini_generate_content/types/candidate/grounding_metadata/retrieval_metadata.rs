use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Metadata related to retrieval in the grounding flow.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RetrievalMetadata {
    /// Optional. Score indicating how likely information from google search could help.
    pub google_search_dynamic_retrieval_score: Option<f64>,
}
