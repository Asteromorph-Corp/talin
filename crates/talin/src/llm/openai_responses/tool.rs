use serde::{Deserialize, Serialize};
use talin_provider::openai_responses::types::{
    ApplyPatchToolParam, CodeInterpreterTool, ComputerUsePreviewTool, FileSearchTool,
    FunctionShellToolParam, ImageGenTool, Tool, WebSearchTool,
};

use crate::llm::LlmBuiltinTool;

/// Definitions for built-in tools for the responses API.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResponsesBuiltinTool {
    /// A tool that searches for relevant content from uploaded files.
    /// Learn more about the [file search tool](https://platform.openai.com/docs/guides/tools-file-search).
    FileSearch(FileSearchTool),
    /// A tool that controls a virtual computer.
    /// Learn more about the [computer use tool](https://platform.openai.com/docs/guides/tools-computer-use).
    ComputerUsePreview(ComputerUsePreviewTool),
    /// Search the Internet for sources related to the prompt.
    /// Learn more about the [web search tool](https://platform.openai.com/docs/guides/tools-web-search).
    WebSearch(WebSearchTool),
    /// A tool that runs Python code to help generate a response to a prompt.
    CodeInterpreter(CodeInterpreterTool),
    /// A tool that generates images using a model like `gpt-image-1`.
    ImageGeneration(ImageGenTool),
    /// A tool that allows the model to execute shell commands in a local environment.
    LocalShell,
    /// A tool that allows the model to execute shell commands.
    Shell,
    /// This tool searches the web for relevant results to use in a response.
    /// Learn more about the [web search tool](https://platform.openai.com/docs/guides/tools-web-search).
    WebSearchPreview(WebSearchTool),
    /// Allows the assistant to create, delete, or update files using unified diffs.
    ApplyPatch,
}

impl LlmBuiltinTool for ResponsesBuiltinTool {}

impl From<FileSearchTool> for ResponsesBuiltinTool {
    fn from(tool: FileSearchTool) -> Self {
        ResponsesBuiltinTool::FileSearch(tool)
    }
}
impl From<ComputerUsePreviewTool> for ResponsesBuiltinTool {
    fn from(tool: ComputerUsePreviewTool) -> Self {
        ResponsesBuiltinTool::ComputerUsePreview(tool)
    }
}
impl From<WebSearchTool> for ResponsesBuiltinTool {
    fn from(tool: WebSearchTool) -> Self {
        ResponsesBuiltinTool::WebSearch(tool)
    }
}
impl From<CodeInterpreterTool> for ResponsesBuiltinTool {
    fn from(tool: CodeInterpreterTool) -> Self {
        ResponsesBuiltinTool::CodeInterpreter(tool)
    }
}
impl From<ImageGenTool> for ResponsesBuiltinTool {
    fn from(tool: ImageGenTool) -> Self {
        ResponsesBuiltinTool::ImageGeneration(tool)
    }
}

impl From<ResponsesBuiltinTool> for Tool {
    fn from(tool: ResponsesBuiltinTool) -> Self {
        match tool {
            ResponsesBuiltinTool::FileSearch(tool) => Tool::FileSearch(tool),
            ResponsesBuiltinTool::ComputerUsePreview(tool) => Tool::ComputerUsePreview(tool),
            ResponsesBuiltinTool::WebSearch(tool) => Tool::WebSearch(tool),
            ResponsesBuiltinTool::CodeInterpreter(tool) => Tool::CodeInterpreter(tool),
            ResponsesBuiltinTool::ImageGeneration(tool) => Tool::ImageGeneration(tool),
            ResponsesBuiltinTool::LocalShell => Tool::LocalShell,
            ResponsesBuiltinTool::Shell => Tool::Shell(FunctionShellToolParam {
                environment: None,
                allowed_callers: None,
            }),
            ResponsesBuiltinTool::WebSearchPreview(tool) => Tool::WebSearchPreview(tool),
            ResponsesBuiltinTool::ApplyPatch => Tool::ApplyPatch(ApplyPatchToolParam::default()),
        }
    }
}
