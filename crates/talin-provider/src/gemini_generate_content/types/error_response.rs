use serde::{Deserialize, Serialize};

/// Wrapper for API error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ApiErrorResponse {
    /// The error details.
    pub error: ApiErrorDetails,
}

/// Error details returned by the Gemini API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ApiErrorDetails {
    /// The error code (e.g., 400, 429, 500).
    #[serde(default)]
    pub code: Option<i32>,

    /// A human-readable error message.
    #[serde(default)]
    pub message: Option<String>,

    /// The error status (e.g., "INVALID_ARGUMENT", "RESOURCE_EXHAUSTED").
    #[serde(default)]
    pub status: Option<String>,

    /// Additional details about the error.
    #[serde(default)]
    pub details: Option<Vec<serde_json::Value>>,
}

impl std::fmt::Display for ApiErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_deref().unwrap_or("Unknown error"))
    }
}
