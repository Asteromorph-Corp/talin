//! Native Claude compaction wire fidelity and usage contracts.

use serde_json::{Value, json};
use talin_provider::claude_messages::types::*;

#[test]
fn request_serializes_both_compaction_modes() {
    let message = MessageParam::user("Hello");
    let on_demand = MessagesRequest::new("claude-opus-5-5", vec![message.clone()], 4096)
        .with_compaction(CompactionConfig::new().with_instructions("Preserve the decisions"));
    let value = serde_json::to_value(on_demand).unwrap();
    assert_eq!(
        value["compaction"],
        json!({
            "type": "summarize",
            "instructions": "Preserve the decisions"
        })
    );
    assert!(value.get("context_management").is_none());

    let edits = vec![ContextManagementEdit::Compact(
        CompactEdit::new()
            .with_trigger(InputTokensThreshold::new(150_000))
            .with_pause_after_compaction(true),
    )];
    let managed = MessagesRequest::new("claude-opus-5-5", vec![message], 4096)
        .with_context_management(ContextManagementConfig::new(edits));
    let value = serde_json::to_value(managed).unwrap();
    assert_eq!(
        value["context_management"]["edits"],
        json!([
            {"type": "compact_20260112", "trigger": {"type": "input_tokens", "value": 150000}, "pause_after_compaction": true}
        ])
    );
    assert!(value.get("compaction").is_none());

    let default_edit =
        serde_json::to_value(ContextManagementEdit::Compact(CompactEdit::new())).unwrap();
    assert_eq!(default_edit, json!({"type": "compact_20260112"}));
    assert_eq!(
        serde_json::to_value(CompactionConfig::new()).unwrap(),
        json!({"type": "summarize"})
    );
}

#[test]
fn compaction_block_round_trips_into_a_request() {
    let tool_changes = json!([
        {"type": "tool_removal", "tool": {"type": "tool_reference", "name": "search"}},
        {"type": "tool_addition", "tool": {"type": "mcp_tool_reference", "name": "query", "server_name": "db"}}
    ]);
    let block: ContentBlock = serde_json::from_value(json!({
        "type": "compaction",
        "content": null,
        "encrypted_content": "opaque",
        "signature": "signed",
        "tool_changes": tool_changes
    }))
    .unwrap();
    let param: ContentBlockParam = block.into();
    let value = serde_json::to_value(param).unwrap();
    assert_eq!(value["type"], "compaction");
    assert_eq!(value["content"], Value::Null);
    assert_eq!(value["signature"], "signed");
    assert_eq!(value["tool_changes"], tool_changes);
    assert!(value.get("cache_control").is_none());
}

#[test]
fn compaction_tool_changes_preserve_empty_null_and_absent() {
    for input in [
        json!({"type": "compaction", "content": "summary", "tool_changes": []}),
        json!({"type": "compaction", "content": "summary", "tool_changes": null}),
        json!({"type": "compaction", "content": "summary"}),
    ] {
        let block: ContentBlock = serde_json::from_value(input.clone()).unwrap();
        let param: ContentBlockParam = block.into();
        let output = serde_json::to_value(param).unwrap();
        assert_eq!(output.get("tool_changes"), input.get("tool_changes"));
    }
}
