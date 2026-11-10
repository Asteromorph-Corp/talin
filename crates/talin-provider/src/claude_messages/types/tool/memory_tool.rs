use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{AllowedCaller, CacheControlEphemeral};
use crate::utils::fixed_value;

/// The memory tool (2025-08-18).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct MemoryTool20250818 {
    /// The name of the tool, must be `"memory"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: MemoryToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Examples of valid inputs for this tool.
    #[cfg_attr(
        test,
        schema(schema_with = crate::claude_messages::spec::schema::object_array_schema)
    )]
    pub input_examples: Option<Vec<serde_json::Value>>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

impl MemoryTool20250818 {
    /// Creates a new [`MemoryTool20250818`].
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

    /// Sets the [`input_examples`](Self::input_examples) of valid inputs for this tool.
    pub fn with_input_examples(mut self, input_examples: Vec<serde_json::Value>) -> Self {
        self.input_examples = Some(input_examples);
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

fixed_value!(MemoryToolName::Memory);
