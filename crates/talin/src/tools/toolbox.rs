use async_trait::async_trait;
use serde_json::Value;

use crate::schemas::ToolOutput;
use crate::tools::{FunctionTool, ToolError, ToolResolver};

mod list_tools;
mod simple_toolbox;

pub use list_tools::ListTools;
pub use simple_toolbox::SimpleToolbox;

/// A collection of tools that can be accessed by name.
///
/// The `Toolbox` trait provides an interface for managing and executing a group
/// of related tools. It supports tool lookup by name and direct tool execution.
#[async_trait]
pub trait Toolbox: Send + Sync {
    /// Returns the name of this toolbox.
    fn name(&self) -> String;

    /// Returns a map of all tools in this toolbox.
    fn resolver(&self) -> Box<dyn ToolResolver<'_> + '_>;

    /// Looks up a tool by name.
    fn get_tool(&self, tool_name: &str) -> Option<&dyn FunctionTool> {
        self.resolver().resolve_tool(tool_name)
    }

    /// Executes a tool by name with the given input.
    async fn call_tool(&self, tool_name: &str, input: Value) -> Result<ToolOutput, ToolError> {
        let tool = self
            .get_tool(tool_name)
            .ok_or(ToolError::ToolNotFound(tool_name.into()))?;

        tool.call(input).await
    }
}
