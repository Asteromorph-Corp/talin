use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::pair::{paired_block, paired_content};
use crate::claude_messages::types::{
    CacheControlEphemeral, ToolSearchResultContentBlock, ToolSearchResultContentBlockParam,
};
use crate::utils::string_enum;

/// A tool search tool result block within a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolSearchToolResultBlock {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: ToolSearchToolResultBlockContent,
}

/// A tool search tool result block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolSearchToolResultBlockParam {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: ToolSearchToolResultBlockParamContent,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

paired_content! {
    /// The content of a result from Claude's internal tool search tool.
    ToolSearchToolResultBlockContent as "ToolSearchToolResultBlockContent"
        | ToolSearchToolResultBlockParamContent
            as "ToolSearchToolResultBlockParamContent"
    {
        /// An error that occurred during the tool search.
        ToolSearchToolResultError(
            ToolSearchToolResultError | ToolSearchToolResultErrorParam
        ),

        /// A successful tool search result.
        ToolSearchToolSearchResult(
            ToolSearchToolSearchResultBlock | ToolSearchToolSearchResultBlockParam
        ),
    }
}

paired_block! {
    /// A successful tool search result block.
    ToolSearchToolSearchResultBlock | ToolSearchToolSearchResultBlockParam {
        /// The tools the search matched.
        tool_references: Vec<ToolSearchResultContentBlock>
                       | Vec<ToolSearchResultContentBlockParam>,
    }
}

paired_block! {
    /// An error that occurred during a tool search.
    ToolSearchToolResultError | ToolSearchToolResultErrorParam {
        /// The error code.
        error_code: ToolSearchToolResultErrorCode,

        /// The error message.
        error_message: Option<String> with with_error_message,
    }
}

string_enum! {
    /// An error code from the tool search tool.
    #[serde(rename_all = "snake_case")]
    pub enum ToolSearchToolResultErrorCode {
        /// `"invalid_tool_input"`
        InvalidToolInput,

        /// `"unavailable"`
        Unavailable,

        /// `"too_many_requests"`
        TooManyRequests,

        /// `"execution_time_exceeded"`
        ExecutionTimeExceeded,
    }
}

impl ToolSearchToolResultBlock {
    /// Creates a new [`ToolSearchToolResultBlock`] from the given tool use id and content.
    pub fn new(tool_use_id: impl Into<String>, content: ToolSearchToolResultBlockContent) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
        }
    }
}

impl ToolSearchToolResultBlockParam {
    /// Creates a new [`ToolSearchToolResultBlockParam`] from the given tool use id and content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: ToolSearchToolResultBlockParamContent,
    ) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
            cache_control: None,
        }
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl From<ToolSearchToolResultBlock> for ToolSearchToolResultBlockParam {
    fn from(value: ToolSearchToolResultBlock) -> Self {
        Self {
            tool_use_id: value.tool_use_id,
            content: value.content.into(),
            cache_control: None,
        }
    }
}
