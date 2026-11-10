use std::collections::BTreeMap;

use crate::config::{Agent, Configs, FromConfigError, McpServer};
use crate::error::{AgentError, AgentToolError, TalinError};

pub(crate) fn validate_configs(configs: &Configs) -> Result<(), TalinError> {
    for (id, agent) in &configs.agents {
        confirm_mcp_tool_existence(id, agent, &configs.servers)?;
    }

    Ok(())
}

fn confirm_mcp_tool_existence(
    id: &str,
    agent: &Agent,
    servers: &BTreeMap<String, McpServer>,
) -> Result<(), TalinError> {
    for mcp_tool in &agent.mcp_servers {
        let Some(server) = servers.get(&mcp_tool.server) else {
            return Err(AgentError::missing_mcp_server(&mcp_tool.server).into_error(id));
        };

        let Some(names) = &mcp_tool.names else {
            continue;
        };

        for name in names {
            if !server.tools.contains(name) {
                return Err(AgentError::InvalidTool(AgentToolError::McpUnspecifiedTool(
                    mcp_tool.server.clone(),
                    name.clone(),
                ))
                .into_error(id));
            }
        }
    }

    Ok(())
}
