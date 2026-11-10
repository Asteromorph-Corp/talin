use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::pair::{paired_block, paired_content};
use crate::claude_messages::types::{CacheControlEphemeral, ToolCaller};
use crate::utils::{fixed_value, string_enum};

/// A web search tool result block within a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebSearchToolResultBlock {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: WebSearchToolResultBlockContent,

    /// What invoked the tool.
    pub caller: ToolCaller,
}

/// A web search tool result block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebSearchToolResultBlockParam {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: WebSearchToolResultBlockParamContent,

    /// What invoked the tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub caller: Option<ToolCaller>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

/// The content of a web search tool result block within a response message.
///
/// The web search results are a list rather than a `type`-tagged block, so this union is
/// untagged.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum WebSearchToolResultBlockContent {
    /// An error that occurred during the web search.
    Error(WebSearchToolResultError),

    /// The search results.
    Results(Vec<WebSearchResultContentItem>),
}

/// The content of a web search tool result block within a request message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum WebSearchToolResultBlockParamContent {
    /// The search results.
    Results(Vec<WebSearchResultContentItemParam>),

    /// An error that occurred during the web search.
    Error(WebSearchToolRequestError),
}

paired_content! {
    /// A content block within a result from Claude's internal web search tool.
    WebSearchResultContentItem as "WebSearchResultContentItem"
        | WebSearchResultContentItemParam
            as "WebSearchResultContentItemParam"
    {
        /// A successful web search result.
        WebSearchResult(WebSearchResultBlock | WebSearchResultBlockParam),
    }
}

paired_block! {
    /// A successful web search result.
    WebSearchResultBlock | WebSearchResultBlockParam {
        /// The title of the web search result.
        title: String,

        /// The encrypted content of the web search result.
        encrypted_content: String,

        /// The URL of the web search result.
        url: String,

        /// The age of the page, if known.
        page_age: Option<String> with with_page_age,
    }
}

paired_block! {
    /// An error that occurred during a web search.
    WebSearchToolResultError | WebSearchToolRequestError {
        /// The type of the block.
        ///
        /// This is always `"web_search_tool_result_error"`.
        #[serde(rename = "type")]
        type_: WebSearchToolResultErrorType,

        /// The error code.
        error_code: WebSearchToolResultErrorCode,
    }
}

string_enum! {
    /// An error code from the web search tool.
    #[serde(rename_all = "snake_case")]
    pub enum WebSearchToolResultErrorCode {
        /// `"invalid_tool_input"`
        InvalidToolInput,

        /// `"unavailable"`
        Unavailable,

        /// `"max_uses_exceeded"`
        MaxUsesExceeded,

        /// `"too_many_requests"`
        TooManyRequests,

        /// `"query_too_long"`
        QueryTooLong,

        /// `"request_too_large"`
        RequestTooLarge,
    }
}

impl WebSearchToolResultBlock {
    /// Creates a new [`WebSearchToolResultBlock`] from the given tool use id and content.
    pub fn new(tool_use_id: impl Into<String>, content: WebSearchToolResultBlockContent) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
            caller: ToolCaller::direct(),
        }
    }

    /// Sets what [`caller`](Self::caller) invoked the tool.
    pub fn with_caller(mut self, caller: ToolCaller) -> Self {
        self.caller = caller;
        self
    }
}

impl WebSearchToolResultBlockParam {
    /// Creates a new [`WebSearchToolResultBlockParam`] from the given tool use id and content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: WebSearchToolResultBlockParamContent,
    ) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
            caller: None,
            cache_control: None,
        }
    }

    /// Sets what [`caller`](Self::caller) invoked the tool.
    pub fn with_caller(mut self, caller: ToolCaller) -> Self {
        self.caller = Some(caller);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl From<WebSearchToolResultBlock> for WebSearchToolResultBlockParam {
    fn from(value: WebSearchToolResultBlock) -> Self {
        Self {
            tool_use_id: value.tool_use_id,
            content: value.content.into(),
            caller: Some(value.caller),
            cache_control: None,
        }
    }
}

impl From<WebSearchToolResultBlockContent> for WebSearchToolResultBlockParamContent {
    fn from(value: WebSearchToolResultBlockContent) -> Self {
        match value {
            WebSearchToolResultBlockContent::Results(results) => {
                WebSearchToolResultBlockParamContent::Results(
                    results.into_iter().map(Into::into).collect(),
                )
            }
            WebSearchToolResultBlockContent::Error(error) => {
                WebSearchToolResultBlockParamContent::Error(error.into())
            }
        }
    }
}

fixed_value!(WebSearchToolResultErrorType::WebSearchToolResultError);
