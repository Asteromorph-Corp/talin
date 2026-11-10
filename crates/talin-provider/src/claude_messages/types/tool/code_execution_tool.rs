use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{AllowedCaller, CacheControlEphemeral};
use crate::utils::fixed_value;

/// The code execution tool (2025-05-22).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct CodeExecutionTool20250522 {
    /// The name of the tool, must be `"code_execution"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: CodeExecutionToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,
}

/// The code execution tool (2025-08-25).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct CodeExecutionTool20250825 {
    /// The name of the tool, must be `"code_execution"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: CodeExecutionToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,
}

/// The code execution tool (2026-01-20).
///
/// Provides REPL state persistence via daemon mode and gVisor checkpoints.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct CodeExecutionTool20260120 {
    /// The name of the tool, must be `"code_execution"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: CodeExecutionToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,
}

/// The code execution tool (2026-05-21).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct CodeExecutionTool20260521 {
    /// The name of the tool, must be `"code_execution"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: CodeExecutionToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,
}

impl CodeExecutionTool20250522 {
    /// Creates a new [`CodeExecutionTool20250522`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
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
}

impl CodeExecutionTool20250825 {
    /// Creates a new [`CodeExecutionTool20250825`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
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
}

impl CodeExecutionTool20260120 {
    /// Creates a new [`CodeExecutionTool20260120`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
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
}

impl CodeExecutionTool20260521 {
    /// Creates a new [`CodeExecutionTool20260521`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
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
}

fixed_value!(CodeExecutionToolName::CodeExecution);
