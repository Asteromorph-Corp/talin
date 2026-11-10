use std::str::FromStr;

use serde::de::Visitor;
use serde::de::value::MapAccessDeserializer;
use serde::{Deserialize, Serialize};
use talin_provider::gemini_generate_content::types::{
    BuiltinTool, CodeExecution, ComputerUse, FileSearch, GoogleMaps, GoogleSearch,
    GoogleSearchRetrieval, UrlContext,
};

/// A built-in tool of the Gemini generateContent API, as configured in an agent's TOML file.
///
/// Wraps [`BuiltinTool`]; see
/// [`ClaudeMessagesBuiltinTool`](super::claude_messages::ClaudeMessagesBuiltinTool) for how the
/// string and map forms differ.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct GeminiGenerateContentBuiltinTool(pub BuiltinTool);

impl GeminiGenerateContentBuiltinTool {
    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string(&self.0).expect("derived Serialize is infallible")
    }
}

impl FromStr for GeminiGenerateContentBuiltinTool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tool: BuiltinTool = match s {
            "google_search_retrieval" => GoogleSearchRetrieval::default().into(),
            "code_execution" => CodeExecution::default().into(),
            "google_search" => GoogleSearch::default().into(),
            "computer_use" => ComputerUse::default().into(),
            "url_context" => UrlContext::default().into(),
            "file_search" => FileSearch::default().into(),
            "google_maps" => GoogleMaps::default().into(),
            _ => return Err(format!("unknown tool: {s}")),
        };
        Ok(Self(tool))
    }
}

impl<'de> Deserialize<'de> for GeminiGenerateContentBuiltinTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(GeminiGenerateContentBuiltinToolVisitor)
    }
}

struct GeminiGenerateContentBuiltinToolVisitor;

impl<'de> Visitor<'de> for GeminiGenerateContentBuiltinToolVisitor {
    type Value = GeminiGenerateContentBuiltinTool;

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
        BuiltinTool::deserialize(MapAccessDeserializer::new(map))
            .map(GeminiGenerateContentBuiltinTool)
    }
}
