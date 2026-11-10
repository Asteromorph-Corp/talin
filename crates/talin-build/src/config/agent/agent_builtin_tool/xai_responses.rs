use std::str::FromStr;

use serde::de::value::MapAccessDeserializer;
use serde::{Deserialize, Serialize};
use talin_provider::xai_responses::types::{
    BuiltinTool, CodeInterpreterTool, FileSearchTool, ImageGenerationTool, ShellEnvironment,
    ShellTool, WebSearchTool, XSearchTool,
};

/// A built-in tool of the xAI Responses API, as configured in an agent's TOML file.
///
/// Wraps [`BuiltinTool`]; see
/// [`ClaudeMessagesBuiltinTool`](super::claude_messages::ClaudeMessagesBuiltinTool) for how the
/// string and map forms differ.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct XaiResponsesBuiltinTool(pub BuiltinTool);

impl XaiResponsesBuiltinTool {
    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string(&self.0).expect("derived Serialize is infallible")
    }
}

impl FromStr for XaiResponsesBuiltinTool {
    type Err = String;

    /// Parses a built-in tool from its name.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tool: BuiltinTool = match s {
            "web_search" => WebSearchTool::default().into(),
            "x_search" => XSearchTool::default().into(),
            "image_generation" => ImageGenerationTool::default().into(),
            "file_search" => FileSearchTool::new(Vec::new()).into(),
            "code_interpreter" => CodeInterpreterTool::default().into(),
            "shell" => ShellTool::new(ShellEnvironment::local(Vec::new())).into(),
            _ => return Err(format!("unknown tool: {s}")),
        };

        Ok(Self(tool))
    }
}

impl<'de> Deserialize<'de> for XaiResponsesBuiltinTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(XaiResponsesBuiltinToolVisitor)
    }
}

struct XaiResponsesBuiltinToolVisitor;

impl<'de> serde::de::Visitor<'de> for XaiResponsesBuiltinToolVisitor {
    type Value = XaiResponsesBuiltinTool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a valid xAI built-in tool")
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
        BuiltinTool::deserialize(MapAccessDeserializer::new(map)).map(XaiResponsesBuiltinTool)
    }
}
