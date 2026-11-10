use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{CacheControlEphemeral, ToolCaller};
use crate::utils::string_enum;

/// A tool use block within a response message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolUseBlock {
    /// The id of the tool use.
    pub id: String,

    /// The name of the tool.
    pub name: String,

    /// The input to the tool.
    #[cfg_attr(
        test,
        schema(schema_with = crate::claude_messages::spec::schema::object_schema)
    )]
    pub input: serde_json::Value,

    /// What invoked the tool.
    pub caller: ToolCaller,

    /// For a toolset member tool use, the toolset family it belongs to.
    pub toolset_name: Option<String>,
}

/// A tool use block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolUseBlockParam {
    /// The id of the tool use.
    pub id: String,

    /// The name of the tool.
    pub name: String,

    /// The input to the tool.
    #[cfg_attr(
        test,
        schema(schema_with = crate::claude_messages::spec::schema::object_schema)
    )]
    pub input: serde_json::Value,

    /// What invoked the tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub caller: Option<ToolCaller>,

    /// For a toolset member tool use, the toolset family it belongs to.
    pub toolset_name: Option<String>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

/// A server tool use block within a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ServerToolUseBlock {
    /// The id of the tool use.
    pub id: String,

    /// The name of the server tool.
    pub name: ServerToolName,

    /// The input to the tool.
    #[cfg_attr(
        test,
        schema(schema_with = crate::claude_messages::spec::schema::object_schema)
    )]
    pub input: serde_json::Value,

    /// What invoked the tool.
    pub caller: ToolCaller,
}

/// A server tool use block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ServerToolUseBlockParam {
    /// The id of the tool use.
    pub id: String,

    /// The name of the server tool.
    pub name: ServerToolName,

    /// The input to the tool.
    #[cfg_attr(
        test,
        schema(schema_with = crate::claude_messages::spec::schema::object_schema)
    )]
    pub input: serde_json::Value,

    /// What invoked the tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub caller: Option<ToolCaller>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

string_enum! {
    /// The name of one of Claude's internal tools.
    #[serde(rename_all = "snake_case")]
    pub enum ServerToolName {
        /// `"web_search"`
        WebSearch,

        /// `"web_fetch"`
        WebFetch,

        /// `"code_execution"`
        CodeExecution,

        /// `"bash_code_execution"`
        BashCodeExecution,

        /// `"text_editor_code_execution"`
        TextEditorCodeExecution,

        /// `"tool_search_tool_regex"`
        ToolSearchToolRegex,

        /// `"tool_search_tool_bm25"`
        ToolSearchToolBm25,
    }
}

/// An MCP tool use block.
///
/// Returned when Claude invokes a tool provided by one of the MCP servers.
#[cfg(feature = "mcp-client-2025-11-20")]
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct McpToolUseBlock {
    /// The id of the tool use.
    pub id: String,

    /// The name of the MCP server that provided the tool.
    pub server_name: String,

    /// The name of the tool.
    pub name: String,

    /// The input to the tool.
    pub input: serde_json::Value,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

impl ToolUseBlock {
    /// Creates a new [`ToolUseBlock`] from the given id, name, and input.
    pub fn new(id: impl Into<String>, name: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input,
            caller: ToolCaller::direct(),
            toolset_name: None,
        }
    }

    /// Sets what [`caller`](Self::caller) invoked the tool.
    pub fn with_caller(mut self, caller: ToolCaller) -> Self {
        self.caller = caller;
        self
    }

    /// Sets the [`toolset_name`](Self::toolset_name) this tool use belongs to.
    pub fn with_toolset_name(mut self, toolset_name: impl Into<String>) -> Self {
        self.toolset_name = Some(toolset_name.into());
        self
    }
}

impl ToolUseBlockParam {
    /// Creates a new [`ToolUseBlockParam`] from the given id, name, and input.
    pub fn new(id: impl Into<String>, name: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input,
            caller: None,
            toolset_name: None,
            cache_control: None,
        }
    }

    /// Sets what [`caller`](Self::caller) invoked the tool.
    pub fn with_caller(mut self, caller: ToolCaller) -> Self {
        self.caller = Some(caller);
        self
    }

    /// Sets the [`toolset_name`](Self::toolset_name) this tool use belongs to.
    pub fn with_toolset_name(mut self, toolset_name: impl Into<String>) -> Self {
        self.toolset_name = Some(toolset_name.into());
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl ServerToolUseBlock {
    /// Creates a new [`ServerToolUseBlock`] from the given id, name, and input.
    pub fn new(id: impl Into<String>, name: ServerToolName, input: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name,
            input,
            caller: ToolCaller::direct(),
        }
    }

    /// Sets what [`caller`](Self::caller) invoked the tool.
    pub fn with_caller(mut self, caller: ToolCaller) -> Self {
        self.caller = caller;
        self
    }
}

impl ServerToolUseBlockParam {
    /// Creates a new [`ServerToolUseBlockParam`] from the given id, name, and input.
    pub fn new(id: impl Into<String>, name: ServerToolName, input: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name,
            input,
            caller: None,
            cache_control: None,
        }
    }

    /// Sets what [`caller`](Self::caller) invoked the tool.
    pub fn with_caller(mut self, caller: ToolCaller) -> Self {
        self.caller = Some(caller);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

#[cfg(feature = "mcp-client-2025-11-20")]
impl McpToolUseBlock {
    /// Creates a new [`McpToolUseBlock`] from the given id, server name, name, and input.
    pub fn new(
        id: impl Into<String>,
        server_name: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            server_name: server_name.into(),
            name: name.into(),
            input,
            cache_control: None,
        }
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl From<ToolUseBlock> for ToolUseBlockParam {
    fn from(value: ToolUseBlock) -> Self {
        Self {
            id: value.id,
            name: value.name,
            input: value.input,
            caller: Some(value.caller),
            toolset_name: value.toolset_name,
            cache_control: None,
        }
    }
}

impl From<ServerToolUseBlock> for ServerToolUseBlockParam {
    fn from(value: ServerToolUseBlock) -> Self {
        Self {
            id: value.id,
            name: value.name,
            input: value.input,
            caller: Some(value.caller),
            cache_control: None,
        }
    }
}
