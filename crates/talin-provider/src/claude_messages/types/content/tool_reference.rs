use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::CacheControlEphemeral;
use crate::utils::tagged_enum;

/// A reference to a tool, as returned by the tool search tool.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolReferenceBlock {
    /// The name of the referenced tool.
    pub tool_name: String,
}

/// A reference to a tool, for inclusion in the content of a tool result block.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolReferenceBlockParam {
    /// The name of the referenced tool.
    pub tool_name: String,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

tagged_enum! {
    /// A content block within a tool search result. Can only be a tool reference block.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ToolSearchResultContentBlock {
        /// A tool reference block.
        ToolReference(ToolReferenceBlock),
    }
}

tagged_enum! {
    /// A content block within a tool search result passed back in a request. Can only be a tool
    /// reference block.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ToolSearchResultContentBlockParam {
        /// A tool reference block.
        ToolReference(ToolReferenceBlockParam),
    }
}

impl ToolReferenceBlock {
    /// Creates a new [`ToolReferenceBlock`] from the given tool name.
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
        }
    }
}

impl ToolReferenceBlockParam {
    /// Creates a new [`ToolReferenceBlockParam`] from the given tool name.
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            cache_control: None,
        }
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl From<ToolReferenceBlock> for ToolReferenceBlockParam {
    fn from(value: ToolReferenceBlock) -> Self {
        ToolReferenceBlockParam::new(value.tool_name)
    }
}

impl From<ToolSearchResultContentBlock> for ToolSearchResultContentBlockParam {
    fn from(value: ToolSearchResultContentBlock) -> Self {
        match value {
            ToolSearchResultContentBlock::ToolReference(reference) => {
                ToolSearchResultContentBlockParam::ToolReference(reference.into())
            }
            ToolSearchResultContentBlock::Unknown(value) => {
                ToolSearchResultContentBlockParam::Unknown(value)
            }
        }
    }
}
