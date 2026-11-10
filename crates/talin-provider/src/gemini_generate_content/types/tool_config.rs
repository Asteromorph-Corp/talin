use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod function_calling_config;
mod retrieval_config;

pub use function_calling_config::*;
pub use retrieval_config::*;

/// Tool configuration for any Tool specified in the request.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolConfig {
    /// Optional. Function calling config.
    pub function_calling_config: Option<FunctionCallingConfig>,

    /// Optional, Retrieval config.
    pub retrieval_config: Option<RetrievalConfig>,

    /// Optional. If true, the API response will include the server-side tool calls and responses
    /// within the Content message. This allows clients to observe the server's tool
    /// interactions.
    pub include_server_side_tool_invocations: Option<bool>,
}

impl ToolConfig {
    /// Creates a new empty ToolConfig.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the function calling configuration.
    pub fn with_function_calling_config(mut self, config: FunctionCallingConfig) -> Self {
        self.function_calling_config = Some(config);
        self
    }

    /// Sets the retrieval configuration.
    pub fn with_retrieval_config(mut self, config: RetrievalConfig) -> Self {
        self.retrieval_config = Some(config);
        self
    }

    /// Sets whether to include server-side tool invocations in the response.
    pub fn with_include_server_side_tool_invocations(mut self, include: bool) -> Self {
        self.include_server_side_tool_invocations = Some(include);
        self
    }
}
