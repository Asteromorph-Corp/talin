use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::pair::{paired_block, paired_content};
use crate::claude_messages::types::{
    CacheControlEphemeral, DocumentBlock, DocumentBlockParam, ToolCaller,
};
use crate::utils::string_enum;

/// A web fetch tool result block within a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebFetchToolResultBlock {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: WebFetchToolResultBlockContent,

    /// What invoked the tool.
    pub caller: ToolCaller,
}

/// A web fetch tool result block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebFetchToolResultBlockParam {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: WebFetchToolResultBlockParamContent,

    /// What invoked the tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub caller: Option<ToolCaller>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

paired_content! {
    /// The content of a result from Claude's internal web fetch tool.
    WebFetchToolResultBlockContent as "WebFetchToolResultBlockContent"
        | WebFetchToolResultBlockParamContent
            as "WebFetchToolResultBlockParamContent"
    {
        /// An error that occurred during the web fetch.
        WebFetchToolResultError(
            WebFetchToolResultErrorBlock | WebFetchToolResultErrorBlockParam
        ),

        /// A successful web fetch result.
        WebFetchResult(WebFetchBlock | WebFetchBlockParam),
    }
}

paired_block! {
    /// A successful web fetch result block.
    WebFetchBlock | WebFetchBlockParam {
        /// The fetched document.
        content: WebFetchResultBlockContent | WebFetchResultBlockParamContent,

        /// The fetched content URL.
        url: String,

        /// ISO 8601 timestamp when the content was retrieved.
        retrieved_at: Option<String> with with_retrieved_at,
    }
}

paired_content! {
    /// The content of a successful web fetch result. Can only be a document block.
    WebFetchResultBlockContent as "WebFetchResultBlockContent"
        | WebFetchResultBlockParamContent
            as "WebFetchResultBlockParamContent"
    {
        /// A document block.
        Document(DocumentBlock | DocumentBlockParam),
    }
}

paired_block! {
    /// An error that occurred during the web fetch.
    WebFetchToolResultErrorBlock | WebFetchToolResultErrorBlockParam {
        /// The error code.
        error_code: WebFetchToolResultErrorCode,
    }
}

string_enum! {
    /// An error code from the web fetch tool.
    #[serde(rename_all = "snake_case")]
    pub enum WebFetchToolResultErrorCode {
        /// `"invalid_tool_input"`
        InvalidToolInput,

        /// `"url_too_long"`
        UrlTooLong,

        /// `"url_not_allowed"`
        UrlNotAllowed,

        /// `"url_not_in_prior_context"`
        UrlNotInPriorContext,

        /// `"url_not_accessible"`
        UrlNotAccessible,

        /// `"unsupported_content_type"`
        UnsupportedContentType,

        /// `"too_many_requests"`
        TooManyRequests,

        /// `"max_uses_exceeded"`
        MaxUsesExceeded,

        /// `"unavailable"`
        Unavailable,

        /// `"content_too_large"`
        ContentTooLarge,
    }
}

impl WebFetchToolResultBlock {
    /// Creates a new [`WebFetchToolResultBlock`] from the given tool use id and content.
    pub fn new(tool_use_id: impl Into<String>, content: WebFetchToolResultBlockContent) -> Self {
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

impl WebFetchToolResultBlockParam {
    /// Creates a new [`WebFetchToolResultBlockParam`] from the given tool use id and content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: WebFetchToolResultBlockParamContent,
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

impl From<WebFetchToolResultBlock> for WebFetchToolResultBlockParam {
    fn from(value: WebFetchToolResultBlock) -> Self {
        Self {
            tool_use_id: value.tool_use_id,
            content: value.content.into(),
            caller: Some(value.caller),
            cache_control: None,
        }
    }
}
