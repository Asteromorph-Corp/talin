use serde::{Deserialize, Serialize};

use super::{
    CodeExecution, ComputerUse, FileSearch, GoogleMaps, GoogleSearch, GoogleSearchRetrieval, Tool,
    UrlContext,
};

/// A built-in tool of the Gemini generateContent API.
///
/// [`Tool`] itself is a struct of optional fields rather than a tagged union, since the wire
/// format lets one request enable several of these at once; this names each of those fields on
/// its own, for a caller that wants exactly one.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum BuiltinTool {
    /// Google Search Retrieval tool.
    GoogleSearchRetrieval(GoogleSearchRetrieval),

    /// Code Execution tool.
    CodeExecution(CodeExecution),

    /// Google Search tool.
    GoogleSearch(GoogleSearch),

    /// Computer Use tool.
    ComputerUse(ComputerUse),

    /// URL Context tool.
    UrlContext(UrlContext),

    /// File Search tool.
    FileSearch(FileSearch),

    /// Google Maps tool.
    GoogleMaps(GoogleMaps),
}

impl From<GoogleSearchRetrieval> for BuiltinTool {
    fn from(tool: GoogleSearchRetrieval) -> Self {
        BuiltinTool::GoogleSearchRetrieval(tool)
    }
}
impl From<CodeExecution> for BuiltinTool {
    fn from(tool: CodeExecution) -> Self {
        BuiltinTool::CodeExecution(tool)
    }
}
impl From<GoogleSearch> for BuiltinTool {
    fn from(tool: GoogleSearch) -> Self {
        BuiltinTool::GoogleSearch(tool)
    }
}
impl From<ComputerUse> for BuiltinTool {
    fn from(tool: ComputerUse) -> Self {
        BuiltinTool::ComputerUse(tool)
    }
}
impl From<UrlContext> for BuiltinTool {
    fn from(tool: UrlContext) -> Self {
        BuiltinTool::UrlContext(tool)
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

impl From<BuiltinTool> for Tool {
    fn from(tool: BuiltinTool) -> Self {
        match tool {
            BuiltinTool::GoogleSearchRetrieval(t) => Tool::new().with_google_search_retrieval(t),
            BuiltinTool::CodeExecution(_) => Tool::new().with_code_execution(),
            BuiltinTool::GoogleSearch(t) => Tool::new().with_google_search(t),
            BuiltinTool::ComputerUse(t) => Tool::new().with_computer_use(t),
            BuiltinTool::UrlContext(t) => Tool::new().with_url_context(t),
            BuiltinTool::FileSearch(t) => Tool::new().with_file_search(t),
            BuiltinTool::GoogleMaps(t) => Tool::new().with_google_maps(t),
        }
    }
}
