use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod grounding_chunk;
mod grounding_support;
mod retrieval_metadata;
mod search_entry_point;

pub use grounding_chunk::*;
pub use grounding_support::*;
pub use retrieval_metadata::*;
pub use search_entry_point::*;

/// Metadata returned to client when grounding is enabled.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GroundingMetadata {
    /// List of supporting references retrieved from specified grounding source.
    pub grounding_chunks: Option<Vec<GroundingChunk>>,

    /// List of grounding support.
    pub grounding_supports: Option<Vec<GroundingSupport>>,

    /// Web search queries for the following-up web search.
    pub web_search_queries: Option<Vec<String>>,

    /// Optional. Google search entry for the following-up web searches.
    pub search_entry_point: Option<SearchEntryPoint>,

    /// Metadata related to retrieval in the grounding flow.
    pub retrieval_metadata: Option<RetrievalMetadata>,

    /// Optional. Token to be used for rendering Google Maps widgets.
    pub google_maps_widget_context_token: Option<String>,
}
