use std::str::FromStr;

use serde::de::Visitor;
use serde::de::value::MapAccessDeserializer;
use serde::{Deserialize, Serialize};
use talin_provider::claude_messages::types::{
    BashTool20250124, Bm25ToolSearchTool20251119, BrowserToolset20260801, BuiltinTool,
    CodeExecutionTool20250522, CodeExecutionTool20250825, CodeExecutionTool20260120,
    CodeExecutionTool20260521, ComputerToolset20260801, MemoryTool20250818,
    RegexToolSearchTool20251119, TextEditorTool20250124, TextEditorTool20250429,
    TextEditorTool20250728, WebFetchTool20250910, WebFetchTool20260209, WebFetchTool20260309,
    WebFetchTool20260318, WebSearchTool20250305, WebSearchTool20260209, WebSearchTool20260318,
};

/// A built-in tool of the Claude Messages API, as configured in an agent's TOML file.
///
/// Wraps [`BuiltinTool`], which carries the current wire tags; this adds only the short,
/// undocumented legacy spellings kept for backwards compatibility (`"bash"` for
/// `"bash_20250124"`, and so on), each resolving to the version of that tool the field's
/// `FromStr` impl below names. Those legacy spellings are recognized for the bare-string form
/// (`builtin_tools = ["bash"]`); the map form (`[[builtin_tools]] type = "bash_20250124"`) takes
/// [`BuiltinTool`]'s own tags and aliases directly, so a tool this crate doesn't have a legacy
/// spelling for is still reachable that way.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct ClaudeMessagesBuiltinTool(pub BuiltinTool);

impl ClaudeMessagesBuiltinTool {
    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string(&self.0).expect("derived Serialize is infallible")
    }
}

impl FromStr for ClaudeMessagesBuiltinTool {
    type Err = String;

    /// Parses a built-in tool from its name.
    ///
    /// A name without a date suffix resolves to the latest available version of
    /// that tool.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tool: BuiltinTool = match s {
            "bash" | "bash_20250124" => BashTool20250124::default().into(),
            "code_execution_20250522" => CodeExecutionTool20250522::default().into(),
            "code_execution_20250825" => CodeExecutionTool20250825::default().into(),
            "code_execution_20260120" => CodeExecutionTool20260120::default().into(),
            "code_execution" | "code_execution_20260521" => {
                CodeExecutionTool20260521::default().into()
            }
            "text_editor_20250124" => TextEditorTool20250124::default().into(),
            "text_editor_20250429" => TextEditorTool20250429::default().into(),
            "text_editor" | "text_editor_20250728" => TextEditorTool20250728::default().into(),
            "web_fetch_20250910" => WebFetchTool20250910::default().into(),
            "web_fetch_20260209" => WebFetchTool20260209::default().into(),
            "web_fetch_20260309" => WebFetchTool20260309::default().into(),
            "web_fetch" | "web_fetch_20260318" => WebFetchTool20260318::default().into(),
            "web_search_20250305" => WebSearchTool20250305::default().into(),
            "web_search_20260209" => WebSearchTool20260209::default().into(),
            "web_search" | "web_search_20260318" => WebSearchTool20260318::default().into(),
            "browser_toolset" | "browser_toolset_20260801" => {
                BrowserToolset20260801::default().into()
            }
            "computer_toolset" | "computer_toolset_20260801" => {
                ComputerToolset20260801::default().into()
            }
            "memory" | "memory_20250818" => MemoryTool20250818::default().into(),
            "regex_tool_search"
            | "regex_tool_search_20251119"
            | "tool_search_tool_regex"
            | "tool_search_tool_regex_20251119" => RegexToolSearchTool20251119::default().into(),
            "bm25_tool_search"
            | "bm25_tool_search_20251119"
            | "tool_search_tool_bm25"
            | "tool_search_tool_bm25_20251119" => Bm25ToolSearchTool20251119::default().into(),
            _ => return Err(format!("unknown tool: {s}")),
        };
        Ok(Self(tool))
    }
}

impl<'de> Deserialize<'de> for ClaudeMessagesBuiltinTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ClaudeMessagesBuiltinToolVisitor)
    }
}

struct ClaudeMessagesBuiltinToolVisitor;

impl<'de> Visitor<'de> for ClaudeMessagesBuiltinToolVisitor {
    type Value = ClaudeMessagesBuiltinTool;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a string or map")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(E::custom)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        BuiltinTool::deserialize(MapAccessDeserializer::new(map)).map(ClaudeMessagesBuiltinTool)
    }
}
