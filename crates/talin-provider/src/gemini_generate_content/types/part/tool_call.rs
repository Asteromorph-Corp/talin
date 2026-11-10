use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A predicted server-side tool call returned from the model.
///
/// This contains information about a tool that the model wants to invoke.
/// The client is NOT expected to execute this ToolCall. Instead, the client
/// should pass this ToolCall back to the API in a subsequent turn within a
/// Content message, along with the corresponding ToolResponse.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCall {
    /// Optional. Unique identifier of the tool call. The server returns the
    /// tool response with the matching id.
    pub id: Option<String>,

    /// Required. The type of tool that was called.
    pub tool_type: ToolType,

    /// Optional. The tool call arguments.
    pub args: Option<serde_json::Value>,
}

impl ToolCall {
    /// Creates a new [`ToolCall`].
    pub fn new(tool_type: ToolType) -> Self {
        Self {
            id: None,
            tool_type,
            args: None,
        }
    }

    /// Sets the `id` field.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the `args` field.
    pub fn with_args(mut self, args: serde_json::Value) -> Self {
        self.args = Some(args);
        self
    }
}

/// The type of tool in the tool call.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ToolType {
    /// Unspecified tool type.
    ToolTypeUnspecified,
    /// Google search tool, maps to `Tool.google_search`.
    GoogleSearchWeb,
    /// Image search tool, maps to `Tool.google_search`.
    GoogleSearchImage,
    /// URL context tool, maps to `Tool.url_context`.
    UrlContext,
    /// Google maps tool, maps to `Tool.google_maps`.
    GoogleMaps,
    /// File search tool, maps to `Tool.file_search`.
    FileSearch,
}
