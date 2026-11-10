use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use crate::utils::tagged_enum;

tagged_enum! {
    /// An MCP server definition.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum McpServer {
        /// An MCP server accessible via a URL.
        Url(McpServerUrl),
    }
}

/// An MCP server definition for a server accessible via a URL.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct McpServerUrl {
    /// The name of the MCP server.
    pub name: String,

    /// The URL of the MCP server.
    pub url: String,

    /// An authorization token for accessing the MCP server.
    pub authorization_token: Option<String>,

    /// Configuration for the tools provided by the MCP server.
    pub tool_configuration: Option<McpToolConfig>,
}

/// Configuration for the tools provided by an MCP server.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct McpToolConfig {
    /// List of allowed tool names. If not provided, all tools are allowed.
    pub allowed_tools: Option<Vec<String>>,

    /// Whether the tools from this MCP server are enabled.
    pub enabled: Option<bool>,
}

impl McpServerUrl {
    /// Creates a new [`McpServerUrl`] from the given name and URL.
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            authorization_token: None,
            tool_configuration: None,
        }
    }

    /// Sets the [`authorization_token`](Self::authorization_token) for the MCP server.
    pub fn with_authorization_token(mut self, authorization_token: impl Into<String>) -> Self {
        self.authorization_token = Some(authorization_token.into());
        self
    }

    /// Sets the [`tool_configuration`](Self::tool_configuration) for the MCP server.
    pub fn with_tool_configuration(mut self, tool_configuration: McpToolConfig) -> Self {
        self.tool_configuration = Some(tool_configuration);
        self
    }
}

impl McpToolConfig {
    /// Creates a new [`McpToolConfig`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_tools`](Self::allowed_tools) for the MCP server.
    pub fn with_allowed_tools(mut self, allowed_tools: Vec<String>) -> Self {
        self.allowed_tools = Some(allowed_tools);
        self
    }

    /// Sets whether the tools from this MCP server are [`enabled`](Self::enabled).
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
}
