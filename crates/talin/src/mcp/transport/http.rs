use rmcp::model::ClientConfig;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::{ClientLifecycleMode, ClientServiceExt};
use secrecy::{ExposeSecret, SecretString};

use crate::mcp::transport::McpTransport;
use crate::mcp::{McpError, McpServer, McpService};

/// Streamable HTTP transport settings.
#[derive(Debug, Clone)]
pub struct HttpTransport {
    /// The URL for the MCP server.
    pub(crate) url: SecretString,
    /// Optional authorization token for the MCP server.
    pub(crate) authorization: Option<SecretString>,
    /// If true, a local client will always be used to connect to the MCP server, even for
    /// providers with native MCP support.
    pub(crate) always_local: bool,
}

impl HttpTransport {
    pub(in crate::mcp) async fn connect(
        &self,
        client_info: ClientConfig,
        lifecycle: ClientLifecycleMode,
    ) -> Result<McpService, McpError> {
        let mut config = StreamableHttpClientTransportConfig::with_uri(self.url.expose_secret());
        if let Some(authorization) = &self.authorization {
            config = config.auth_header(authorization.expose_secret());
        }
        let transport = StreamableHttpClientTransport::from_config(config);
        Ok(client_info
            .serve_with_lifecycle(transport, lifecycle)
            .await?)
    }
}

impl McpServer<HttpTransport> {
    /// Creates a new streamable HTTP MCP server.
    pub fn new(url: impl Into<SecretString>, name: impl Into<String>) -> Self {
        Self {
            transport: HttpTransport {
                url: url.into(),
                authorization: None,
                always_local: false,
            },
            name: name.into(),
            description: None,
            allowed_tools: None,
        }
    }

    /// Sets the authorization token for the MCP server.
    pub fn with_authorization(mut self, authorization: impl Into<SecretString>) -> Self {
        self.transport.authorization = Some(authorization.into());
        self
    }

    /// Forces a local client for this server, even for providers with native MCP support.
    ///
    /// Set for servers that are only reachable from this process (e.g. IP allowlist).
    pub fn always_local(mut self, always_local: bool) -> Self {
        self.transport.always_local = always_local;
        self
    }
}

impl From<McpServer<HttpTransport>> for McpServer {
    fn from(server: McpServer<HttpTransport>) -> Self {
        server.map_transport(McpTransport::Http)
    }
}
