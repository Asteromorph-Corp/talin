//! Model Context Protocol (MCP) servers for LLM agents.

use std::collections::HashMap;

use rmcp::model::{ClientCapabilities, ClientConfig, Implementation, ProtocolVersion};
use rmcp::service::RunningService;
use rmcp::{ClientLifecycleMode, RoleClient};
use secrecy::SecretString;

#[cfg(feature = "mcp-stdio")]
use crate::mcp::transport::StdioTransport;
use crate::mcp::transport::{HttpTransport, McpTransport};
use crate::tools::SimpleToolbox;

mod client;
mod conversion;
mod error;
mod set;
mod task;
#[cfg(test)]
pub(crate) mod test_server;
mod tool;

pub mod transport;

pub use client::McpClient;
pub use error::McpError;
pub(crate) use set::McpRouting;
pub use set::McpServerSet;
pub use tool::McpFunctionTool;

/// A running MCP service connection.
pub type McpService = RunningService<RoleClient, ClientConfig>;

/// A model context protocol (MCP) server that provides tools for LLM agents.
#[derive(Debug, Clone)]
pub struct McpServer<T = McpTransport> {
    /// How to connect to the MCP server.
    pub(crate) transport: T,
    /// The name of this MCP server.
    pub(crate) name: String,
    /// The description of the MCP server.
    pub(crate) description: Option<String>,
    /// List of allowed tool names.
    ///
    /// If not set, all tools from the server are allowed.
    pub(crate) allowed_tools: Option<Vec<String>>,
}

/// An MCP server reachable over streamable HTTP.
pub type McpServerHttp = McpServer<HttpTransport>;

/// An MCP server run as a local child process over stdio.
///
/// Always handled by a local client, since providers cannot reach a process on this machine.
#[cfg(feature = "mcp-stdio")]
pub type McpServerStdio = McpServer<StdioTransport>;

/// Identifier for the service opened for a [`McpServer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ServiceKey([u8; 32]);

impl<T> McpServer<T> {
    /// The name of this MCP server.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The description of this MCP server.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the description of this MCP server.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The allowed tool names, or `None` if every tool of the server is allowed.
    pub fn allowed_tools(&self) -> Option<&[String]> {
        self.allowed_tools.as_deref()
    }

    /// Restricts which tools are available from the MCP server.
    pub fn with_allowed_tools(mut self, allowed_tools: Vec<String>) -> Self {
        self.allowed_tools = Some(allowed_tools);
        self
    }

    fn map_transport<U>(self, f: impl FnOnce(T) -> U) -> McpServer<U> {
        McpServer {
            transport: f(self.transport),
            name: self.name,
            description: self.description,
            allowed_tools: self.allowed_tools,
        }
    }
}

impl McpServer {
    /// The identity of the service this server opens.
    pub(crate) fn service_key(&self) -> ServiceKey {
        ServiceKey::new(&self.name, &self.transport)
    }

    /// Creates a new streamable HTTP MCP server.
    #[must_use]
    pub fn http(url: impl Into<SecretString>, name: impl Into<String>) -> Self {
        Self::from(McpServerHttp::new(url, name))
    }

    /// Creates a new stdio MCP server that runs `command`.
    #[must_use]
    #[cfg(feature = "mcp-stdio")]
    pub fn stdio(command: impl Into<String>, name: impl Into<String>) -> Self {
        Self::from(McpServerStdio::new(command, name))
    }

    /// Whether this server must be handled by a local client rather than the provider.
    ///
    /// Always true for stdio servers.
    pub fn is_local(&self) -> bool {
        match &self.transport {
            McpTransport::Http(http) => http.always_local,
            #[cfg(feature = "mcp-stdio")]
            McpTransport::Stdio(_) => true,
        }
    }

    /// Returns this server as an HTTP server, or `None` for a stdio server.
    pub fn as_http(&self) -> Option<McpServerHttp> {
        match &self.transport {
            McpTransport::Http(http) => Some(McpServer {
                transport: http.clone(),
                name: self.name.clone(),
                description: self.description.clone(),
                allowed_tools: self.allowed_tools.clone(),
            }),
            #[cfg(feature = "mcp-stdio")]
            McpTransport::Stdio(_) => None,
        }
    }

    /// Initializes a connection to the server using the transport.
    async fn connect(&self) -> Result<McpService, McpError> {
        let client_info = ClientConfig::new(
            ClientCapabilities::builder().enable_tasks().build(),
            Implementation::from_build_env(),
        );
        let lifecycle = ClientLifecycleMode::Auto {
            preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            legacy_version: Some(ProtocolVersion::V_2025_11_25),
        };
        match &self.transport {
            McpTransport::Http(http) => http.connect(client_info, lifecycle).await,
            #[cfg(feature = "mcp-stdio")]
            McpTransport::Stdio(stdio) => stdio.connect(client_info, lifecycle).await,
        }
    }

    /// Connects to the MCP server and returns its tools as [`McpFunctionTool`]s keyed by name.
    pub async fn to_function_tools(&self) -> Result<HashMap<String, McpFunctionTool>, McpError> {
        let tools = McpClient::initilize(self).await?.functions;
        Ok(tools)
    }

    /// Converts this MCP server connection into a [`SimpleToolbox`].
    pub async fn to_toolbox(
        &self,
    ) -> Result<SimpleToolbox<HashMap<String, McpFunctionTool>>, McpError> {
        let tools = self.to_function_tools().await?;
        Ok(SimpleToolbox::new(self.name(), tools))
    }
}

impl ServiceKey {
    fn new(name: &str, transport: &McpTransport) -> Self {
        use secrecy::ExposeSecret;

        #[cfg(feature = "mcp-stdio")]
        let args_len: String;
        let mut parts = vec![name];
        match transport {
            McpTransport::Http(http) => {
                parts.extend(["http", http.url.expose_secret()]);
                parts.extend(http.authorization.as_ref().map(|a| a.expose_secret()));
            }
            #[cfg(feature = "mcp-stdio")]
            McpTransport::Stdio(stdio) => {
                args_len = stdio.args.len().to_string();
                parts.extend(["stdio", &stdio.command, &args_len]);
                parts.extend(stdio.args.iter().map(String::as_str));
                for (key, value) in &stdio.env {
                    parts.extend([key.as_str(), value.expose_secret()]);
                }
            }
        }

        // The part count and each part's length are hashed too, so no two servers share an
        // encoding.
        let mut hasher = blake3::Hasher::new();
        hasher.update(&(parts.len() as u64).to_le_bytes());
        for part in parts {
            hasher.update(&(part.len() as u64).to_le_bytes());
            hasher.update(part.as_bytes());
        }
        Self(*hasher.finalize().as_bytes())
    }
}

#[cfg(test)]
mod service_key_tests {
    use super::*;

    fn key(server: McpServer) -> ServiceKey {
        server.service_key()
    }

    #[test]
    fn ignores_what_does_not_change_the_connection() {
        let a = McpServerHttp::new("https://x", "a").with_allowed_tools(vec!["t".into()]);
        let b = McpServerHttp::new("https://x", "a")
            .with_description("d")
            .always_local(true);
        assert_eq!(key(a.into()), key(b.into()));
    }

    #[test]
    fn separates_one_server_declared_under_two_names() {
        let a = key(McpServerHttp::new("https://x", "a").into());
        let b = key(McpServerHttp::new("https://x", "b").into());
        assert_ne!(a, b);
    }

    #[test]
    fn tells_connections_apart() {
        let base = key(McpServerHttp::new("https://x", "a").into());
        let url = key(McpServerHttp::new("https://y", "a").into());
        let auth = key(McpServerHttp::new("https://x", "a")
            .with_authorization("t")
            .into());
        assert_ne!(base, url);
        assert_ne!(base, auth);
    }

    #[test]
    #[cfg(feature = "mcp-stdio")]
    fn does_not_confuse_field_boundaries() {
        let one = McpServerStdio::new("cmd", "s").with_args(vec!["ab".into(), "c".into()]);
        let two = McpServerStdio::new("cmd", "s").with_args(vec!["a".into(), "bc".into()]);
        assert_ne!(key(one.into()), key(two.into()));
        let env = |k: &str, v: &str| {
            key(McpServerStdio::new("cmd", "s")
                .with_env(vec![(k.into(), v.into())])
                .into())
        };
        assert_ne!(env("ab", "c"), env("a", "bc"));
    }
}
