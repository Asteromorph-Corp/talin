use serde::{Deserialize, Serialize};

use crate::utils::tagged_enum;

mod member;

mod allowed_caller;
mod bash_tool;
mod browser_toolset;
mod builtin_tool;
mod code_execution_tool;
mod computer_toolset;
mod custom_tool;
mod memory_tool;
mod response_inclusion;
mod text_editor_tool;
mod tool_search_tool;
mod web_fetch_tool;
mod web_search_tool;

pub use allowed_caller::*;
pub use bash_tool::*;
pub use browser_toolset::*;
pub use builtin_tool::*;
pub use code_execution_tool::*;
pub use computer_toolset::*;
pub use custom_tool::*;
pub use memory_tool::*;
pub use response_inclusion::*;
pub use text_editor_tool::*;
pub use tool_search_tool::*;
pub use web_fetch_tool::*;
pub use web_search_tool::*;

#[cfg(feature = "mcp-client-2025-11-20")]
mod mcp_toolset;

#[cfg(feature = "mcp-client-2025-11-20")]
pub use mcp_toolset::{McpToolConfig, McpToolset};

tagged_enum! {
    /// A tool definition. Can be a custom tool or one of Claude's internal tools.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ToolUnion {
        /// A custom tool.
        @schema(title = "Tool")
        Custom(Tool),
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
        #[serde( rename = "tool_search_tool_regex_20251119", alias = "tool_search_tool_regex" )]
        @schema(title = "RegexToolSearchTool20251119")
        RegexToolSearch20251119(RegexToolSearchTool20251119),
        /// BM25 tool search tool (2025-11-19).
        #[serde( rename = "tool_search_tool_bm25_20251119", alias = "tool_search_tool_bm25" )]
        @schema(title = "Bm25ToolSearch20251119")
        Bm25ToolSearch20251119(Bm25ToolSearchTool20251119),
        /// A toolset provided by an MCP server.
        #[cfg(feature = "mcp-client-2025-11-20")]
        McpToolset(McpToolset),
    }
}
