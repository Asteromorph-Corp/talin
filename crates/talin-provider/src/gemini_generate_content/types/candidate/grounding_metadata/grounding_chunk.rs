use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Grounding chunk.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum GroundingChunk {
    /// Chunk from the web.
    Web(Web),

    /// Chunk from context retrieved by the file search tool.
    RetrievedContext(RetrievedContext),

    /// Chunk from Google Maps.
    Maps(Maps),
}

/// Chunk from the web.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Web {
    /// URI reference of the chunk.
    pub uri: String,

    /// Title of the chunk.
    pub title: String,
}

/// Chunk from context retrieved by the file search tool.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RetrievedContext {
    /// Optional. URI reference of the semantic retrieval document.
    pub uri: Option<String>,

    /// Optional. Title of the document.
    pub title: Option<String>,

    /// Optional. Text of the chunk.
    pub text: Option<String>,

    /// Optional. Name of the fileSearchStores containing the document. Example:
    /// `fileSearchStores/123`
    pub file_search_store: Option<String>,
}

/// Chunk from Google Maps.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Maps {
    /// URI reference of the place.
    pub uri: String,

    /// Title of the place.
    pub title: String,

    /// Text description of the place answer.
    pub text: String,

    /// This ID of the place, in places/{placeId} format. A user can use this ID to look up that
    /// place.
    pub place_id: String,

    /// Sources that provide answers about the features of a given place in Google Maps.
    pub place_answer_sources: PlaceAnswerSources,
}

/// Collection of sources that provide answers about the features of a given place in Google Maps.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlaceAnswerSources {
    /// Snippets of reviews that are used to generate answers about the features of a given place
    /// in Google Maps.
    pub review_snippets: Vec<ReviewSnippet>,
}

/// Encapsulates a snippet of a user review that answers a question about the features of a specific
/// place in Google Maps.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReviewSnippet {
    /// The ID of the review from which the snippet is extracted.
    pub review_id: String,
    /// A link that corresponds to the user review on Google Maps.
    pub google_maps_uri: String,
    /// The title of the review.
    pub title: String,
}
