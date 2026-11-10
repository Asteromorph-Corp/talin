use std::sync::Arc;

use async_trait::async_trait;
use rmcp::model::{CallToolRequestParams, CallToolResponse};
use schemars::Schema;
use serde::de::Error;
use serde_json::Value;

use crate::mcp::conversion::{parse_mcp_contents, parse_mcp_error};
use crate::mcp::task::poll_task;
use crate::mcp::{McpError, McpService};
use crate::schemas::ToolOutput;
use crate::tools::{FunctionTool, ToolError, sealed};

/// A function tool representation of an MCP tool.
///
/// For LLMs that do not have native support for MCP tools, this adapter lets MCP tools to be used
/// as standard function tools.
pub struct McpFunctionTool {
    client: Arc<McpService>,
    name: String,
    description: Option<String>,
    parameters: Schema,
}

impl McpFunctionTool {
    /// Creates a new [`McpFunctionTool`].
    pub fn new(
        client: Arc<McpService>,
        name: String,
        description: Option<String>,
        parameters: Schema,
    ) -> Self {
        Self {
            client,
            name,
            description,
            parameters,
        }
    }

    /// Creates an [`McpFunctionTool`] from an rmcp [`Tool`](rmcp::model::Tool) definition.
    ///
    /// This is the primary way to create `McpFunctionTool` instances when
    /// converting tools from an MCP server.
    pub fn from_rmcp_tool(
        service: &Arc<McpService>,
        tool: rmcp::model::Tool,
    ) -> Result<Self, serde_json::Error> {
        let parameters = Schema::try_from(tool.schema_as_json_value())?;
        let name = tool.name.into_owned();
        let description = tool.description.map(|d| d.into_owned());
        let tool = McpFunctionTool::new(Arc::clone(service), name, description, parameters);
        Ok(tool)
    }
}

impl sealed::Sealed for McpFunctionTool {}

#[async_trait]
impl FunctionTool for McpFunctionTool {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn description(&self) -> String {
        self.description
            .as_ref()
            .map_or_else(|| "No description provided".to_string(), |d| d.to_string())
    }

    fn parameters(&self) -> Schema {
        self.parameters.clone()
    }

    fn strict(&self) -> bool {
        false
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let input = match input {
            Value::Object(obj) => obj,
            _ => {
                return Err(ToolError::InputDeserialize(serde_json::Error::custom(
                    "invalid type, expected object",
                )));
            }
        };

        let response = self
            .client
            .call_tool_once(CallToolRequestParams::new(self.name.clone()).with_arguments(input))
            .await
            .map_err(McpError::from)?;
        let tool_result = match response {
            CallToolResponse::Complete(result) => result,
            CallToolResponse::Task(task) => poll_task(&self.client, task.task).await?,
            CallToolResponse::InputRequired(_) => return Err(McpError::InputRequired.into()),
            _ => return Err(McpError::invalid_content("unsupported `tools/call` response").into()),
        };

        if tool_result.is_error.unwrap_or(false) {
            let error_message = parse_mcp_error(tool_result.content)?;
            Err(ToolError::Execution(error_message.into()))
        } else {
            let content = parse_mcp_contents(tool_result.content)?;
            Ok(ToolOutput::from(content))
        }
    }
}
