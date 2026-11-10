//! Currently, the tool change structures here only contain responses (not requests) for tool
//! changes in the middle of LLM execution; these are returned as a result of compaction of a range
//! of messages containing such tool changes. Thus, these are not strictly needed as tool change
//! requests are currently not implemented for this library, but we create the types in advance.

use serde::{Deserialize, Serialize};

use crate::claude_messages::types::ToolUnion;

/// A `tool_changes` entry in a compaction block.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResponseToolChange {
    /// Records a tool added within the compacted conversation.
    ToolAddition(ResponseToolAdditionBlock),
    /// Records a tool removed within the compacted conversation.
    ToolRemoval(ResponseToolRemovalBlock),
}

/// [`BetaResponseToolAdditionBlock`](https://platform.claude.com/docs/en/api/http/beta/messages#beta_response_tool_addition_block).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ResponseToolAdditionBlock {
    /// The tool added to or removed from the compacted conversation.
    pub tool: ResponseToolAdditionBlockTool,
}

/// [`BetaResponseToolRemovalBlock`](https://platform.claude.com/docs/en/api/http/beta/messages#beta_response_tool_removal_block).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ResponseToolRemovalBlock {
    /// The tool added to or removed from the compacted conversation.
    pub tool: ResponseToolRemovalBlockTool,
}

/// The `tool` union of [`ResponseToolAdditionBlock`].
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResponseToolAdditionBlockTool {
    /// References a tool by name.
    ToolReference(ResponseToolChangeToolReference),
    /// References an MCP tool by its tool and server names.
    McpToolReference(ResponseToolChangeMCPToolReference),
    /// References an MCP toolset by server name.
    McpToolsetReference(ResponseToolChangeMCPToolsetReference),
    /// Carries the full definition of an added tool.
    ToolDefinition(Box<ToolChangeToolDefinition>),
}

/// The `tool` union of [`ResponseToolRemovalBlock`].
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResponseToolRemovalBlockTool {
    /// References a tool by name.
    ToolReference(ResponseToolChangeToolReference),
    /// References an MCP tool by its tool and server names.
    McpToolReference(ResponseToolChangeMCPToolReference),
    /// References an MCP toolset by server name.
    McpToolsetReference(ResponseToolChangeMCPToolsetReference),
}

/// [`BetaResponseToolChangeToolReference`](https://platform.claude.com/docs/en/api/http/beta/messages#beta_response_tool_change_tool_reference).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ResponseToolChangeToolReference {
    /// The name of the referenced tool.
    pub name: String,
}

/// [`BetaResponseToolChangeMCPToolReference`](https://platform.claude.com/docs/en/api/http/beta/messages#beta_response_tool_change_mcp_tool_reference).
/// MCP connector tools also require `mcp-client-2026-09-15`.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ResponseToolChangeMCPToolReference {
    /// The name of the referenced tool.
    pub name: String,
    /// The name of the MCP server providing the referenced tool or toolset.
    pub server_name: String,
}

/// [`BetaResponseToolChangeMCPToolsetReference`](https://platform.claude.com/docs/en/api/http/beta/messages#beta_response_tool_change_mcp_toolset_reference).
/// MCP connector toolsets also require `mcp-client-2026-09-15`.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ResponseToolChangeMCPToolsetReference {
    /// The name of the MCP server providing the referenced tool or toolset.
    pub server_name: String,
}

/// [`BetaToolChangeToolDefinition`](https://platform.claude.com/docs/en/api/http/beta/messages#beta_tool_change_tool_definition).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolChangeToolDefinition {
    /// The full definition of the added tool.
    pub definition: ToolUnion,
}
