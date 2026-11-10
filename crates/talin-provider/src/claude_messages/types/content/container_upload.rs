use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::CacheControlEphemeral;

/// A file uploaded to the container, as returned in a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContainerUploadBlock {
    /// The id of the uploaded file.
    pub file_id: String,
}

/// A file to be uploaded to the container.
///
/// Files uploaded through this block are available in the container's input directory.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContainerUploadBlockParam {
    /// The id of the uploaded file.
    pub file_id: String,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

impl ContainerUploadBlock {
    /// Creates a new [`ContainerUploadBlock`] from the id of an uploaded file.
    pub fn new(file_id: impl Into<String>) -> Self {
        Self {
            file_id: file_id.into(),
        }
    }
}

impl ContainerUploadBlockParam {
    /// Creates a new [`ContainerUploadBlockParam`] from the id of an uploaded file.
    pub fn new(file_id: impl Into<String>) -> Self {
        Self {
            file_id: file_id.into(),
            cache_control: None,
        }
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl From<ContainerUploadBlock> for ContainerUploadBlockParam {
    fn from(value: ContainerUploadBlock) -> Self {
        Self {
            file_id: value.file_id,
            cache_control: None,
        }
    }
}
