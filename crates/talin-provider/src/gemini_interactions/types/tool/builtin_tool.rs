use serde::{Deserialize, Serialize};

use super::{CodeExecution, ComputerUse, FileSearch, GoogleMaps, GoogleSearch, Tool, UrlContext};

/// A built-in tool of the Gemini Interactions API, i.e. every [`Tool`] variant except the
/// user-defined [`Tool::Function`] and the MCP-provided [`Tool::McpServer`].
///
/// The tags here match [`Tool`]'s exactly, so this only ever narrows a [`Tool`]; it never
/// disagrees with it about what a tag means.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum BuiltinTool {
    /// Enables the model to execute code as part of generation.
    CodeExecution(CodeExecution),
    /// Enables the model to fetch URL context.
    UrlContext(UrlContext),
    /// Enables the model to ground responses with Google Search.
    GoogleSearch(GoogleSearch),
    /// Enables the model to retrieve knowledge from file search stores.
    FileSearch(FileSearch),
    /// Enables the model to ground responses with Google Maps.
    GoogleMaps(GoogleMaps),
    /// Enables the model to interact with a computer.
    ComputerUse(ComputerUse),
}

impl From<CodeExecution> for BuiltinTool {
    fn from(tool: CodeExecution) -> Self {
        BuiltinTool::CodeExecution(tool)
    }
}
impl From<UrlContext> for BuiltinTool {
    fn from(tool: UrlContext) -> Self {
        BuiltinTool::UrlContext(tool)
    }
}
impl From<GoogleSearch> for BuiltinTool {
    fn from(tool: GoogleSearch) -> Self {
        BuiltinTool::GoogleSearch(tool)
    }
}
impl From<FileSearch> for BuiltinTool {
    fn from(tool: FileSearch) -> Self {
        BuiltinTool::FileSearch(tool)
    }
}
impl From<GoogleMaps> for BuiltinTool {
    fn from(tool: GoogleMaps) -> Self {
        BuiltinTool::GoogleMaps(tool)
    }
}
impl From<ComputerUse> for BuiltinTool {
    fn from(tool: ComputerUse) -> Self {
        BuiltinTool::ComputerUse(tool)
    }
}

impl From<BuiltinTool> for Tool {
    fn from(tool: BuiltinTool) -> Self {
        match tool {
            BuiltinTool::CodeExecution(t) => Tool::CodeExecution(t),
            BuiltinTool::UrlContext(t) => Tool::UrlContext(t),
            BuiltinTool::GoogleSearch(t) => Tool::GoogleSearch(t),
            BuiltinTool::FileSearch(t) => Tool::FileSearch(t),
            BuiltinTool::GoogleMaps(t) => Tool::GoogleMaps(t),
            BuiltinTool::ComputerUse(t) => Tool::ComputerUse(t),
        }
    }
}
