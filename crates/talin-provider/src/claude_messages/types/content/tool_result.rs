use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{
    BrowserStateBlockParam, CacheControlEphemeral, DocumentBlockParam, ImageBlockParam,
    SearchResultBlockParam, TextBlockParam, TextOrItems, ToolReferenceBlockParam,
};
use crate::utils::tagged_enum;

/// A tool result block within a request message.
///
/// Used to provide the results of a user-defined tool invocation.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolResultBlockParam {
    /// The id of the tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    #[cfg_attr(
        test,
        schema(
            nullable = false,
            schema_with = crate::claude_messages::spec::schema::text_or_tool_result_block_param_contents_schema,
        )
    )]
    pub content: Option<TextOrItems<ToolResultBlockParamContent>>,

    /// Whether the tool execution resulted in an error.
    #[cfg_attr(test, schema(nullable = false))]
    pub is_error: Option<bool>,

    /// For a toolset member tool result, the toolset family of the paired tool use.
    pub toolset_name: Option<String>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

tagged_enum! {
    /// A content block within a tool result block.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ToolResultBlockParamContent {
        /// A text block.
        Text(TextBlockParam),
        /// An image block.
        Image(ImageBlockParam),
        /// A search result block.
        SearchResult(SearchResultBlockParam),
        /// A document block.
        Document(DocumentBlockParam),
        /// A tool reference block.
        ToolReference(ToolReferenceBlockParam),
        /// A browser state block, describing the caller's browser after a browser toolset member call.
        BrowserState(BrowserStateBlockParam),
    }
}

/// An MCP tool result block within a request message.
#[cfg(feature = "mcp-client-2025-11-20")]
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct McpToolResultBlockParam {
    /// The id of the tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: Option<TextOrItems<McpToolResultContentItem>>,

    /// Whether the tool execution resulted in an error.
    pub is_error: Option<bool>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

tagged_enum! {
    /// A content block within an MCP tool result block. Can only be a text block.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum McpToolResultContentItem {
        /// A text block.
        Text(TextBlockParam),
    }
}

impl ToolResultBlockParam {
    /// Creates a new [`ToolResultBlockParam`] from the given tool use id.
    pub fn new(tool_use_id: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: None,
            is_error: None,
            toolset_name: None,
            cache_control: None,
        }
    }

    /// Sets the [`content`](Self::content) of the tool result.
    pub fn with_content(mut self, content: TextOrItems<ToolResultBlockParamContent>) -> Self {
        self.content = Some(content);
        self
    }

    /// Sets whether the tool execution resulted in an [`error`](Self::is_error).
    pub fn with_is_error(mut self, is_error: bool) -> Self {
        self.is_error = Some(is_error);
        self
    }

    /// Sets the [`toolset_name`](Self::toolset_name) of the paired tool use.
    pub fn with_toolset_name(mut self, toolset_name: impl Into<String>) -> Self {
        self.toolset_name = Some(toolset_name.into());
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

#[cfg(feature = "mcp-client-2025-11-20")]
impl McpToolResultBlockParam {
    /// Creates a new [`McpToolResultBlockParam`] from the given tool use id.
    pub fn new(tool_use_id: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: None,
            is_error: None,
            cache_control: None,
        }
    }

    /// Sets the [`content`](Self::content) of the tool result.
    pub fn with_content(mut self, content: TextOrItems<McpToolResultContentItem>) -> Self {
        self.content = Some(content);
        self
    }

    /// Sets whether the tool execution resulted in an [`error`](Self::is_error).
    pub fn with_is_error(mut self, is_error: bool) -> Self {
        self.is_error = Some(is_error);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}
