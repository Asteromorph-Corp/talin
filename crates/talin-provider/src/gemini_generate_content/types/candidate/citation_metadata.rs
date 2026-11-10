use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A collection of source attributions for a piece of content.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CitationMetadata {
    /// Citations to sources for a specific response.
    pub citation_sources: Option<Vec<CitationSource>>,
}

/// A citation to a source for a portion of a specific response.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CitationSource {
    /// Optional. Start of segment of the response that is attributed to this source.
    pub start_index: Option<u32>,

    /// Optional. End of the attributed segment, exclusive.
    pub end_index: Option<u32>,

    /// Optional. URI that is attributed as a source for a portion of the text.
    pub uri: Option<String>,

    /// Optional. License for the GitHub project that is attributed as a source for segment.
    pub license: Option<String>,
}
