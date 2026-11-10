use serde::{Deserialize, Serialize};

/// Reference to a file uploaded via the File API.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FileData {
    /// The MIME type of the file.
    pub mime_type: String,

    /// URI of the file.
    pub file_uri: String,
}

impl FileData {
    /// Creates a new [`FileData`].
    pub fn new(mime_type: impl Into<String>, file_uri: impl Into<String>) -> Self {
        Self {
            mime_type: mime_type.into(),
            file_uri: file_uri.into(),
        }
    }
}
