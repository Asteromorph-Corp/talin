use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::tagged_enum;

tagged_enum! {
    /// How the model should use the provided tools.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ToolChoice {
        /// Automatically decide by itself whether to use tools.
        Auto(ParallelToolUse),
        /// Must use any available tool at least once.
        Any(ParallelToolUse),
        /// Must use the specified tool.
        Tool(ToolChoiceTool),
        /// Must not use any tools.
        None,
    }
}

/// Configuration for parallel tool use.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct ParallelToolUse {
    /// Whether to disable parallel tool use.
    #[cfg_attr(test, schema(nullable = false))]
    pub disable_parallel_tool_use: Option<bool>,
}

/// Tool choice specifying a particular tool.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolChoiceTool {
    /// The name of the tool to use.
    pub name: String,

    /// Whether to disable parallel tool use.
    #[cfg_attr(test, schema(nullable = false))]
    pub disable_parallel_tool_use: Option<bool>,
}

impl ToolChoice {
    /// Creates a new [`ToolChoice`] that lets the model decide whether to use tools.
    pub const fn auto() -> Self {
        ToolChoice::Auto(ParallelToolUse::new())
    }

    /// Creates a new [`ToolChoice`] that requires using any available tool.
    pub const fn any() -> Self {
        ToolChoice::Any(ParallelToolUse::new())
    }

    /// Creates a new [`ToolChoice`] that requires using the specified tool.
    pub fn tool(name: impl Into<String>) -> Self {
        ToolChoice::Tool(ToolChoiceTool::new(name))
    }

    /// Creates a new [`ToolChoice`] that requires not using any tools.
    pub const fn none() -> Self {
        ToolChoice::None
    }

    /// Sets whether to disable parallel tool use.
    ///
    /// Has no effect on [`ToolChoice::None`].
    pub const fn with_disable_parallel_tool_use(mut self, disabled: bool) -> Self {
        match &mut self {
            ToolChoice::Auto(config) | ToolChoice::Any(config) => {
                config.disable_parallel_tool_use = Some(disabled);
            }
            ToolChoice::Tool(tool) => {
                tool.disable_parallel_tool_use = Some(disabled);
            }
            _ => (),
        }
        self
    }
}

impl ParallelToolUse {
    /// Creates a new [`ParallelToolUse`].
    pub const fn new() -> Self {
        ParallelToolUse {
            disable_parallel_tool_use: None,
        }
    }

    /// Creates a new [`ParallelToolUse`] allowing parallel tool use.
    pub const fn allowed() -> Self {
        ParallelToolUse {
            disable_parallel_tool_use: Some(false),
        }
    }

    /// Creates a new [`ParallelToolUse`] disabling parallel tool use.
    pub const fn disabled() -> Self {
        ParallelToolUse {
            disable_parallel_tool_use: Some(true),
        }
    }

    /// Sets [`disable_parallel_tool_use`](Self::disable_parallel_tool_use).
    pub const fn with_disable_parallel_tool_use(mut self, disable: bool) -> Self {
        self.disable_parallel_tool_use = Some(disable);
        self
    }
}

impl ToolChoiceTool {
    /// Creates a new [`ToolChoiceTool`] with the given tool name.
    pub fn new(name: impl Into<String>) -> Self {
        ToolChoiceTool {
            name: name.into(),
            disable_parallel_tool_use: None,
        }
    }

    /// Sets [`disable_parallel_tool_use`](Self::disable_parallel_tool_use).
    pub fn with_disable_parallel_tool_use(mut self, disable: bool) -> Self {
        self.disable_parallel_tool_use = Some(disable);
        self
    }
}

impl Default for ToolChoice {
    fn default() -> Self {
        Self::auto()
    }
}
