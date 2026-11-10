use rmcp::ServiceError;
use rmcp::model::ErrorCode;
use rmcp::service::ClientInitializeError;
use thiserror::Error;

use crate::tools::ToolError;

/// Errors that can occur during MCP operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum McpError {
    /// Failed to initialize the MCP client.
    #[error("failed to initialize MCP client: {0}")]
    ClientInitialize(#[source] Box<ClientInitializeError>),

    /// Failed to spawn the MCP server's child process (stdio transport).
    #[error("failed to spawn MCP server process: {0}")]
    Spawn(#[source] std::io::Error),

    /// An error from the underlying MCP service.
    #[error(transparent)]
    Service(Box<rmcp::ServiceError>),

    /// Failed to deserialize the response from the MCP server.
    #[error("failed to deserialize response: {0}")]
    ResponseDeserialize(#[from] serde_json::Error),

    /// The MCP server returned an invalid response (e.g. invalid base64 content).
    #[error("invalid content: {0}")]
    InvalidContent(String),

    /// The requested tool does not exist in the MCP server.
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    /// The server returned a multi-round-trip response requiring client input.
    #[error("MCP server required client input")]
    InputRequired,
}

impl From<ClientInitializeError> for McpError {
    fn from(error: ClientInitializeError) -> Self {
        McpError::ClientInitialize(Box::new(error))
    }
}

impl From<rmcp::ServiceError> for McpError {
    fn from(error: rmcp::ServiceError) -> Self {
        McpError::Service(Box::new(error))
    }
}

impl McpError {
    /// Constructs a new [`McpError::InvalidContent`] with the given message.
    pub fn invalid_content(message: impl Into<String>) -> Self {
        McpError::InvalidContent(message.into())
    }
}

impl From<McpError> for ToolError {
    fn from(error: McpError) -> Self {
        match error {
            McpError::Service(e) => ToolError::from(*e),
            McpError::ToolNotFound(tool) => ToolError::ToolNotFound(tool),
            other => ToolError::Mcp(Box::new(other)),
        }
    }
}

impl From<rmcp::ServiceError> for ToolError {
    fn from(error: rmcp::ServiceError) -> Self {
        match error {
            ServiceError::McpError(data)
                if matches!(
                    data.code,
                    ErrorCode::METHOD_NOT_FOUND | ErrorCode::RESOURCE_NOT_FOUND
                ) =>
            {
                ToolError::ToolNotFound(data.message.into_owned())
            }
            ServiceError::McpError(data) => ToolError::Execution(data.message.into()),
            error @ (ServiceError::Timeout { .. } | ServiceError::Cancelled { .. }) => {
                ToolError::execution(error)
            }
            error => ToolError::Mcp(Box::new(McpError::from(error))),
        }
    }
}
