//! This module consists of conversion logic between the schema types of the crate and the types in
//! `talin_provider::openai_chat::types`.

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde_json::json;
use talin_provider::openai_chat::types::{
    ChatCompletionMessageCustomToolCall, ChatCompletionMessageToolCall,
    ChatCompletionMessageToolCalls, ChatCompletionNamedToolChoice,
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestAssistantMessageContentPart, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartAudio, ChatCompletionRequestMessageContentPartFile,
    ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionRequestToolMessage, ChatCompletionRequestToolMessageContent,
    ChatCompletionRequestToolMessageContentPart, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart,
    ChatCompletionResponseMessage, ChatCompletionTool, ChatCompletionToolChoiceOption,
    ChatCompletionTools, FileObject, FunctionCall, FunctionName, FunctionObject, ImageDetail,
    ImageUrl, InputAudio, InputAudioFormat, ReasoningEffort as OpenAiReasoningEffort,
    ResponseFormat as ChatCompletionResponseFormat, ResponseFormatJsonSchema, ToolChoiceOptions,
};

use crate::llm::openai_chat::helper::collect_texts;
use crate::llm::{LlmError, UnsupportedContent};
use crate::schemas::{
    AssistantMessage, AudioContent, AudioMimeType, BodyMessage, ContentPart, DocumentContent,
    ImageContent, LlmStep, MediaSource, ReasoningEffort, ResponseFormat, ToolCall, ToolCallMessage,
    ToolChoice, ToolMessage, UserMessage,
};
use crate::text::parse::{extract_preamble, remove_thought};
use crate::text::render::render_bytes;
use crate::tools::FunctionSpec;
use crate::utils::NoneIfEmpty;

impl From<ToolCall> for ChatCompletionMessageToolCalls {
    fn from(call: ToolCall) -> Self {
        ChatCompletionMessageToolCalls::Function(ChatCompletionMessageToolCall::from(call))
    }
}
impl From<ToolCall> for ChatCompletionMessageToolCall {
    fn from(call: ToolCall) -> Self {
        Self {
            id: call.id,
            function: FunctionCall {
                name: call.name,
                arguments: serde_json::to_string(&call.arguments)
                    .unwrap_or_else(|_| call.arguments.to_string()),
            },
        }
    }
}
impl From<ToolCall> for FunctionCall {
    fn from(call: ToolCall) -> Self {
        Self {
            name: call.name,
            arguments: serde_json::to_string(&call.arguments)
                .unwrap_or_else(|_| call.arguments.to_string()),
        }
    }
}

impl From<ChatCompletionMessageToolCalls> for ToolCall {
    fn from(call: ChatCompletionMessageToolCalls) -> Self {
        match call {
            ChatCompletionMessageToolCalls::Custom(call) => Self::from(call),
            ChatCompletionMessageToolCalls::Function(call) => Self::from(call),
        }
    }
}
impl From<ChatCompletionMessageToolCall> for ToolCall {
    fn from(call: ChatCompletionMessageToolCall) -> Self {
        let id = Some(call.id);
        let name = call.function.name;
        let arguments = serde_json::from_str(&call.function.arguments).ok();
        Self::new(id, name, arguments)
    }
}
impl From<ChatCompletionMessageCustomToolCall> for ToolCall {
    fn from(call: ChatCompletionMessageCustomToolCall) -> Self {
        let id = Some(call.id);
        let name = call.custom_tool.name;
        let arguments = serde_json::from_str(&call.custom_tool.input).ok();
        Self::new(id, name, arguments)
    }
}
impl TryFrom<FunctionCall> for ToolCall {
    type Error = serde_json::Error;

    fn try_from(call: FunctionCall) -> Result<Self, Self::Error> {
        let name = call.name;
        let arguments = serde_json::from_str(&call.arguments).ok();
        Ok(Self::new(None, name, arguments))
    }
}

impl TryFrom<BodyMessage> for ChatCompletionRequestMessage {
    type Error = UnsupportedContent;

    fn try_from(message: BodyMessage) -> Result<Self, Self::Error> {
        match message {
            BodyMessage::User(message) => ChatCompletionRequestMessage::try_from(message),
            BodyMessage::Assistant(message) => ChatCompletionRequestMessage::try_from(message),
            BodyMessage::ToolCall(message) => ChatCompletionRequestMessage::try_from(message),
            BodyMessage::Tool(message) => ChatCompletionRequestMessage::try_from(message),
        }
    }
}
impl TryFrom<UserMessage> for ChatCompletionRequestMessage {
    type Error = UnsupportedContent;

    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        Ok(ChatCompletionRequestMessage::from(
            ChatCompletionRequestUserMessage::try_from(message)?,
        ))
    }
}
impl TryFrom<UserMessage> for ChatCompletionRequestUserMessage {
    type Error = UnsupportedContent;

    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        if matches!(message.content.as_slice(), [ContentPart::Text(_)]) {
            let Some(ContentPart::Text(text)) = message.content.into_iter().next() else {
                unreachable!("Text content checked");
            };
            return Ok(Self {
                name: None,
                content: ChatCompletionRequestUserMessageContent::Text(text),
            });
        };

        let parts = message
            .content
            .into_iter()
            .map(ChatCompletionRequestUserMessageContentPart::try_from)
            .collect::<Result<Vec<_>, Self::Error>>()?;
        Ok(Self {
            name: None,
            content: ChatCompletionRequestUserMessageContent::Array(parts),
        })
    }
}
impl TryFrom<AssistantMessage> for ChatCompletionRequestMessage {
    type Error = UnsupportedContent;

    fn try_from(message: AssistantMessage) -> Result<Self, Self::Error> {
        let message = ChatCompletionRequestAssistantMessage::try_from(message)?;
        Ok(ChatCompletionRequestMessage::from(message))
    }
}
impl TryFrom<AssistantMessage> for ChatCompletionRequestAssistantMessage {
    type Error = UnsupportedContent;

    fn try_from(message: AssistantMessage) -> Result<Self, Self::Error> {
        let mut texts = collect_texts(message.content, "assistant output")?;

        if let Some(thought) = message.thought {
            if let Some(first) = texts.first_mut() {
                *first = format!("{thought}\n</think>\n{first}");
            } else {
                texts.push(format!("{thought}\n</think>"));
            }
        }

        if texts.len() == 1 {
            let Some(text) = texts.into_iter().next() else {
                unreachable!("Length 1 checked");
            };
            // Use deprecated field `function_call` for construction.
            #[allow(deprecated)]
            return Ok(ChatCompletionRequestAssistantMessage {
                name: None,
                content: Some(ChatCompletionRequestAssistantMessageContent::Text(text)),
                refusal: None,
                audio: None,
                tool_calls: None,
                function_call: None,
            });
        }

        let parts = texts
            .into_iter()
            .map(|text| {
                ChatCompletionRequestAssistantMessageContentPart::Text(
                    ChatCompletionRequestMessageContentPartText {
                        text,
                        prompt_cache_breakpoint: None,
                    },
                )
            })
            .collect::<Vec<_>>();
        // Use deprecated field `function_call` for construction.
        #[allow(deprecated)]
        Ok(ChatCompletionRequestAssistantMessage {
            name: None,
            content: Some(ChatCompletionRequestAssistantMessageContent::Array(parts)),
            refusal: None,
            audio: None,
            tool_calls: None,
            function_call: None,
        })
    }
}
impl TryFrom<ToolCallMessage> for ChatCompletionRequestMessage {
    type Error = UnsupportedContent;
    fn try_from(message: ToolCallMessage) -> Result<Self, Self::Error> {
        let message = ChatCompletionRequestAssistantMessage::try_from(message)?;
        Ok(ChatCompletionRequestMessage::from(message))
    }
}
impl TryFrom<ToolCallMessage> for ChatCompletionRequestAssistantMessage {
    type Error = UnsupportedContent;

    fn try_from(message: ToolCallMessage) -> Result<Self, Self::Error> {
        let (message, tool_calls) = message.split();

        let mut base = ChatCompletionRequestAssistantMessage::try_from(message)?;
        base.tool_calls = tool_calls
            .into_iter()
            .map(ChatCompletionMessageToolCalls::from)
            .collect::<Vec<_>>()
            .none_if_empty();
        Ok(base)
    }
}
impl TryFrom<ToolMessage> for ChatCompletionRequestMessage {
    type Error = UnsupportedContent;

    fn try_from(message: ToolMessage) -> Result<Self, Self::Error> {
        let message = ChatCompletionRequestToolMessage::try_from(message)?;
        Ok(ChatCompletionRequestMessage::from(message))
    }
}
impl TryFrom<ToolMessage> for ChatCompletionRequestToolMessage {
    type Error = UnsupportedContent;

    fn try_from(message: ToolMessage) -> Result<Self, Self::Error> {
        if let Some(text) = message.output.as_text() {
            return Ok(ChatCompletionRequestToolMessage {
                tool_call_id: message.call_id,
                content: ChatCompletionRequestToolMessageContent::Text(text.to_owned()),
            });
        }

        let texts = collect_texts(message.output.content, "tool output")?;
        let parts = texts
            .into_iter()
            .map(|text| {
                ChatCompletionRequestToolMessageContentPart::Text(
                    ChatCompletionRequestMessageContentPartText {
                        text,
                        prompt_cache_breakpoint: None,
                    },
                )
            })
            .collect::<Vec<_>>();
        Ok(ChatCompletionRequestToolMessage {
            tool_call_id: message.call_id,
            content: ChatCompletionRequestToolMessageContent::Array(parts),
        })
    }
}

impl TryFrom<ContentPart> for ChatCompletionRequestUserMessageContentPart {
    type Error = UnsupportedContent;

    fn try_from(part: ContentPart) -> Result<Self, Self::Error> {
        match part {
            ContentPart::Text(text) => Ok(ChatCompletionRequestUserMessageContentPart::Text(
                ChatCompletionRequestMessageContentPartText {
                    text,
                    prompt_cache_breakpoint: None,
                },
            )),
            ContentPart::Image(image) => Ok(ChatCompletionRequestUserMessageContentPart::ImageUrl(
                ChatCompletionRequestMessageContentPartImage::try_from(image)?,
            )),
            ContentPart::Audio(audio) => {
                Ok(ChatCompletionRequestUserMessageContentPart::InputAudio(
                    ChatCompletionRequestMessageContentPartAudio::try_from(audio)?,
                ))
            }
            ContentPart::Video(_) => Err(UnsupportedContent::new(
                "OpenAI Chat Completions",
                "video content",
            )),
            ContentPart::Document(document) => {
                Ok(ChatCompletionRequestUserMessageContentPart::File(
                    ChatCompletionRequestMessageContentPartFile::try_from(document)?,
                ))
            }
        }
    }
}
impl TryFrom<ImageContent> for ChatCompletionRequestMessageContentPartImage {
    type Error = UnsupportedContent;

    fn try_from(image: ImageContent) -> Result<Self, Self::Error> {
        let image_url = ImageUrl::try_from(image)?;
        Ok(Self {
            image_url,
            prompt_cache_breakpoint: None,
        })
    }
}
impl TryFrom<ImageContent> for ImageUrl {
    type Error = UnsupportedContent;

    /// Chat Completions carries an image as a URL or an inline data URL.
    fn try_from(image: ImageContent) -> Result<Self, Self::Error> {
        let url = match image.source {
            MediaSource::Url(url) => url,
            MediaSource::Bytes(raw) => render_bytes(raw, image.mime),
            MediaSource::FileId(_) => {
                return Err(UnsupportedContent::new(
                    "OpenAI Chat Completions",
                    "image file ids",
                ));
            }
        };
        Ok(ImageUrl {
            url,
            detail: Some(ImageDetail::Auto),
        })
    }
}
impl TryFrom<AudioContent> for ChatCompletionRequestMessageContentPartAudio {
    type Error = UnsupportedContent;

    fn try_from(audio: AudioContent) -> Result<Self, Self::Error> {
        Ok(Self {
            input_audio: InputAudio::try_from(audio)?,
            prompt_cache_breakpoint: None,
        })
    }
}
impl TryFrom<AudioContent> for InputAudio {
    type Error = UnsupportedContent;

    fn try_from(audio: AudioContent) -> Result<Self, Self::Error> {
        let data = match audio.source {
            MediaSource::Url(_) => {
                return Err(UnsupportedContent::new(
                    "OpenAI Chat Completions",
                    "audio URLs",
                ));
            }
            MediaSource::Bytes(raw) => BASE64_STANDARD.encode(raw),
            MediaSource::FileId(_) => {
                return Err(UnsupportedContent::new(
                    "OpenAI Chat Completions",
                    "audio file ids",
                ));
            }
        };
        let format = InputAudioFormat::try_from(audio.mime)?;
        Ok(InputAudio { data, format })
    }
}
impl TryFrom<AudioMimeType> for InputAudioFormat {
    type Error = UnsupportedContent;

    fn try_from(mime: AudioMimeType) -> Result<Self, Self::Error> {
        match mime {
            AudioMimeType::Mp3 => Ok(InputAudioFormat::Mp3),
            AudioMimeType::Wav => Ok(InputAudioFormat::Wav),
            other => Err(UnsupportedContent::new(
                "OpenAI Chat Completions",
                other.to_string(),
            )),
        }
    }
}
impl TryFrom<DocumentContent> for ChatCompletionRequestMessageContentPartFile {
    type Error = UnsupportedContent;

    fn try_from(document: DocumentContent) -> Result<Self, Self::Error> {
        let file = FileObject::try_from(document)?;
        Ok(Self {
            file,
            prompt_cache_breakpoint: None,
        })
    }
}
impl TryFrom<DocumentContent> for FileObject {
    type Error = UnsupportedContent;

    fn try_from(document: DocumentContent) -> Result<Self, Self::Error> {
        // Using `from_value` because `FileObject` cannot be constructed as of `async-openai`
        // version 0.41.1
        match document.source {
            MediaSource::Url(_) => Err(UnsupportedContent::new(
                "OpenAI Chat Completions",
                "document URLs",
            )),
            MediaSource::Bytes(raw) => Ok(serde_json::from_value::<FileObject>(json!({
                "file_data": render_bytes(raw, document.mime),
                "filename": document.name
            }))
            .map_err(|e| {
                UnsupportedContent::new("OpenAI Chat Completions", format!("document bytes: {e}"))
            })?),
            MediaSource::FileId(file_id) => Ok(serde_json::from_value::<FileObject>(json!({
                "file_id": file_id,
                "filename": document.name
            }))
            .map_err(|e| {
                UnsupportedContent::new("OpenAI Chat Completions", format!("document file id: {e}"))
            })?),
        }
    }
}

impl TryFrom<ChatCompletionResponseMessage> for LlmStep {
    type Error = LlmError;

    fn try_from(message: ChatCompletionResponseMessage) -> Result<Self, Self::Error> {
        let tool_calls = match message.tool_calls {
            Some(tool_calls) => tool_calls
                .into_iter()
                .map(ToolCall::from)
                .collect::<Vec<_>>(),
            // Handle deprecated `function_call` for completeness.
            // `function_call` is only considered when `tool_calls` is absent.
            #[allow(deprecated)]
            None => match message.function_call {
                Some(function_call) => {
                    let tool_call =
                        ToolCall::try_from(function_call).map_err(LlmError::ResponseDeserialize)?;
                    vec![tool_call]
                }
                None => vec![],
            },
        };

        if !tool_calls.is_empty() {
            let parts = match message.content {
                Some(text) => vec![ContentPart::Text(text)],
                None => vec![],
            };
            let tool_calls = tool_calls.into_iter().collect::<Vec<_>>();
            return Ok(LlmStep::ToolCall(ToolCallMessage::from_parts(
                None, parts, tool_calls,
            )));
        }
        if let Some(text) = message.content {
            let content = remove_thought(&text).to_owned();
            let thought = extract_preamble(&text, &content)
                .none_if_empty()
                .map(|s| s.to_owned());
            return Ok(LlmStep::new_text(thought, content));
        }
        if let Some(refusal) = message.refusal {
            return Err(LlmError::Refused(refusal));
        }

        Err(LlmError::other(
            "cannot convert LLM generation result to LlmOutputContent",
        ))
    }
}

impl From<FunctionSpec> for ChatCompletionTools {
    fn from(tool: FunctionSpec) -> Self {
        ChatCompletionTools::Function(ChatCompletionTool::from(tool))
    }
}
impl From<FunctionSpec> for ChatCompletionTool {
    fn from(tool: FunctionSpec) -> Self {
        ChatCompletionTool {
            function: FunctionObject::from(tool),
        }
    }
}
impl From<FunctionSpec> for FunctionObject {
    fn from(tool: FunctionSpec) -> Self {
        FunctionObject {
            name: tool.name,
            description: tool.description,
            parameters: Some(tool.parameters.to_value()),
            strict: Some(tool.strict),
        }
    }
}

impl From<ReasoningEffort> for OpenAiReasoningEffort {
    fn from(effort: ReasoningEffort) -> Self {
        match effort {
            ReasoningEffort::None => OpenAiReasoningEffort::None,
            ReasoningEffort::Minimal => OpenAiReasoningEffort::Minimal,
            ReasoningEffort::Low => OpenAiReasoningEffort::Low,
            ReasoningEffort::Medium => OpenAiReasoningEffort::Medium,
            ReasoningEffort::High => OpenAiReasoningEffort::High,
            ReasoningEffort::Xhigh => OpenAiReasoningEffort::Xhigh,
            ReasoningEffort::Max => OpenAiReasoningEffort::Xhigh,
        }
    }
}

impl From<ResponseFormat> for ChatCompletionResponseFormat {
    fn from(format: ResponseFormat) -> Self {
        match format {
            ResponseFormat::Text => ChatCompletionResponseFormat::Text,
            ResponseFormat::JsonObject => ChatCompletionResponseFormat::JsonObject,
            ResponseFormat::JsonSchema(s) => ChatCompletionResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    name: s.name,
                    description: s.description,
                    schema: s.schema.to_value(),
                    strict: s.strict,
                },
            },
        }
    }
}

impl From<ToolChoice> for ChatCompletionToolChoiceOption {
    fn from(choice: ToolChoice) -> Self {
        match choice {
            ToolChoice::None => ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::None),
            ToolChoice::Auto => ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Auto),
            ToolChoice::Required => {
                ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Required)
            }
            ToolChoice::Function(name) => {
                ChatCompletionToolChoiceOption::Function(ChatCompletionNamedToolChoice {
                    function: FunctionName { name },
                })
            }
            ToolChoice::Mcp(name, server_label) => {
                log::warn!("MCP function {name} ({server_label}) is not supported for the api.");
                ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Auto)
            }
            ToolChoice::ProviderSpecific(id) => {
                log::warn!("Provider specific tool {id} is not supported for the api.");
                ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Auto)
            }
        }
    }
}
