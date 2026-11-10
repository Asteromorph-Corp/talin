use indoc::indoc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::text::render::add_indent;

/// A tool call made by an LLM.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct ToolCall {
    /// The id of the tool call.
    ///
    /// If id is not provided by the provider, it is generated using `Uuid::new_v4`.
    pub id: String,
    /// The name of the tool to call.
    pub name: String,
    /// The arguments to pass to the tool.
    pub arguments: Value,
}

impl ToolCall {
    /// Constructs a new [`ToolCall`].
    ///
    /// If `id` is not provided, a new UUID will be generated.
    pub fn new(id: Option<String>, name: impl Into<String>, arguments: Option<Value>) -> Self {
        Self {
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: name.into(),
            arguments: arguments.unwrap_or_else(|| Value::Object(serde_json::Map::default())),
        }
    }
}

impl std::fmt::Display for ToolCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            indoc! {r#"
            {{
                "name": "{}",
                "arguments": {}
            }}"#},
            self.name,
            add_indent(
                &serde_json::to_string_pretty(&self.arguments)
                    .unwrap_or_else(|_| self.arguments.to_string()),
                4,
                false
            )
        )
    }
}
