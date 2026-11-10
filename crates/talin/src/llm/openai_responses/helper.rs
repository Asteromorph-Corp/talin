use talin_provider::openai_responses::types::{
    InputContent, InputItem, InputMessage, InputRole, InputTextContent, Item, MessageItem, Role,
};

/// Whether the record contains a compaction item.
pub(super) fn has_compaction(record: &[InputItem]) -> bool {
    record
        .iter()
        .any(|item| matches!(item, InputItem::Item(Item::Compaction(_))))
}

/// Keeps the latest compacted native window and all subsequent records.
/// Retained items within a compacted window must be sent unchanged.
pub(super) fn discard_compacted(items: &mut Vec<Vec<InputItem>>) {
    if let Some(start) = items.iter().rposition(|record| has_compaction(record)) {
        items.drain(..start);
    }
}

/// Whether the item is a system or developer message.
pub(crate) fn is_system_item(item: &InputItem) -> bool {
    match item {
        InputItem::Item(Item::Message(MessageItem::Input(message))) => {
            matches!(message.role, InputRole::System | InputRole::Developer)
        }
        InputItem::EasyMessage(message) => {
            matches!(message.role, Role::System | Role::Developer)
        }
        _ => false,
    }
}

/// Constructs a system [`InputMessage`] with the given contents.
pub(super) fn system_message(contents: Vec<String>) -> Option<InputMessage> {
    if contents.is_empty() {
        return None;
    }

    let content = contents
        .into_iter()
        .map(|text| {
            InputContent::InputText(InputTextContent {
                text,
                prompt_cache_breakpoint: None,
            })
        })
        .collect::<Vec<_>>();
    Some(InputMessage {
        role: InputRole::System,
        content,
        status: None,
    })
}

/// Builds a plain-text user message as a single input item.
pub(super) fn text_user_item(text: impl Into<String>) -> InputItem {
    let content = vec![InputContent::InputText(InputTextContent {
        text: text.into(),
        prompt_cache_breakpoint: None,
    })];
    let message = InputMessage {
        role: InputRole::User,
        content,
        status: None,
    };
    InputItem::Item(Item::Message(MessageItem::Input(message)))
}
