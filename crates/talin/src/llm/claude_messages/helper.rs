use talin_provider::claude_messages::beta_headers::MCP_CLIENT;
use talin_provider::claude_messages::types::{
    ContentBlockParam, McpServer, McpToolset, MessageParam, RefusalStopDetails,
    RefusalStopDetailsInner, StopReason, TextBlockParam, TextOrItems, ToolUnion,
};

use crate::llm::LlmError;
use crate::llm::claude_messages::MessagesBuiltinTool;
use crate::mcp::McpServerHttp;
use crate::tools::ToolSpec;
use crate::utils::NoneIfEmpty;

const REFUSED_NO_DETAILS: &str = "No details provided";

/// Whether the block is a compaction carrying replacement context; a failed compaction is not.
pub(super) fn is_usable_compaction(block: &ContentBlockParam) -> bool {
    matches!(block, ContentBlockParam::Compaction(compaction) if
        compaction.content.as_ref().is_some_and(|text| !text.is_empty()) ||
        compaction.encrypted_content.as_ref().is_some_and(|text| !text.is_empty()))
}

/// Keeps the latest usable compaction block and all following native content.
pub(super) fn discard_compacted(messages: &mut Vec<MessageParam>) {
    let boundary = messages
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, message)| {
            let Some(TextOrItems::Items(blocks)) = message.content() else {
                return None;
            };
            let block = blocks.iter().rposition(is_usable_compaction)?;
            Some((index, block))
        });
    if let Some((message, block)) = boundary {
        messages.drain(..message);
        if let Some(TextOrItems::Items(blocks)) = messages[0].content_mut() {
            blocks.drain(..block);
        }
    }
}

pub(super) fn into_items(
    items: TextOrItems<ContentBlockParam>,
) -> impl Iterator<Item = ContentBlockParam> {
    match items {
        TextOrItems::Text(text) => vec![ContentBlockParam::Text(TextBlockParam::new(text))],
        TextOrItems::Items(items) => items,
        _ => Vec::new(),
    }
    .into_iter()
}

/// Maps the tool specifications to Claude tools and MCP servers.
pub(super) fn map_tools(
    tools: Vec<ToolSpec>,
    mcp_servers: Vec<McpServerHttp>,
    builtin_tools: Vec<MessagesBuiltinTool>,
    mcp_enabled: bool,
) -> (Option<Vec<ToolUnion>>, Option<Vec<McpServer>>) {
    let function_tools = ToolSpec::into_functions(tools)
        .into_iter()
        .map(ToolUnion::from);
    let builtin_tools = builtin_tools.into_iter().map(ToolUnion::from);

    if !mcp_servers.is_empty() && !mcp_enabled {
        log::warn!(r#"MCP tools provided, but "{MCP_CLIENT}" is not enabled"#);
        let tools = function_tools.chain(builtin_tools).collect::<Vec<_>>();
        return (tools.none_if_empty(), None);
    }

    let (mcp_servers, mcp_toolsets): (Vec<_>, Vec<_>) = mcp_servers
        .into_iter()
        .map(<(McpServer, McpToolset)>::from)
        .unzip();
    let mcp_toolsets = mcp_toolsets.into_iter().map(ToolUnion::McpToolset);
    let tools = function_tools
        .chain(builtin_tools)
        .chain(mcp_toolsets)
        .collect::<Vec<_>>();
    (tools.none_if_empty(), mcp_servers.none_if_empty())
}

pub(super) fn refusal_error(details: Option<RefusalStopDetailsInner>) -> LlmError {
    let details = details.map_or_else(
        || String::from(REFUSED_NO_DETAILS),
        |details| format!("{details:?}"),
    );
    LlmError::Refused(details)
}

pub(super) fn check_stop_reason(
    reason: StopReason,
    details: Option<RefusalStopDetails>,
) -> Result<(), LlmError> {
    match reason {
        StopReason::EndTurn => Ok(()),
        StopReason::MaxTokens => Ok(()),
        StopReason::StopSequence => Ok(()),
        StopReason::ToolUse => Ok(()),
        StopReason::PauseTurn => Ok(()),
        StopReason::Compaction => Ok(()),
        StopReason::Refusal => {
            let details = details.and_then(|details| match details {
                RefusalStopDetails::Refusal(details) => Some(details),
                _ => None,
            });
            Err(refusal_error(details))
        }
        // `Other` and any future variant we don't yet handle.
        _ => Err(LlmError::Other(format!(
            "Claude responded with unknown stop reason: {reason:?}"
        ))),
    }
}

/// Builds a plain-text user message.
pub(super) fn text_user_message(text: impl Into<String>) -> MessageParam {
    MessageParam::user(TextOrItems::Text(text.into()))
}
