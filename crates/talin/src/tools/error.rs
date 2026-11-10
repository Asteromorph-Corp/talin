use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::mcp::McpError;

/// Errors that can occur during tool execution.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ToolError {
    /// The tool reported a failure.
    #[error("error while running tool: {0}")]
    Execution(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Failed to deserialize model-provided input for the tool.
    #[error("failed to deserialize tool input: {0}")]
    InputDeserialize(#[source] serde_json::Error),

    /// The tool ran, but its result could not be processed. (e.g. rejected by
    /// [`Strategy::process_tool_result`], provider couldn't represent the content)
    ///
    /// [`Strategy::process_tool_result`]: crate::strategy::Strategy::process_tool_result
    #[error("failed to process tool output: {0}")]
    OutputProcessing(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// The requested tool does not exist.
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    /// The tool is currently unavailable.
    #[error("tool is unavailable")]
    Unavailable,

    /// An MCP transport or protocol failure.
    #[error("MCP error: {0}")]
    Mcp(#[source] Box<McpError>),
}

/// The feedback string for the tool error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToolErrorFeedback {
    /// The tool failed while running, with the error message.
    Execution(String),
    /// The model-provided arguments were invalid, with the error message.
    InvalidArguments(String),
    /// The requested tool does not exist.
    NoSuchTool,
    /// The tool is currently unavailable.
    Unavailable,
}

impl ToolError {
    /// Creates a new [`ToolError::Execution`] from any error type.
    pub fn execution<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        ToolError::Execution(Box::new(error))
    }

    /// Returns a feedback string to return to the model.
    pub fn feedback(&self) -> ToolErrorFeedback {
        match self {
            ToolError::Execution(e) => ToolErrorFeedback::Execution(e.to_string()),
            ToolError::OutputProcessing(e) => ToolErrorFeedback::Execution(format!(
                "the tool ran, but its result could not be processed: {e}"
            )),
            ToolError::InputDeserialize(e) => ToolErrorFeedback::InvalidArguments(e.to_string()),
            ToolError::ToolNotFound(_) => ToolErrorFeedback::NoSuchTool,
            ToolError::Unavailable => ToolErrorFeedback::Unavailable,
            ToolError::Mcp(_) => ToolErrorFeedback::Unavailable,
        }
    }
}

impl std::fmt::Display for ToolErrorFeedback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolErrorFeedback::Execution(e) => write!(f, "{e}"),
            ToolErrorFeedback::InvalidArguments(e) => write!(f, "invalid arguments: {e}"),
            ToolErrorFeedback::NoSuchTool => write!(f, "no such tool"),
            ToolErrorFeedback::Unavailable => write!(f, "the tool is unavailable"),
        }
    }
}
