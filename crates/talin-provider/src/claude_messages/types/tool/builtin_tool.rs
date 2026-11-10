use serde::{Deserialize, Serialize};

use super::{
    BashTool20250124, Bm25ToolSearchTool20251119, BrowserToolset20260801,
    CodeExecutionTool20250522, CodeExecutionTool20250825, CodeExecutionTool20260120,
    CodeExecutionTool20260521, ComputerToolset20260801, MemoryTool20250818,
    RegexToolSearchTool20251119, TextEditorTool20250124, TextEditorTool20250429,
    TextEditorTool20250728, ToolUnion, WebFetchTool20250910, WebFetchTool20260209,
    WebFetchTool20260309, WebFetchTool20260318, WebSearchTool20250305, WebSearchTool20260209,
    WebSearchTool20260318,
};

/// A built-in tool of the Claude Messages API, i.e. every [`ToolUnion`] variant except
/// [`ToolUnion::Custom`] and the MCP-provided [`ToolUnion::McpToolset`].
///
/// The tags here match [`ToolUnion`]'s exactly, so this only ever narrows a [`ToolUnion`]; it
/// never disagrees with it about what a tag means.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum BuiltinTool {
    /// Bash tool (2025-01-24).
    #[serde(rename = "bash_20250124")]
    Bash20250124(BashTool20250124),

    /// Code execution tool (2025-05-22).
    #[serde(rename = "code_execution_20250522")]
    CodeExecution20250522(CodeExecutionTool20250522),

    /// Code execution tool (2025-08-25).
    #[serde(rename = "code_execution_20250825")]
    CodeExecution20250825(CodeExecutionTool20250825),

    /// Code execution tool (2026-01-20).
    #[serde(rename = "code_execution_20260120")]
    CodeExecution20260120(CodeExecutionTool20260120),

    /// Code execution tool (2026-05-21).
    #[serde(rename = "code_execution_20260521")]
    CodeExecution20260521(CodeExecutionTool20260521),

    /// Text editor tool (2025-01-24).
    #[serde(rename = "text_editor_20250124")]
    TextEditor20250124(TextEditorTool20250124),

    /// Text editor tool (2025-04-29).
    #[serde(rename = "text_editor_20250429")]
    TextEditor20250429(TextEditorTool20250429),

    /// Text editor tool (2025-07-28).
    #[serde(rename = "text_editor_20250728")]
    TextEditor20250728(TextEditorTool20250728),

    /// Web fetch tool (2025-09-10).
    #[serde(rename = "web_fetch_20250910")]
    WebFetch20250910(WebFetchTool20250910),

    /// Web fetch tool (2026-02-09).
    #[serde(rename = "web_fetch_20260209")]
    WebFetch20260209(WebFetchTool20260209),

    /// Web fetch tool (2026-03-09).
    #[serde(rename = "web_fetch_20260309")]
    WebFetch20260309(WebFetchTool20260309),

    /// Web fetch tool (2026-03-18).
    #[serde(rename = "web_fetch_20260318")]
    WebFetch20260318(WebFetchTool20260318),

    /// Web search tool (2025-03-05).
    #[serde(rename = "web_search_20250305")]
    WebSearch20250305(WebSearchTool20250305),

    /// Web search tool (2026-02-09).
    #[serde(rename = "web_search_20260209")]
    WebSearch20260209(WebSearchTool20260209),

    /// Web search tool (2026-03-18).
    #[serde(rename = "web_search_20260318")]
    WebSearch20260318(WebSearchTool20260318),

    /// Browser toolset (2026-08-01).
    #[serde(rename = "browser_toolset_20260801")]
    BrowserToolset20260801(BrowserToolset20260801),

    /// Computer toolset (2026-08-01).
    #[serde(rename = "computer_toolset_20260801")]
    ComputerToolset20260801(ComputerToolset20260801),

    /// Memory tool (2025-08-18).
    #[serde(rename = "memory_20250818")]
    Memory20250818(MemoryTool20250818),

    /// Regex tool search tool (2025-11-19).
    #[serde(
        rename = "tool_search_tool_regex_20251119",
        alias = "tool_search_tool_regex"
    )]
    RegexToolSearch20251119(RegexToolSearchTool20251119),

    /// BM25 tool search tool (2025-11-19).
    #[serde(
        rename = "tool_search_tool_bm25_20251119",
        alias = "tool_search_tool_bm25"
    )]
    Bm25ToolSearch20251119(Bm25ToolSearchTool20251119),
}

impl From<BashTool20250124> for BuiltinTool {
    fn from(tool: BashTool20250124) -> Self {
        BuiltinTool::Bash20250124(tool)
    }
}
impl From<CodeExecutionTool20250522> for BuiltinTool {
    fn from(tool: CodeExecutionTool20250522) -> Self {
        BuiltinTool::CodeExecution20250522(tool)
    }
}
impl From<CodeExecutionTool20250825> for BuiltinTool {
    fn from(tool: CodeExecutionTool20250825) -> Self {
        BuiltinTool::CodeExecution20250825(tool)
    }
}
impl From<CodeExecutionTool20260120> for BuiltinTool {
    fn from(tool: CodeExecutionTool20260120) -> Self {
        BuiltinTool::CodeExecution20260120(tool)
    }
}
impl From<CodeExecutionTool20260521> for BuiltinTool {
    fn from(tool: CodeExecutionTool20260521) -> Self {
        BuiltinTool::CodeExecution20260521(tool)
    }
}
impl From<TextEditorTool20250124> for BuiltinTool {
    fn from(tool: TextEditorTool20250124) -> Self {
        BuiltinTool::TextEditor20250124(tool)
    }
}
impl From<TextEditorTool20250429> for BuiltinTool {
    fn from(tool: TextEditorTool20250429) -> Self {
        BuiltinTool::TextEditor20250429(tool)
    }
}
impl From<TextEditorTool20250728> for BuiltinTool {
    fn from(tool: TextEditorTool20250728) -> Self {
        BuiltinTool::TextEditor20250728(tool)
    }
}
impl From<WebFetchTool20250910> for BuiltinTool {
    fn from(tool: WebFetchTool20250910) -> Self {
        BuiltinTool::WebFetch20250910(tool)
    }
}
impl From<WebFetchTool20260209> for BuiltinTool {
    fn from(tool: WebFetchTool20260209) -> Self {
        BuiltinTool::WebFetch20260209(tool)
    }
}
impl From<WebFetchTool20260309> for BuiltinTool {
    fn from(tool: WebFetchTool20260309) -> Self {
        BuiltinTool::WebFetch20260309(tool)
    }
}
impl From<WebFetchTool20260318> for BuiltinTool {
    fn from(tool: WebFetchTool20260318) -> Self {
        BuiltinTool::WebFetch20260318(tool)
    }
}
impl From<WebSearchTool20250305> for BuiltinTool {
    fn from(tool: WebSearchTool20250305) -> Self {
        BuiltinTool::WebSearch20250305(tool)
    }
}
impl From<WebSearchTool20260209> for BuiltinTool {
    fn from(tool: WebSearchTool20260209) -> Self {
        BuiltinTool::WebSearch20260209(tool)
    }
}
impl From<WebSearchTool20260318> for BuiltinTool {
    fn from(tool: WebSearchTool20260318) -> Self {
        BuiltinTool::WebSearch20260318(tool)
    }
}
impl From<BrowserToolset20260801> for BuiltinTool {
    fn from(tool: BrowserToolset20260801) -> Self {
        BuiltinTool::BrowserToolset20260801(tool)
    }
}
impl From<ComputerToolset20260801> for BuiltinTool {
    fn from(tool: ComputerToolset20260801) -> Self {
        BuiltinTool::ComputerToolset20260801(tool)
    }
}
impl From<MemoryTool20250818> for BuiltinTool {
    fn from(tool: MemoryTool20250818) -> Self {
        BuiltinTool::Memory20250818(tool)
    }
}
impl From<RegexToolSearchTool20251119> for BuiltinTool {
    fn from(tool: RegexToolSearchTool20251119) -> Self {
        BuiltinTool::RegexToolSearch20251119(tool)
    }
}
impl From<Bm25ToolSearchTool20251119> for BuiltinTool {
    fn from(tool: Bm25ToolSearchTool20251119) -> Self {
        BuiltinTool::Bm25ToolSearch20251119(tool)
    }
}

impl From<BuiltinTool> for ToolUnion {
    fn from(tool: BuiltinTool) -> Self {
        match tool {
            BuiltinTool::Bash20250124(tool) => ToolUnion::Bash20250124(tool),
            BuiltinTool::CodeExecution20250522(tool) => ToolUnion::CodeExecution20250522(tool),
            BuiltinTool::CodeExecution20250825(tool) => ToolUnion::CodeExecution20250825(tool),
            BuiltinTool::CodeExecution20260120(tool) => ToolUnion::CodeExecution20260120(tool),
            BuiltinTool::CodeExecution20260521(tool) => ToolUnion::CodeExecution20260521(tool),
            BuiltinTool::TextEditor20250124(tool) => ToolUnion::TextEditor20250124(tool),
            BuiltinTool::TextEditor20250429(tool) => ToolUnion::TextEditor20250429(tool),
            BuiltinTool::TextEditor20250728(tool) => ToolUnion::TextEditor20250728(tool),
            BuiltinTool::WebFetch20250910(tool) => ToolUnion::WebFetch20250910(tool),
            BuiltinTool::WebFetch20260209(tool) => ToolUnion::WebFetch20260209(tool),
            BuiltinTool::WebFetch20260309(tool) => ToolUnion::WebFetch20260309(tool),
            BuiltinTool::WebFetch20260318(tool) => ToolUnion::WebFetch20260318(tool),
            BuiltinTool::WebSearch20250305(tool) => ToolUnion::WebSearch20250305(tool),
            BuiltinTool::WebSearch20260209(tool) => ToolUnion::WebSearch20260209(tool),
            BuiltinTool::WebSearch20260318(tool) => ToolUnion::WebSearch20260318(tool),
            BuiltinTool::BrowserToolset20260801(tool) => ToolUnion::BrowserToolset20260801(tool),
            BuiltinTool::ComputerToolset20260801(tool) => ToolUnion::ComputerToolset20260801(tool),
            BuiltinTool::Memory20250818(tool) => ToolUnion::Memory20250818(tool),
            BuiltinTool::RegexToolSearch20251119(tool) => ToolUnion::RegexToolSearch20251119(tool),
            BuiltinTool::Bm25ToolSearch20251119(tool) => ToolUnion::Bm25ToolSearch20251119(tool),
        }
    }
}
