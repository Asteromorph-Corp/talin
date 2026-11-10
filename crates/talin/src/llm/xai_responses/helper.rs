use talin_provider::xai_responses::types::{
    InputMessage, InputTextItem, ModelInputContent, ModelInputContentItem, ModelInputPart, Role,
};

/// Builds a plain-text user message as a single request item.
///
/// Used for the crate's own scaffolding prompts. They carry no media, so they are constructed
/// directly rather than routed through the fallible
/// [`from_message`](crate::llm::LlmRequestItem::from_message) conversion.
pub(super) fn text_user_parts(text: impl Into<String>) -> Vec<ModelInputPart> {
    let content = ModelInputContent::Parts(vec![ModelInputContentItem::InputText(
        InputTextItem::new(text.into()),
    )]);
    vec![ModelInputPart::Message(InputMessage::new(
        content,
        Role::User,
    ))]
}
