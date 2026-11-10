use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Grounding support.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GroundingSupport {
    /// A list of indices (into 'grounding_chunk') specifying the citations.
    pub grounding_chunk_indices: Option<Vec<i32>>,

    /// Confidence score of the support references. Ranges from 0 to 1.
    pub confidence_scores: Option<Vec<f64>>,

    /// Segment of the content this support belongs to.
    pub segment: Option<Segment>,
}

/// Segment of the content.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Segment {
    /// Output only. The index of a Part object within its parent Content object.
    pub part_index: Option<u32>,

    /// Output only. Start index in the given Part, measured in bytes.
    pub start_index: Option<u32>,

    /// Output only. End index in the given Part, measured in bytes.
    pub end_index: Option<u32>,

    /// Output only. The text corresponding to the segment from the response.
    pub text: Option<String>,
}
