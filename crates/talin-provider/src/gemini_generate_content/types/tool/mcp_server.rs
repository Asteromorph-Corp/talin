use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

use crate::gemini_generate_content::types::duration::GoogleDuration;

/// Configuration for a single Model Context Protocol (MCP) server that exposes
/// tools to the model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct McpServer {
    /// Required. The name of the MCP server.
    pub name: String,

    /// Optional. Configuration for the streamable HTTP transport.
    pub streamable_http_transport: Option<StreamableHttpTransport>,
}

impl McpServer {
    /// Creates a new McpServer with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            streamable_http_transport: None,
        }
    }

    /// Sets the streamable HTTP transport configuration.
    pub fn with_streamable_http_transport(
        mut self,
        streamable_http_transport: StreamableHttpTransport,
    ) -> Self {
        self.streamable_http_transport = Some(streamable_http_transport);
        self
    }
}

/// HTTP transport mechanism used to reach an MCP server.
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StreamableHttpTransport {
    /// Required. Full URL for the MCP server endpoint.
    pub url: String,

    /// Optional. HTTP headers (e.g. authentication) sent with requests.
    pub headers: Option<HashMap<String, String>>,

    /// Optional. HTTP timeout for regular operations.
    #[serde(default)]
    #[serde_as(as = "Option<GoogleDuration>")]
    pub timeout: Option<Duration>,

    /// Optional. Timeout for Server-Sent Events read operations.
    #[serde(default)]
    #[serde_as(as = "Option<GoogleDuration>")]
    pub sse_read_timeout: Option<Duration>,

    /// Optional. Whether to close the client session on transport close.
    pub terminate_on_close: Option<bool>,
}

impl StreamableHttpTransport {
    /// Creates a new StreamableHttpTransport with the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: None,
            timeout: None,
            sse_read_timeout: None,
            terminate_on_close: None,
        }
    }

    /// Sets the HTTP headers sent with requests.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Sets the HTTP timeout for regular operations.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the timeout for Server-Sent Events read operations.
    pub fn with_sse_read_timeout(mut self, sse_read_timeout: Duration) -> Self {
        self.sse_read_timeout = Some(sse_read_timeout);
        self
    }

    /// Sets whether to close the client session on transport close.
    pub fn with_terminate_on_close(mut self, terminate_on_close: bool) -> Self {
        self.terminate_on_close = Some(terminate_on_close);
        self
    }
}
