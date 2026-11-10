//! Read-only queries over the entries of a transcript.
//!
//! Used by [`Function::available`](crate::tools::Function::available) implementations to
//! determine whether a tool is available given the conversation so far.

use crate::schemas::BodyMessage;
use crate::tools::normalize_tool_name;
use crate::transcript::EntryView;

/// Counts the completed calls to the tool named `tool_name` in `views`, whether they succeeded
/// or failed.
pub fn count_tool_calls(views: &[EntryView<'_>], tool_name: &str) -> usize {
    let name = normalize_tool_name(tool_name);
    views
        .iter()
        .filter(|view| match view {
            EntryView::Message(_, BodyMessage::Tool(m)) => normalize_tool_name(&m.name) == name,
            EntryView::ToolError(_, m) => normalize_tool_name(&m.name) == name,
            _ => false,
        })
        .count()
}

/// Counts the successful calls to the tool named `tool_name` in `views`.
pub fn count_successful_tool_calls(views: &[EntryView<'_>], tool_name: &str) -> usize {
    let name = normalize_tool_name(tool_name);
    views
        .iter()
        .filter(|view| {
            matches!(view,
                EntryView::Message(_, BodyMessage::Tool(m)) if normalize_tool_name(&m.name) == name)
        })
        .count()
}

/// Whether `views` contain any tool call.
pub(crate) fn has_tool_calls(views: &[EntryView<'_>]) -> bool {
    views
        .iter()
        .any(|view| matches!(view, EntryView::Message(_, BodyMessage::ToolCall(_))))
}
