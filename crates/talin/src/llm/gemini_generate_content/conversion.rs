use talin_provider::gemini_generate_content::types::{
    Content, Data, FunctionCall, FunctionCallingConfig, FunctionDeclaration, Part, ThinkingConfig,
    ThinkingLevel, ToolConfig,
};

use crate::llm::UnsupportedContent;
use crate::schemas::{
    AssistantMessage, BodyMessage, ContentPart, LlmStep, MediaContent, MediaSource,
    ReasoningEffort, ToolCall, ToolCallMessage, ToolChoice, ToolMessage, UserMessage,
};
use crate::tools::FunctionSpec;

impl From<ToolCall> for FunctionCall {
    fn from(call: ToolCall) -> Self {
        FunctionCall::new(call.name)
            .with_id(call.id)
            .with_args(call.arguments)
    }
}

impl From<FunctionCall> for ToolCall {
    fn from(call: FunctionCall) -> Self {
        ToolCall::new(call.id, call.name, call.args)
    }
}

impl TryFrom<BodyMessage> for Content {
    type Error = UnsupportedContent;

    fn try_from(message: BodyMessage) -> Result<Self, Self::Error> {
        match message {
            BodyMessage::User(user_message) => Content::try_from(user_message),
            BodyMessage::Assistant(assistant_message) => Content::try_from(assistant_message),
            BodyMessage::ToolCall(tool_call_message) => Content::try_from(tool_call_message),
            BodyMessage::Tool(tool_message) => Ok(Content::from(tool_message)),
        }
    }
}

impl TryFrom<UserMessage> for Content {
    type Error = UnsupportedContent;

    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        let parts = message
            .content
            .into_iter()
            .map(Part::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Content::user_with_parts(parts))
    }
}

impl TryFrom<AssistantMessage> for Content {
    type Error = UnsupportedContent;

    fn try_from(message: AssistantMessage) -> Result<Self, Self::Error> {
        let parts = message
            .content
            .into_iter()
            .map(Part::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Content::model_with_parts(parts))
    }
}

impl TryFrom<ToolCallMessage> for Content {
    type Error = UnsupportedContent;

    fn try_from(message: ToolCallMessage) -> Result<Self, Self::Error> {
        let (message, tool_calls) = message.split();
        let item = Self::try_from(message)?;
        let calls = tool_calls.into_iter().map(|tool_call| {
            Part::function_call(Some(tool_call.id), tool_call.name, tool_call.arguments)
        });
        let parts = item.parts.into_iter().flatten().chain(calls).collect();
        Ok(Content::model_with_parts(parts))
    }
}

impl From<ToolMessage> for Content {
    fn from(message: ToolMessage) -> Self {
        let result = serde_json::to_value(&message.output.content).unwrap_or_default();
        let content_as_result_key = serde_json::json!({ "result": result });
        Content::user().with_part(Part::function_response(
            Some(message.call_id),
            message.name,
            content_as_result_key,
        ))
    }
}

impl TryFrom<ContentPart> for Part {
    type Error = UnsupportedContent;

    fn try_from(part: ContentPart) -> Result<Self, Self::Error> {
        match part {
            ContentPart::Text(text) => Ok(Part::text(text)),
            ContentPart::Image(image) => Part::try_from(image),
            ContentPart::Audio(audio) => Part::try_from(audio),
            ContentPart::Video(video) => Part::try_from(video),
            ContentPart::Document(document) => Part::try_from(document),
        }
    }
}

impl<M: std::fmt::Display> TryFrom<MediaContent<M>> for Part {
    type Error = UnsupportedContent;

    fn try_from(media: MediaContent<M>) -> Result<Self, Self::Error> {
        let mime = media.mime.to_string();
        match media.source {
            MediaSource::Bytes(raw) => Ok(Part::inline_data(mime, raw.to_vec())),
            MediaSource::FileId(file_id) => Ok(Part::file_data(mime, file_id)),
            MediaSource::Url(_) => Err(UnsupportedContent::new(
                "Gemini `/generateContent`",
                "media URLs; upload the media or inline its bytes instead",
            )),
        }
    }
}

impl From<Content> for LlmStep {
    fn from(content: Content) -> Self {
        let mut thoughts = Vec::new();
        let mut texts = Vec::new();
        let mut tool_calls = Vec::new();

        if let Some(parts) = content.parts {
            for part in parts {
                match part.data {
                    Data::Text(text) => {
                        if part.thought.unwrap_or(false) {
                            thoughts.push(text.clone());
                        } else {
                            texts.push(text.clone());
                        }
                    }
                    Data::FunctionCall(function_call) => {
                        tool_calls.push(ToolCall::from(function_call));
                    }
                    _ => {}
                }
            }
        }
        LlmStep::from_parts(thoughts, texts, tool_calls)
    }
}

impl From<FunctionSpec> for FunctionDeclaration {
    fn from(spec: FunctionSpec) -> Self {
        FunctionDeclaration::new(spec.name, spec.description.unwrap_or_default())
            .with_parameters(spec.parameters)
    }
}

impl From<ReasoningEffort> for ThinkingConfig {
    fn from(effort: ReasoningEffort) -> Self {
        let thinking_level = match effort {
            ReasoningEffort::None => ThinkingLevel::Minimal,
            ReasoningEffort::Minimal => ThinkingLevel::Minimal,
            ReasoningEffort::Low => ThinkingLevel::Low,
            ReasoningEffort::Medium => ThinkingLevel::Medium,
            ReasoningEffort::High => ThinkingLevel::High,
            ReasoningEffort::Xhigh => ThinkingLevel::High,
            ReasoningEffort::Max => ThinkingLevel::High,
        };

        let mut config = ThinkingConfig::default()
            .with_include_thoughts(true)
            .with_thinking_level(thinking_level);

        if effort == ReasoningEffort::None {
            config = config.with_thinking_budget(0);
        }
        config
    }
}

impl From<ToolChoice> for ToolConfig {
    fn from(choice: ToolChoice) -> Self {
        let function_calling_config = match choice {
            ToolChoice::None => FunctionCallingConfig::none(),
            ToolChoice::Auto => FunctionCallingConfig::auto(),
            ToolChoice::Required => FunctionCallingConfig::any(),
            ToolChoice::Function(name) | ToolChoice::Mcp(name, _) => {
                FunctionCallingConfig::any().with_allowed_function(name)
            }
            ToolChoice::ProviderSpecific(id) => {
                log::warn!("Provider specific tool choice {id} not recognized, defaulting to auto");
                FunctionCallingConfig::auto()
            }
        };

        ToolConfig::new().with_function_calling_config(function_calling_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::ImageMimeType;

    #[test]
    fn rejects_image_url_over_silent_drop() {
        let image = MediaSource::from_url("https://e.com/a.png").image(ImageMimeType::Png);
        assert!(Part::try_from(image).is_err());
    }
}
