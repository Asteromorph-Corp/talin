//! The transports used to communicate with an [`McpServer`].
//!
//! Supported transports:
//! - Streamable HTTP: [`HttpTransport`]
//! - Stdio: `StdioTransport` (requires the `mcp-stdio` feature)
//!
//! [`McpServer`]: crate::mcp::McpServer

mod http;
#[cfg(feature = "mcp-stdio")]
mod stdio;

pub use http::HttpTransport;
#[cfg(feature = "mcp-stdio")]
pub use stdio::StdioTransport;

/// The transport for an [`McpServer`] which specifies how to connect to the server.
///
/// [`McpServer`]: crate::mcp::McpServer
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum McpTransport {
    /// Connect to a remote MCP server over HTTP.
    Http(HttpTransport),
    /// Launches the local MCP server as a child process, and communcate with it over stdin/stdout.
    ///
    /// Requires the `mcp-stdio` feature.
    #[cfg(feature = "mcp-stdio")]
    Stdio(StdioTransport),
}
