//! A module for tools used by LLM agents.
//!
//! This module provides the infrastructure for creating and using tools with LLM agents.
//! Tools allow agents to interact with external systems, perform computations, and
//! access information beyond their training data.
//!
//! # Overview
//!
//! The tools system consists of several key components:
//!
//! - **[`Function`]** - The core trait for implementing custom tools
//! - **[`FunctionTool`]** - A dyn-compatible interface for tools (auto-implemented for `Function`)
//! - **[`Tool`]** - A function tool that can be registered with an agent
//! - **[`Toolbox`]** - A trait for managing collections of tools
//!
//! MCP (Model Context Protocol) servers are not tools. See the [`mcp`](crate::mcp) module.
//!
//! # Creating Custom Tools
//!
//! Implement the [`Function`] trait to create a custom tool:
//!
//! ```rust
//! use async_trait::async_trait;
//! use schemars::JsonSchema;
//! use serde::Deserialize;
//! use talin::tools::Function;
//!
//! #[derive(Deserialize, JsonSchema)]
//! struct WeatherInput {
//!     city: String,
//! }
//!
//! struct WeatherTool;
//!
//! #[async_trait]
//! impl Function for WeatherTool {
//!     type Input = WeatherInput;
//!     type Output = String;
//!
//!     fn name(&self) -> String {
//!         "get_weather".into()
//!     }
//!     fn description(&self) -> String {
//!         "Get current weather for a city".into()
//!     }
//!
//!     async fn call(
//!         &self,
//!         input: Self::Input,
//!     ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
//!         Ok(format!("Weather in {}: Sunny, 72°F", input.city))
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

mod error;
mod function;
mod name;
mod resolver;
mod tool_ref;
mod tool_spec;
mod toolbox;

pub mod config;

pub use config::{ConfigSchema, FromConfig};
pub use error::{ToolError, ToolErrorFeedback};
pub(crate) use function::sealed;
pub use function::{
    DefaultFunctionInput, EmptyFunctionInput, Function, FunctionSpec, FunctionTool,
};
pub(crate) use name::normalize_tool_name;
pub use resolver::ToolResolver;
pub(crate) use tool_ref::AsToolRef;
pub use tool_ref::ToolRef;
pub use tool_spec::ToolSpec;
pub use toolbox::{ListTools, SimpleToolbox, Toolbox};

/// A tool that can be used by an LLM agent.
///
/// # Variants
///
/// - `Function` - A tool implemented via the [`FunctionTool`] trait
///
/// # Example
///
/// ```rust,ignore
/// use talin::tools::{Tool, DuckDuckGoSearch};
///
/// let search_tool: Tool = DuckDuckGoSearch::default().into();
///
/// // Build an index for quick lookup
/// let tools = Tool::build_index(vec![search_tool]);
/// ```
#[derive(Clone)]
#[non_exhaustive]
pub enum Tool {
    /// A function tool.
    Function(Arc<dyn FunctionTool>),
}

/// Extension trait for tools.
pub trait ToolExt: AsToolRef {
    /// Get the name of this tool.
    fn name(&self) -> String {
        match self.as_tool_ref() {
            ToolRef::Function(func) => FunctionTool::name(func),
        }
    }

    /// Get the [`ToolSpec`] for this tool.
    fn get_spec(&self) -> ToolSpec {
        match self.as_tool_ref() {
            ToolRef::Function(f) => ToolSpec::Function(f.get_spec()),
        }
    }

    /// Return the tool as a [`FunctionTool`].
    fn as_function(&self) -> Option<&dyn FunctionTool> {
        match self.as_tool_ref() {
            ToolRef::Function(func) => Some(func),
        }
    }
}
impl<T: AsToolRef> ToolExt for T {}

impl Tool {
    /// Builds a lookup index from a collection of tools.
    ///
    /// The index uses normalized tool names as keys, allowing for efficient
    /// tool lookup by name. This is useful for agent implementations that
    /// need to quickly find and execute tools based on LLM outputs.
    pub fn build_index(tools: Vec<Self>) -> HashMap<String, Self> {
        tools
            .into_iter()
            .map(|tool| (normalize_tool_name(&tool.name()), tool))
            .collect()
    }
}

impl<F: FunctionTool + 'static> From<F> for Tool {
    fn from(tool: F) -> Self {
        Tool::Function(Arc::new(tool))
    }
}
