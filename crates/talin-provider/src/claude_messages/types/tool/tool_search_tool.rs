use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{AllowedCaller, CacheControlEphemeral};
use crate::utils::fixed_value;

/// The regex tool search tool (2025-11-19).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct RegexToolSearchTool20251119 {
    /// The name of the tool, must be `"tool_search_tool_regex"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: RegexToolSearchToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

/// The BM25 tool search tool (2025-11-19).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct Bm25ToolSearchTool20251119 {
    /// The name of the tool, must be `"tool_search_tool_bm25"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: Bm25ToolSearchToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

impl RegexToolSearchTool20251119 {
    /// Creates a new [`RegexToolSearchTool20251119`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets whether to enforce [`strict`](Self::strict) input validation for this tool.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl Bm25ToolSearchTool20251119 {
    /// Creates a new [`Bm25ToolSearchTool20251119`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets whether to enforce [`strict`](Self::strict) input validation for this tool.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

fixed_value!(RegexToolSearchToolName::ToolSearchToolRegex);

fixed_value!(Bm25ToolSearchToolName::ToolSearchToolBm25);
