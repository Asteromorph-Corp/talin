use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::mcp::{McpError, McpFunctionTool, McpServer, McpService, ServiceKey};
use crate::tools::normalize_tool_name;

/// An MCP client used by models that do not support native MCP function calling.
pub struct McpClient {
    /// The connection `service` was opened with.
    pub(crate) key: ServiceKey,
    pub(crate) service: Arc<McpService>,
    /// Whether every tool the server exposes is allowed.
    pub(crate) allow_all: bool,
    pub(crate) functions: HashMap<String, McpFunctionTool>,
}

impl McpClient {
    /// Connects to the MCP server and retrieves its allowed tools as function tools.
    pub(crate) async fn initilize(config: &McpServer) -> Result<Self, McpError> {
        let service = Self::connect(config).await?;
        Self::from_service(service, config).await
    }

    /// Opens a connection to the MCP server.
    pub(crate) async fn connect(config: &McpServer) -> Result<Arc<McpService>, McpError> {
        Ok(Arc::new(config.connect().await?))
    }

    /// Retrieves the allowed tools of the MCP server over an existing connection.
    pub(crate) async fn from_service(
        service: Arc<McpService>,
        config: &McpServer,
    ) -> Result<Self, McpError> {
        Self::fetch(
            config.service_key(),
            service,
            config.allowed_tools.as_deref(),
        )
        .await
    }

    /// Re-fetches the server's tools over the existing session, applying the (possibly changed)
    /// allow-list. Reuses the cached [`service`](Self::service) rather than reconnecting.
    pub(crate) async fn sync(&mut self, config: &McpServer) -> Result<(), McpError> {
        let service = self.service.clone();
        *self = Self::fetch(self.key, service, config.allowed_tools.as_deref()).await?;
        Ok(())
    }

    /// Whether this client already reflects `config`: the same connection and allow-list.
    pub(crate) fn reflects(&self, config: &McpServer) -> bool {
        self.key == config.service_key() && self.reflects_allow_list(config)
    }

    /// Whether this client already reflects `config`'s allow-list.
    pub(crate) fn reflects_allow_list(&self, config: &McpServer) -> bool {
        match config.allowed_tools.as_ref() {
            None => self.allow_all,
            Some(_) if self.allow_all => false,
            Some(allowed) => {
                let desired = allowed
                    .iter()
                    .map(|name| normalize_tool_name(name))
                    .collect::<HashSet<_>>();
                desired.len() == self.functions.len()
                    && desired.iter().all(|name| self.functions.contains_key(name))
            }
        }
    }

    /// Constructs a [`McpClient`] from an initialized server and allow list.
    async fn fetch(
        key: ServiceKey,
        service: Arc<McpService>,
        allowed_tools: Option<&[String]>,
    ) -> Result<Self, McpError> {
        let allowed = allowed_tools.map(|names| {
            names
                .iter()
                .map(|name| normalize_tool_name(name))
                .collect::<HashSet<_>>()
        });

        let all_tools = service.list_all_tools().await?;
        let mut functions: HashMap<String, McpFunctionTool> =
            HashMap::with_capacity(all_tools.len());
        for tool in all_tools {
            let name = normalize_tool_name(tool.name.as_ref());
            let tool = McpFunctionTool::from_rmcp_tool(&service, tool)?;

            if let Some(names) = &allowed
                && !names.contains(&name)
            {
                continue;
            }
            functions.insert(name, tool);
        }

        if let Some(ref names) = allowed {
            let found = functions.keys().map(String::as_str).collect::<HashSet<_>>();
            let missing = names
                .iter()
                .filter(|name| !found.contains(name.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                return Err(McpError::ToolNotFound(missing.join(", ")));
            }
        }

        Ok(Self {
            key,
            service,
            allow_all: allowed_tools.is_none(),
            functions,
        })
    }
}
