use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::ToolType;

/// The output from a server-side ToolCall execution.
///
/// This contains the results of a tool invocation that was initiated by a
/// ToolCall from the model. The client should pass this ToolResponse back
/// to the API in a subsequent turn within a Content message, along with
/// the corresponding ToolCall.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolResponse {
    /// Optional. The identifier of the tool call this response is for.
    pub id: Option<String>,

    /// Required. The type of tool that was called, matching the `tool_type`
    /// in the corresponding ToolCall.
    pub tool_type: ToolType,

    /// Optional. The tool response.
    pub response: Option<serde_json::Value>,
}

impl ToolResponse {
    /// Creates a new [`ToolResponse`].
    pub fn new(tool_type: ToolType) -> Self {
        Self {
            id: None,
            tool_type,
            response: None,
        }
    }

    /// Sets the `id` field.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the `response` field.
    pub fn with_response(mut self, response: serde_json::Value) -> Self {
        self.response = Some(response);
        self
    }
}
