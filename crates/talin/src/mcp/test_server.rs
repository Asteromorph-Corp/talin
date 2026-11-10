//! An in-process MCP server for tests of the local MCP clients.

use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::mcp::{McpServer, McpServerHttp};

/// Arguments for the `echo` tool. `#[tool]` extracts JSON arguments through the [`Parameters`]
/// wrapper, so the payload type must derive [`Deserialize`] + [`JsonSchema`] — a bare `String`
/// parameter is not a recognized extractor and fails to implement `IntoToolRoute`.
#[derive(Debug, Deserialize, JsonSchema)]
struct EchoArgs {
    /// The text to echo back.
    text: String,
}

/// A minimal MCP server exposing two tools: `echo` (returns its `text` argument) and `greet`
/// (returns a fixed greeting). Two tools let the tests exercise `allowed_tools` filtering.
#[derive(Clone)]
struct MockMcpServer {
    // Read by the `#[tool_handler]`-generated dispatch, which dead-code analysis can't see.
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MockMcpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Echoes the provided text.")]
    async fn echo(
        &self,
        Parameters(EchoArgs { text }): Parameters<EchoArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "echo: {text}"
        ))]))
    }

    #[tool(description = "Returns a fixed greeting.")]
    async fn greet(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text("hello")]))
    }
}

#[tool_handler]
impl ServerHandler for MockMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
}

/// Serves [`MockMcpServer`] over streamable HTTP on an ephemeral port, returning the MCP endpoint
/// URL. The server task runs for the duration of the test process.
pub(crate) async fn spawn_mock_server() -> String {
    let service = StreamableHttpService::new(
        || Ok(MockMcpServer::new()),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );
    let app = axum::Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{addr}/mcp")
}

/// The mock server named `mock`, always handled by a local client even with native MCP support.
///
/// Without `allowed_tools`, every tool of the server is allowed.
pub(crate) fn local_server(url: &str, allowed_tools: Option<&[&str]>) -> McpServer {
    let server = McpServerHttp::new(url, "mock").always_local(true);
    let server = match allowed_tools {
        Some(names) => server.with_allowed_tools(names.iter().map(|&n| n.to_owned()).collect()),
        None => server,
    };
    McpServer::from(server)
}
