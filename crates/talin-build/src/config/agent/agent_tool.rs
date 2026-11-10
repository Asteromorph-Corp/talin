use std::str::FromStr;

use serde::de::Error;
use serde::{Deserialize, Serialize};

use crate::config::{AgentBuiltinTool, AgentLocalTool, AgentMcpTool};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", remote = "Self")]
pub(crate) enum AgentTool {
    Local(AgentLocalTool),
    Mcp(AgentMcpTool),
    Builtin(AgentBuiltinTool),
}

impl AgentTool {
    /// Partitions a vector of [`AgentTool`] into separate vectors for each tool type.
    pub(crate) fn partition_tools(
        tools: Vec<AgentTool>,
    ) -> (
        Vec<AgentLocalTool>,
        Vec<AgentMcpTool>,
        Vec<AgentBuiltinTool>,
    ) {
        tools
            .into_iter()
            .fold((Vec::new(), Vec::new(), Vec::new()), |mut acc, tool| {
                match tool {
                    AgentTool::Local(local_tool) => acc.0.push(local_tool),
                    AgentTool::Mcp(mcp_tool) => acc.1.push(mcp_tool),
                    AgentTool::Builtin(builtin_tool) => acc.2.push(builtin_tool),
                }
                acc
            })
    }
}

impl FromStr for AgentTool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((kind, tool)) = s.split_once('.') else {
            return Ok(AgentTool::Local(AgentLocalTool::from(s.to_owned())));
        };

        match kind {
            "local" => Ok(AgentTool::Local(AgentLocalTool::from(tool.to_owned()))),
            "mcp" => Ok(AgentTool::Mcp(tool.parse()?)),
            "builtin" => Ok(AgentTool::Builtin(tool.parse()?)),
            _ => Err(format!("unknown tool kind prefix: {}", kind)),
        }
    }
}

impl Serialize for AgentTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Self::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for AgentTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = toml::Value::deserialize(deserializer)?;
        if let Some(table) = value.as_table_mut()
            && !table.contains_key("kind")
        {
            table.insert("kind".to_owned(), toml::Value::from("local"));
        }

        match value {
            toml::Value::String(s) => s.parse().map_err(D::Error::custom),
            toml::Value::Table(table) => {
                let tool = AgentTool::deserialize(table).map_err(D::Error::custom)?;
                Ok(tool)
            }
            _ => Err(serde::de::Error::custom("expected a string or a table")),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::string(serde_json::json!("local.my_tool"), "my_tool")]
    #[case::map(
        serde_json::json!({
            "kind": "local",
            "name": "my_tool"
        }),
        "my_tool"
    )]
    fn deserializes_local(#[case] value: serde_json::Value, #[case] expected_name: &str) {
        let tool: AgentTool = serde_json::from_value(value).unwrap();
        let AgentTool::Local(tool) = tool else {
            panic!("expected a local tool");
        };
        assert_eq!(tool.name, expected_name);
    }

    #[rstest]
    #[case::string_specific(serde_json::json!("mcp.my_server/some_tool"), "my_server", Some("some_tool"))]
    #[case::string_all(serde_json::json!("mcp.my_server/*"), "my_server", None)]
    #[case::map_specific(
        serde_json::json!({
            "kind": "mcp",
            "server": "my_server",
            "name": "some_tool"
        }),
        "my_server",
        Some("some_tool")
    )]
    #[case::map_all(
        serde_json::json!({
            "kind": "mcp",
            "server": "my_server"
        }),
        "my_server",
        None
    )]
    fn deserializes_mcp(
        #[case] value: serde_json::Value,
        #[case] expected_server: &str,
        #[case] expected_name: Option<&str>,
    ) {
        let tool: AgentTool = serde_json::from_value(value).unwrap();
        let AgentTool::Mcp(tool) = tool else {
            panic!("expected a mcp tool");
        };
        assert_eq!(tool.server, expected_server);
        assert_eq!(tool.name.as_deref(), expected_name);
    }

    #[rstest]
    #[case::string(serde_json::json!("builtin.openai_responses/web_search"))]
    #[case::map(
        serde_json::json!({
            "kind": "builtin",
            "provider": "openai_responses",
            "type": "web_search",
        })
    )]
    fn deserialize_string_builtin(#[case] value: serde_json::Value) {
        let tool: AgentTool = serde_json::from_value(value).unwrap();
        assert!(matches!(
            tool,
            AgentTool::Builtin(AgentBuiltinTool::OpenAiResponses(_))
        ));
    }
}
