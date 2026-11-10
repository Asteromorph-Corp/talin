use rmcp::model::ClientConfig;
use rmcp::transport::TokioChildProcess;
use rmcp::{ClientLifecycleMode, ClientServiceExt};
use secrecy::{ExposeSecret, SecretString};

use crate::mcp::transport::McpTransport;
use crate::mcp::{McpError, McpServer, McpService};

/// Stdio (child process) transport settings.
#[derive(Debug, Clone)]
pub struct StdioTransport {
    /// The program to execute.
    pub(crate) command: String,
    /// Arguments passed to the program.
    pub(crate) args: Vec<String>,
    /// Extra environment variables for the process.
    pub(crate) env: Vec<(String, SecretString)>,
}

impl StdioTransport {
    pub(in crate::mcp) async fn connect(
        &self,
        client_info: ClientConfig,
        lifecycle: ClientLifecycleMode,
    ) -> Result<McpService, McpError> {
        let mut cmd = tokio::process::Command::new(&self.command);
        cmd.args(&self.args);
        for (key, value) in &self.env {
            cmd.env(key, value.expose_secret());
        }
        let transport = TokioChildProcess::new(cmd).map_err(McpError::Spawn)?;
        Ok(client_info
            .serve_with_lifecycle(transport, lifecycle)
            .await?)
    }
}

impl McpServer<StdioTransport> {
    /// Creates a new stdio MCP server that runs `command`.
    #[must_use]
    pub fn new(command: impl Into<String>, name: impl Into<String>) -> Self {
        let transport = StdioTransport {
            command: command.into(),
            args: Vec::new(),
            env: Vec::new(),
        };
        Self {
            transport,
            name: name.into(),
            description: None,
            allowed_tools: None,
        }
    }

    /// Sets the command-line arguments of the process.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.transport.args = args;
        self
    }

    /// Sets extra environment variables of the process.
    pub fn with_env(mut self, env: Vec<(String, SecretString)>) -> Self {
        self.transport.env = env;
        self
    }
}

impl From<McpServer<StdioTransport>> for McpServer {
    fn from(server: McpServer<StdioTransport>) -> Self {
        server.map_transport(McpTransport::Stdio)
    }
}
