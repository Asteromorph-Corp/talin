use std::str::FromStr;

use serde::de::Visitor;
use serde::de::value::MapAccessDeserializer;
use serde::{Deserialize, Serialize};
use talin_provider::openai_responses::types::{
    CodeInterpreterTool, ComputerTool, ComputerUsePreviewTool, FileSearchTool,
    FunctionShellToolParam, ImageGenTool, ToolSearchToolParam, WebSearchTool,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", remote = "Self", rename_all = "snake_case")]
pub(crate) enum OpenAiResponsesBuiltinTool {
    FileSearch(FileSearchTool),
    ComputerUsePreview(ComputerUsePreviewTool),
    #[serde(
        alias = "web_search_2025_08_26",
        alias = "web_search_preview",
        alias = "web_search_preview_2025_03_11"
    )]
    WebSearch(WebSearchTool),
    CodeInterpreter(CodeInterpreterTool),
    ImageGeneration(ImageGenTool),
    LocalShell,
    Shell(FunctionShellToolParam),
    Computer(ComputerTool),
    ToolSearch(ToolSearchToolParam),
    ApplyPatch,
}

impl OpenAiResponsesBuiltinTool {
    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string(self).expect("derived Serialize is infallible")
    }
}

impl FromStr for OpenAiResponsesBuiltinTool {
    type Err = String;

    /// Parses a built-in tool from its name.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tool = match s {
            "file_search" => OpenAiResponsesBuiltinTool::FileSearch(FileSearchTool::default()),
            "computer_use_preview" => {
                OpenAiResponsesBuiltinTool::ComputerUsePreview(ComputerUsePreviewTool::default())
            }
            "web_search"
            | "web_search_2025_08_26"
            | "web_search_preview"
            | "web_search_preview_2025_03_11" => {
                OpenAiResponsesBuiltinTool::WebSearch(WebSearchTool::default())
            }
            "code_interpreter" => {
                OpenAiResponsesBuiltinTool::CodeInterpreter(CodeInterpreterTool::default())
            }
            "image_generation" => {
                OpenAiResponsesBuiltinTool::ImageGeneration(ImageGenTool::default())
            }
            "local_shell" => OpenAiResponsesBuiltinTool::LocalShell,
            "shell" => OpenAiResponsesBuiltinTool::Shell(FunctionShellToolParam::default()),
            "computer" => OpenAiResponsesBuiltinTool::Computer(ComputerTool::default()),
            "tool_search" => OpenAiResponsesBuiltinTool::ToolSearch(ToolSearchToolParam::default()),
            "apply_patch" => OpenAiResponsesBuiltinTool::ApplyPatch,
            _ => return Err(format!("unknown tool: {s}")),
        };
        Ok(tool)
    }
}

impl Serialize for OpenAiResponsesBuiltinTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Self::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for OpenAiResponsesBuiltinTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(OpenAiResponsesBuiltinToolVisitor)
    }
}

struct OpenAiResponsesBuiltinToolVisitor;

impl<'de> Visitor<'de> for OpenAiResponsesBuiltinToolVisitor {
    type Value = OpenAiResponsesBuiltinTool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string or a map")
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
        OpenAiResponsesBuiltinTool::deserialize(MapAccessDeserializer::new(map))
    }
}
