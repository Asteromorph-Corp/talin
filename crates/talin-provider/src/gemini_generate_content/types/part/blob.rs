use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

/// Raw media bytes with MIME type.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Blob {
    /// The MIME type of the data.
    pub mime_type: String,

    /// Raw bytes of the data (base64-encoded when serialized).
    #[serde_as(as = "Base64")]
    pub data: Vec<u8>,
}

impl Blob {
    /// Creates a new [`Blob`].
    pub fn new(mime_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            mime_type: mime_type.into(),
            data,
        }
    }
}
