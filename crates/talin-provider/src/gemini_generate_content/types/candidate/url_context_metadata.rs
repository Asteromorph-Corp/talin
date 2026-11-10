use serde::{Deserialize, Serialize};

/// Metadata related to url context retrieval tool.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UrlContextMetadata {
    /// List of url context.
    pub url_metadata: Vec<UrlMetadata>,
}

/// Context of the a single url retrieval.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UrlMetadata {
    /// Retrieved url by the tool.
    pub retrieved_url: String,
    /// Status of the url retrieval.
    pub url_retrieval_status: Option<UrlRetrievalStatus>,
}

/// Status of the url retrieval.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum UrlRetrievalStatus {
    /// Default value. This value is unused.
    UrlRetrievalStatusUnspecified,
    /// Url retrieval is successful.
    UrlRetrievalStatusSuccess,
    /// Url retrieval is failed due to error.
    UrlRetrievalStatusError,
    /// Url retrieval is failed because the content is behind paywall.
    UrlRetrievalStatusPaywall,
    /// Url retrieval is failed because the content is unsafe.
    UrlRetrievalStatusUnsafe,
}
