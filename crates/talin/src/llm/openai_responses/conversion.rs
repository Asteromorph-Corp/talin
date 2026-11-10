//! This module consists of conversion logic between the schema types of the crate and the types in
//! `talin_provider::openai_responses::types`.

use secrecy::ExposeSecret;
use talin_provider::openai_chat::types::ImageDetail;
use talin_provider::openai_responses::types::{
    EasyInputContent, EasyInputMessage, FunctionCallOutput, FunctionCallOutputItemParam,
    FunctionTool, FunctionToolCall, InputContent, InputFileContent, InputImageContent, InputItem,
    InputMessage, InputRole, InputTextContent, Item, MCPTool, MCPToolAllowedTools,
    MCPToolApprovalSetting, MCPToolRequireApproval, MessageItem, MessageType, OutputItem,
    OutputMessageContent, Reasoning, ReasoningEffort as OpenAiReasoningEffort,
    ReasoningItemContent, ReasoningSummary, ResponseFormatJsonSchema, ResponseTextParam, Role,
    SummaryPart, TextResponseFormatConfiguration, Tool, ToolChoiceFunction, ToolChoiceMCP,
    ToolChoiceOptions, ToolChoiceParam, ToolChoiceTypes,
};

use crate::llm::{LlmError, UnsupportedContent};
use crate::mcp::McpServerHttp;
use crate::schemas::{
    AssistantMessage, BodyMessage, ContentPart, DocumentContent, ImageContent, LlmStep,
    MediaSource, ReasoningEffort, ResponseFormat, SystemMessage, ToolCall, ToolChoice, ToolMessage,
    UserMessage,
};
use crate::text::render::render_bytes;
use crate::tools::{FunctionSpec, ToolSpec};

impl From<ToolCall> for FunctionToolCall {
    fn from(call: ToolCall) -> Self {
        Self {
            id: None,
            call_id: call.id,
            name: call.name,
            arguments: serde_json::to_string(&call.arguments)
                .unwrap_or_else(|_| call.arguments.to_string()),
            namespace: None,
            status: None,
            caller: None,
            r#async: None,
        }
    }
}
impl From<ToolCall> for InputItem {
    fn from(call: ToolCall) -> Self {
        InputItem::Item(Item::FunctionCall(FunctionToolCall::from(call)))
    }
}

impl TryFrom<FunctionToolCall> for ToolCall {
    type Error = serde_json::Error;

    fn try_from(call: FunctionToolCall) -> Result<Self, Self::Error> {
        let id = Some(call.call_id);
        let name = call.name;
        let arguments = serde_json::from_str(&call.arguments).ok();
        Ok(ToolCall::new(id, name, arguments))
    }
}

impl TryFrom<BodyMessage> for Vec<InputItem> {
    type Error = UnsupportedContent;

    fn try_from(message: BodyMessage) -> Result<Self, Self::Error> {
        let items = match message {
            BodyMessage::User(message) => vec![InputItem::try_from(message)?],
            BodyMessage::Assistant(message) => vec![InputItem::try_from(message)?],
            BodyMessage::ToolCall(message) => {
                let (message, calls) = message.split();
                let message = InputItem::try_from(message)?;
                let calls = calls.into_iter().map(InputItem::from);
                [message].into_iter().chain(calls).collect()
            }
            BodyMessage::Tool(message) => vec![InputItem::try_from(message)?],
        };
        Ok(items)
    }
}
impl From<SystemMessage> for InputMessage {
    fn from(message: SystemMessage) -> Self {
        let content = vec![InputContent::InputText(InputTextContent {
            text: message.content,
            prompt_cache_breakpoint: None,
        })];
        InputMessage {
            role: InputRole::System,
            content,
            status: None,
        }
    }
}
impl From<SystemMessage> for InputItem {
    fn from(message: SystemMessage) -> Self {
        InputItem::Item(Item::Message(MessageItem::Input(InputMessage::from(
            message,
        ))))
    }
}
impl TryFrom<UserMessage> for InputItem {
    type Error = UnsupportedContent;

    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        let message = InputMessage::try_from(message)?;
        Ok(InputItem::Item(Item::Message(MessageItem::Input(message))))
    }
}
impl TryFrom<UserMessage> for InputMessage {
    type Error = UnsupportedContent;

    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        let content = message
            .content
            .into_iter()
            .map(InputContent::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(InputMessage {
            role: InputRole::User,
            content,
            status: None,
        })
    }
}
impl TryFrom<AssistantMessage> for InputItem {
    type Error = UnsupportedContent;

    fn try_from(message: AssistantMessage) -> Result<Self, Self::Error> {
        let content = if matches!(message.content.as_slice(), [ContentPart::Text(_)]) {
            let Some(ContentPart::Text(text)) = message.content.into_iter().next() else {
                unreachable!("Text content checked");
            };
            EasyInputContent::Text(text)
        } else {
            let content = message
                .content
                .into_iter()
                .map(InputContent::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            EasyInputContent::ContentList(content)
        };

        Ok(InputItem::EasyMessage(EasyInputMessage {
            r#type: MessageType::Message,
            role: Role::Assistant,
            content,
            phase: None,
        }))
    }
}
impl TryFrom<ToolMessage> for InputItem {
    type Error = UnsupportedContent;

    fn try_from(message: ToolMessage) -> Result<Self, Self::Error> {
        let output = FunctionCallOutputItemParam::try_from(message)?;
        Ok(InputItem::Item(Item::FunctionCallOutput(output)))
    }
}
impl TryFrom<ToolMessage> for FunctionCallOutputItemParam {
    type Error = UnsupportedContent;

    fn try_from(message: ToolMessage) -> Result<Self, Self::Error> {
        let output = if let Some(text) = message.output.as_text() {
            FunctionCallOutput::Text(text.to_owned())
        } else {
            let items = message
                .output
                .content
                .into_iter()
                .map(InputContent::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            FunctionCallOutput::Content(items)
        };

        Ok(FunctionCallOutputItemParam {
            call_id: Some(message.call_id),
            output,
            id: None,
            status: None,
            name: None,
            namespace: None,
            caller: None,
        })
    }
}

impl TryFrom<ContentPart> for InputContent {
    type Error = UnsupportedContent;

    fn try_from(part: ContentPart) -> Result<Self, Self::Error> {
        match part {
            ContentPart::Text(text) => Ok(InputContent::InputText(InputTextContent {
                text,
                prompt_cache_breakpoint: None,
            })),
            ContentPart::Image(image) => {
                Ok(InputContent::InputImage(InputImageContent::from(image)))
            }
            ContentPart::Audio(_) => Err(UnsupportedContent::new("OpenAI Responses", "audio")),
            ContentPart::Video(_) => Err(UnsupportedContent::new("OpenAI Responses", "video")),
            ContentPart::Document(document) => {
                Ok(InputContent::InputFile(InputFileContent::from(document)))
            }
        }
    }
}
impl From<ImageContent> for InputImageContent {
    fn from(image: ImageContent) -> Self {
        match image.source {
            MediaSource::Url(url) => InputImageContent {
                image_url: Some(url),
                file_id: None,
                detail: ImageDetail::Auto,
                prompt_cache_breakpoint: None,
            },
            MediaSource::Bytes(raw) => InputImageContent {
                image_url: Some(render_bytes(raw, image.mime)),
                file_id: None,
                detail: ImageDetail::Auto,
                prompt_cache_breakpoint: None,
            },
            MediaSource::FileId(file_id) => InputImageContent {
                image_url: None,
                file_id: Some(file_id),
                detail: ImageDetail::Auto,
                prompt_cache_breakpoint: None,
            },
        }
    }
}
impl From<DocumentContent> for InputFileContent {
    fn from(document: DocumentContent) -> Self {
        let (file_data, file_url, file_id) = match document.source {
            MediaSource::Bytes(raw) => (Some(render_bytes(raw, document.mime)), None, None),
            MediaSource::Url(url) => (None, Some(url), None),
            MediaSource::FileId(file_id) => (None, None, Some(file_id)),
        };
        InputFileContent {
            file_data,
            file_id,
            file_url,
            filename: document.name,
            prompt_cache_breakpoint: None,
            detail: None,
        }
    }
}

impl TryFrom<Vec<OutputItem>> for LlmStep {
    type Error = LlmError;

    fn try_from(items: Vec<OutputItem>) -> Result<Self, Self::Error> {
        let mut thoughts = Vec::new();
        let mut texts = Vec::new();
        let mut tool_calls = Vec::new();

        for item in items {
            match item {
                OutputItem::Reasoning(reasoning) => {
                    let content = reasoning.content.unwrap_or_default();
                    if content.is_empty() {
                        thoughts.extend(reasoning.summary.into_iter().map(|part| match part {
                            SummaryPart::SummaryText(text) => text.text,
                        }));
                    } else {
                        thoughts.extend(content.into_iter().map(|c| match c {
                            ReasoningItemContent::ReasoningText(text) => text.text,
                        }));
                    }
                }
                OutputItem::Message(message) => {
                    for content in message.content {
                        match content {
                            OutputMessageContent::OutputText(t) => {
                                texts.push(t.text);
                            }
                            OutputMessageContent::Refusal(r) => {
                                return Err(LlmError::Refused(r.refusal));
                            }
                        }
                    }
                }
                OutputItem::FunctionCall(call) => {
                    let calls = ToolCall::try_from(call).map_err(LlmError::ResponseDeserialize)?;
                    tool_calls.push(calls);
                }
                OutputItem::McpApprovalRequest(_) => return Err(LlmError::InputRequired),
                _ => {}
            }
        }
        Ok(LlmStep::from_parts(thoughts, texts, tool_calls))
    }
}

impl From<ToolSpec> for Tool {
    fn from(tool: ToolSpec) -> Self {
        match tool {
            ToolSpec::Function(func) => Tool::Function(FunctionTool::from(func)),
        }
    }
}
impl From<McpServerHttp> for Tool {
    fn from(server: McpServerHttp) -> Self {
        Tool::Mcp(MCPTool::from(server))
    }
}
impl From<FunctionSpec> for FunctionTool {
    fn from(tool: FunctionSpec) -> Self {
        FunctionTool {
            name: tool.name,
            description: tool.description,
            parameters: Some(tool.parameters.to_value()),
            strict: Some(tool.strict),
            defer_loading: None,
            allowed_callers: None,
            r#async: None,
            output_schema: None,
        }
    }
}
impl From<McpServerHttp> for MCPTool {
    fn from(tool: McpServerHttp) -> Self {
        let allowed_tools = tool.allowed_tools.map(MCPToolAllowedTools::List);

        MCPTool {
            server_url: Some(tool.transport.url.expose_secret().to_owned()),
            server_label: tool.name,
            server_description: tool.description,
            allowed_tools,
            authorization: tool
                .transport
                .authorization
                .map(|auth| auth.expose_secret().to_owned()),
            defer_loading: None,
            connector_id: None,
            headers: None,
            require_approval: Some(MCPToolRequireApproval::ApprovalSetting(
                MCPToolApprovalSetting::Never,
            )),
            allowed_callers: None,
            tunnel_id: None,
        }
    }
}

impl From<ReasoningEffort> for Reasoning {
    fn from(effort: ReasoningEffort) -> Self {
        let effort = match effort {
            ReasoningEffort::None => OpenAiReasoningEffort::None,
            ReasoningEffort::Minimal => OpenAiReasoningEffort::Minimal,
            ReasoningEffort::Low => OpenAiReasoningEffort::Low,
            ReasoningEffort::Medium => OpenAiReasoningEffort::Medium,
            ReasoningEffort::High => OpenAiReasoningEffort::High,
            ReasoningEffort::Xhigh => OpenAiReasoningEffort::Xhigh,
            ReasoningEffort::Max => OpenAiReasoningEffort::Xhigh,
        };

        Reasoning {
            effort: Some(effort),
            summary: Some(ReasoningSummary::Auto),
            context: None,
            mode: None,
        }
    }
}

impl From<ResponseFormat> for ResponseTextParam {
    fn from(format: ResponseFormat) -> Self {
        let format = match format {
            ResponseFormat::Text => TextResponseFormatConfiguration::Text,
            ResponseFormat::JsonObject => TextResponseFormatConfiguration::JsonObject,
            ResponseFormat::JsonSchema(schema) => {
                TextResponseFormatConfiguration::JsonSchema(ResponseFormatJsonSchema {
                    name: schema.name,
                    description: schema.description,
                    schema: schema.schema.to_value(),
                    strict: schema.strict,
                })
            }
        };

        ResponseTextParam {
            format,
            verbosity: None,
        }
    }
}

impl From<ToolChoice> for ToolChoiceParam {
    fn from(value: ToolChoice) -> Self {
        match value {
            ToolChoice::None => ToolChoiceParam::Option(ToolChoiceOptions::None),
            ToolChoice::Auto => ToolChoiceParam::Option(ToolChoiceOptions::Auto),
            ToolChoice::Required => ToolChoiceParam::Option(ToolChoiceOptions::Required),
            ToolChoice::Function(name) => ToolChoiceParam::Function(ToolChoiceFunction { name }),
            ToolChoice::Mcp(name, server_label) => {
                ToolChoiceParam::Mcp(ToolChoiceMCP { name, server_label })
            }
            ToolChoice::FILE_SEARCH => ToolChoiceParam::BuiltIn(ToolChoiceTypes::FileSearch),
            ToolChoice::WEB_SEARCH_PREVIEW => {
                ToolChoiceParam::BuiltIn(ToolChoiceTypes::WebSearchPreview)
            }
            ToolChoice::COMPUTER_USE_PREVIEW => {
                ToolChoiceParam::BuiltIn(ToolChoiceTypes::ComputerUsePreview)
            }
            ToolChoice::CODE_INTERPRETER => {
                ToolChoiceParam::BuiltIn(ToolChoiceTypes::CodeInterpreter)
            }
            ToolChoice::IMAGE_GENERATION => {
                ToolChoiceParam::BuiltIn(ToolChoiceTypes::ImageGeneration)
            }
            ToolChoice::APPLY_PATCH => ToolChoiceParam::ApplyPatch,
            ToolChoice::SHELL => ToolChoiceParam::Shell,
            ToolChoice::ProviderSpecific(hosted) => {
                log::warn!("Hosted function {hosted} is not supported for the api.");
                ToolChoiceParam::Option(ToolChoiceOptions::Auto)
            }
        }
    }
}
