use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::CacheControlEphemeral;

/// A set of tools configured for use with a specific MCP server.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct McpToolset {
    /// The name of the MCP server providing the tools. Must match the name of one of the
    /// servers in [`mcp_servers`](crate::claude_messages::types::MessagesRequest::mcp_servers).
    pub mcp_server_name: String,

    /// Default configuration applied to all tools in this set. Individual tool configs in
    /// [`configs`](Self::configs) will override these defaults.
    pub default_config: Option<McpToolConfig>,

    /// Per-tool configuration overrides. Keys are tool names, values are configuration objects.
    pub configs: Option<HashMap<String, McpToolConfig>>,

    /// Cache breakpoint configuration for this toolset.
    pub cache_control: Option<CacheControlEphemeral>,
}

/// Configuration options for an MCP tool.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct McpToolConfig {
    /// Whether this tool is enabled.
    pub enabled: Option<bool>,

    /// Whether the tool definitions should be deferred until first use.
    pub defer_loading: Option<bool>,
}

impl McpToolset {
    /// Creates a new [`McpToolset`] for the given MCP server name.
    pub fn new(mcp_server_name: impl Into<String>) -> Self {
        Self {
            mcp_server_name: mcp_server_name.into(),
            default_config: None,
            configs: None,
            cache_control: None,
        }
    }

    /// Sets the [`default_config`](Self::default_config) applied to all tools in this set.
    pub fn with_default_config(mut self, default_config: McpToolConfig) -> Self {
        self.default_config = Some(default_config);
        self
    }

    /// Sets the per-tool [`configs`](Self::configs) overrides.
    pub fn with_configs(mut self, configs: HashMap<String, McpToolConfig>) -> Self {
        self.configs = Some(configs);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this toolset.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl McpToolConfig {
    /// Creates a new [`McpToolConfig`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`McpToolConfig`] with the given enabled state.
    pub const fn enabled(enabled: bool) -> Self {
        Self {
            enabled: Some(enabled),
            defer_loading: None,
        }
    }

    /// Sets whether this tool is [`enabled`](Self::enabled).
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets whether the tool definitions should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }
}
