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
    /// The name of the MCP server. Must be referenced by exactly one
    /// [`McpToolset`](crate::claude_messages::types::McpToolset) in
    /// [`tools`](crate::claude_messages::types::MessagesRequest::tools).
    pub name: String,

    /// The URL of the MCP server. Must start with `https://`.
    pub url: String,

    /// An OAuth authorization token for accessing the MCP server.
    pub authorization_token: Option<String>,
}

impl McpServerUrl {
    /// Creates a new [`McpServerUrl`] from the given name and URL.
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            authorization_token: None,
        }
    }

    /// Sets the [`authorization_token`](Self::authorization_token) for the MCP server.
    pub fn with_authorization_token(mut self, authorization_token: impl Into<String>) -> Self {
        self.authorization_token = Some(authorization_token.into());
        self
    }
}
