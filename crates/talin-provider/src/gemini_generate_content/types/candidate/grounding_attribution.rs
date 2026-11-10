use serde::{Deserialize, Serialize};

use crate::gemini_generate_content::types::Content;

/// Attribution for a source that contributed to an answer.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GroundingAttribution {
    /// Output only. Identifier for the source contributing to this attribution.
    pub source_id: AttributionSourceId,
    /// Grounding source content that makes up this attribution.
    pub content: Content,
}

/// Identifier for the source contributing to this attribution.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum AttributionSourceId {
    /// Identifier for an inline passage.
    GroundingPassage(GroundingPassageId),
    /// Identifier for a Chunk fetched via Semantic Retriever.
    SemanticRetrieverChunk(SemanticRetrieverChunk),
}

/// Identifier for a part within a GroundingPassage.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GroundingPassageId {
    /// Output only. ID of the passage matching the GenerateAnswerRequest's GroundingPassage.id.
    pub passage_id: String,
    /// Output only. Index of the part within the GenerateAnswerRequest's GroundingPassage.content
    pub part_index: u32,
}

/// Identifier for a Chunk retrieved via Semantic Retriever specified in the GenerateAnswerRequest
/// using SemanticRetrieverConfig.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticRetrieverChunk {
    /// Output only. Name of the source matching the request's SemanticRetrieverConfig.source.
    /// Example: corpora/123 or corpora/123/documents/abc
    pub source: String,
    /// Output only. Name of the Chunk containing the attributed text. Example:
    /// corpora/123/documents/abc/chunks/xyz
    pub chunk: String,
}
