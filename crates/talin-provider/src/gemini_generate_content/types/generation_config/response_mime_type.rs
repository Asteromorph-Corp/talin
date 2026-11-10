use serde::{Deserialize, Serialize};

/// MIME type of the generated candidate text.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResponseMimeType {
    /// Text output.
    #[serde(rename = "text/plain")]
    TextPlain,
    /// JSON response in the response candidates.
    #[serde(rename = "application/json")]
    ApplicationJson,
    /// ENUM as a string response in the response candidates.
    #[serde(rename = "text/x.enum")]
    TextXEnum,
}
