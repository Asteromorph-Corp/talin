use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::error::AgentToolError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentMcpTool {
    pub server: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct AgentMcpToolGrouped {
    pub server: String,
    pub names: Option<Vec<String>>,
}

impl AgentMcpTool {
    /// Groups MCP tools by server name and validates tool specifications.
    pub(crate) fn group_tools(
        tools: Vec<AgentMcpTool>,
    ) -> Result<Vec<AgentMcpToolGrouped>, AgentToolError> {
        tools
            .into_iter()
            // Group tools by server name, preserving insertion order
            .fold(IndexMap::new(), |mut acc, tool| {
                acc.entry(tool.server)
                    .or_insert_with(Vec::new)
                    .push(tool.name);
                acc
            })
            .into_iter()
            .map(|(server, names)| collapse_checked(server, names))
            .collect()
    }

    /// Regroup base mcp tools and override tools.
    pub(crate) fn regroup_tools(
        tools: Vec<AgentMcpToolGrouped>,
        tools_override: Vec<AgentMcpToolGrouped>,
    ) -> Vec<AgentMcpToolGrouped> {
        let mut tool_map: IndexMap<String, Option<Vec<String>>> = IndexMap::new();

        for tool in tools.into_iter().chain(tools_override) {
            let entry = tool_map
                .entry(tool.server)
                .or_insert_with(|| Some(Vec::new()));

            match (entry, tool.names) {
                // If the existing entry is a wildcard, it remains a wildcard
                (None, _) => continue,
                // If the new entry is a wildcard, it overrides the existing specific tools
                (entry, None) => *entry = None,
                // Both entries have specific tools, merge them
                (Some(existing), Some(new)) => existing.extend(new),
            }
        }

        // Convert the map back into a vector of AgentMcpToolGrouped
        tool_map
            .into_iter()
            .map(|(server, names)| AgentMcpToolGrouped { server, names })
            .collect()
    }
}

impl FromStr for AgentMcpTool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((server, name)) = s.split_once('/') else {
            return Err(format!(
                "Invalid format for mcp tool: {s}, expected '<server_id>/<tool_name>'"
            ));
        };

        let name = match name {
            "*" => None,
            _ => Some(name.to_owned()),
        };

        Ok(AgentMcpTool {
            server: server.to_owned(),
            name,
        })
    }
}

/// Validates and collapses a vector of optional tool names for a single MCP server.
///
/// This function enforces the following rules:
/// - `None` represents a wildcard (`*`) that includes all tools from the server
/// - At most one wildcard is allowed
/// - Wildcards cannot be mixed with specific tool names
fn collapse_checked(
    server: String,
    names: Vec<Option<String>>,
) -> Result<AgentMcpToolGrouped, AgentToolError> {
    let (none_count, some_values) = names.into_iter().fold(
        (0usize, Vec::new()),
        |(mut none_count, mut some_values), val| {
            match val {
                Some(value) => some_values.push(value),
                None => none_count += 1,
            }
            (none_count, some_values)
        },
    );

    let names = match none_count {
        // No wildcards, return the specific tool names
        0 => Some(some_values),
        // Only wildcard, no specific tools - this is valid
        1 if some_values.is_empty() => None,
        // Wildcard mixed with specific tools - this is an error
        1 => {
            let first = some_values
                .into_iter()
                .next()
                .expect("some_values is not empty");
            return Err(AgentToolError::McpWildcardConflict(server, first));
        }
        // Multiple wildcards - this is an error
        _ => return Err(AgentToolError::McpMultipleWildcard(server)),
    };
    Ok(AgentMcpToolGrouped { server, names })
}
