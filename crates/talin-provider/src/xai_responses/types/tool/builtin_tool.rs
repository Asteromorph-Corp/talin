use serde::{Deserialize, Serialize};

use super::{
    CodeInterpreterTool, FileSearchTool, ImageGenerationTool, ModelTool, ShellTool, WebSearchTool,
    XSearchTool,
};

/// A built-in tool of the xAI Responses API, i.e. every [`ModelTool`] variant except the
/// user-defined [`ModelTool::Function`] and the MCP-provided [`ModelTool::Mcp`].
///
/// The tags here match [`ModelTool`]'s exactly, so this only ever narrows a [`ModelTool`]; it
/// never disagrees with it about what a tag means.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum BuiltinTool {
    /// Search the web.
    WebSearch(WebSearchTool),
    /// Search X.
    XSearch(XSearchTool),
    /// Generate images from text prompts.
    ImageGeneration(ImageGenerationTool),
    /// Search the knowledge bases.
    FileSearch(FileSearchTool),
    /// Execute code.
    CodeInterpreter(CodeInterpreterTool),
    /// A local shell execution tool.
    Shell(ShellTool),
}

impl From<WebSearchTool> for BuiltinTool {
    fn from(tool: WebSearchTool) -> Self {
        BuiltinTool::WebSearch(tool)
    }
}
impl From<XSearchTool> for BuiltinTool {
    fn from(tool: XSearchTool) -> Self {
        BuiltinTool::XSearch(tool)
    }
}
impl From<ImageGenerationTool> for BuiltinTool {
    fn from(tool: ImageGenerationTool) -> Self {
        BuiltinTool::ImageGeneration(tool)
    }
}
impl From<FileSearchTool> for BuiltinTool {
    fn from(tool: FileSearchTool) -> Self {
        BuiltinTool::FileSearch(tool)
    }
}
impl From<CodeInterpreterTool> for BuiltinTool {
    fn from(tool: CodeInterpreterTool) -> Self {
        BuiltinTool::CodeInterpreter(tool)
    }
}
impl From<ShellTool> for BuiltinTool {
    fn from(tool: ShellTool) -> Self {
        BuiltinTool::Shell(tool)
    }
}

impl From<BuiltinTool> for ModelTool {
    fn from(tool: BuiltinTool) -> Self {
        match tool {
            BuiltinTool::WebSearch(tool) => ModelTool::WebSearch(tool),
            BuiltinTool::XSearch(tool) => ModelTool::XSearch(tool),
            BuiltinTool::ImageGeneration(tool) => ModelTool::ImageGeneration(tool),
            BuiltinTool::FileSearch(tool) => ModelTool::FileSearch(tool),
            BuiltinTool::CodeInterpreter(tool) => ModelTool::CodeInterpreter(tool),
            BuiltinTool::Shell(tool) => ModelTool::Shell(tool),
        }
    }
}
