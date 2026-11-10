use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use secrecy::ExposeSecret;
use talin_provider::xai_responses::types::{
    FunctionTool, FunctionToolCall, FunctionToolCallOutput, InputFileItem, InputImageItem,
    InputMessage, InputTextItem, McpTool as XaiMcpTool, ModelInputContent, ModelInputContentItem,
    ModelInputPart, ModelOutput, ModelResponseFormat, ModelTool, ModelToolChoice,
    ModelToolChoiceMode, ModelToolChoiceTool, OutputMessageContent, ReasoningConfiguration,
    ReasoningEffort as XaiReasoningEffort, ReasoningText, Role, SummaryText,
};

use crate::llm::UnsupportedContent;
use crate::mcp::McpServerHttp;
use crate::schemas::{
    AssistantMessage, BodyMessage, ContentPart, DocumentContent, ImageContent, LlmStep,
    MediaSource, ReasoningEffort, ResponseFormat, ToolCall, ToolCallMessage, ToolChoice,
    ToolMessage, UserMessage,
};
use crate::text::render::render_bytes;
use crate::tools::{FunctionSpec, ToolSpec};

impl From<Vec<ModelOutput>> for LlmStep {
    fn from(outputs: Vec<ModelOutput>) -> Self {
        let mut texts = Vec::new();
        let mut tool_calls = Vec::new();
        let mut thoughts = Vec::new();

        for output in outputs {
            match output {
                ModelOutput::Message(msg) => {
                    for content in msg.content {
                        match content {
                            OutputMessageContent::OutputText(t) => texts.push(t.text),
                            OutputMessageContent::Refusal(r) => {
                                log::warn!("Refusal: {}", r.refusal);
                                thoughts.push(format!("Refusal: {}", r.refusal));
                            }
                            _ => { /* Ignore other content types */ }
                        }
                    }
                }
                ModelOutput::FunctionCall(tool) => {
                    let id = Some(tool.call_id);
                    let name = tool.name;
                    let arguments = serde_json::from_str(&tool.arguments).ok();
                    tool_calls.push(ToolCall::new(id, name, arguments));
                }
                ModelOutput::Reasoning(reasoning) => {
                    if let Some(contents) = reasoning.content {
                        thoughts.extend(contents.into_iter().filter_map(|content| match content {
                            ReasoningText::ReasoningText(text) => Some(text.text),
                            _ => None,
                        }));
                    } else {
                        thoughts.extend(reasoning.summary.into_iter().filter_map(|summary| {
                            match summary {
                                SummaryText::SummaryText(text) => Some(text.text),
                                _ => None,
                            }
                        }));
                    }
                }
                _ => { /* Ignore other output types */ }
            }
        }
        LlmStep::from_parts(thoughts, texts, tool_calls)
    }
}

impl TryFrom<BodyMessage> for Vec<ModelInputPart> {
    type Error = UnsupportedContent;

    fn try_from(message: BodyMessage) -> Result<Self, Self::Error> {
        Ok(match message {
            BodyMessage::User(message) => vec![ModelInputPart::try_from(message)?],
            BodyMessage::Assistant(message) => vec![ModelInputPart::try_from(message)?],
            BodyMessage::ToolCall(message) => Self::try_from(message)?,
            BodyMessage::Tool(message) => vec![ModelInputPart::try_from(message)?],
        })
    }
}

impl TryFrom<UserMessage> for ModelInputPart {
    type Error = UnsupportedContent;

    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        let content = map_content(message.content)?;
        Ok(ModelInputPart::Message(InputMessage::new(
            content,
            Role::User,
        )))
    }
}
impl TryFrom<AssistantMessage> for ModelInputPart {
    type Error = UnsupportedContent;

    fn try_from(message: AssistantMessage) -> Result<Self, Self::Error> {
        let content = map_content(message.content)?;
        Ok(ModelInputPart::Message(InputMessage::new(
            content,
            Role::Assistant,
        )))
    }
}
impl TryFrom<ToolCallMessage> for Vec<ModelInputPart> {
    type Error = UnsupportedContent;

    fn try_from(message: ToolCallMessage) -> Result<Self, Self::Error> {
        let (message, tool_calls) = message.split();
        let item = ModelInputPart::try_from(message)?;
        let tool_calls = tool_calls.into_iter().map(ModelInputPart::from);
        Ok([item].into_iter().chain(tool_calls).collect())
    }
}
impl From<ToolCall> for ModelInputPart {
    fn from(call: ToolCall) -> Self {
        ModelInputPart::ModelOutput(ModelOutput::FunctionCall(FunctionToolCall::from(call)))
    }
}
impl TryFrom<ToolMessage> for ModelInputPart {
    type Error = UnsupportedContent;

    fn try_from(message: ToolMessage) -> Result<Self, Self::Error> {
        let output = map_content(message.output.content)?;
        Ok(ModelInputPart::FunctionCallOutput(
            FunctionToolCallOutput::with_content(message.call_id, output),
        ))
    }
}

fn map_content(content: Vec<ContentPart>) -> Result<ModelInputContent, UnsupportedContent> {
    if matches!(content.as_slice(), [ContentPart::Text(_)]) {
        let Some(ContentPart::Text(text)) = content.into_iter().next() else {
            unreachable!("Text content checked");
        };
        return Ok(ModelInputContent::Text(text));
    }

    let parts = content
        .into_iter()
        .map(ModelInputContentItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ModelInputContent::Parts(parts))
}

impl TryFrom<ContentPart> for ModelInputContentItem {
    type Error = UnsupportedContent;

    fn try_from(part: ContentPart) -> Result<Self, Self::Error> {
        match part {
            ContentPart::Text(text) => {
                Ok(ModelInputContentItem::InputText(InputTextItem::new(text)))
            }
            ContentPart::Image(image) => ModelInputContentItem::try_from(image),
            ContentPart::Audio(_) => Err(UnsupportedContent::new("xAI Responses", "audio content")),
            ContentPart::Video(_) => Err(UnsupportedContent::new("xAI Responses", "video content")),
            ContentPart::Document(document) => ModelInputContentItem::try_from(document),
        }
    }
}
impl TryFrom<ImageContent> for ModelInputContentItem {
    type Error = UnsupportedContent;

    fn try_from(image: ImageContent) -> Result<Self, Self::Error> {
        Ok(ModelInputContentItem::InputImage(InputImageItem::try_from(
            image,
        )?))
    }
}
impl TryFrom<ImageContent> for InputImageItem {
    type Error = UnsupportedContent;

    /// xAI carries an image as a URL or an inline data URL. Its `file_id` field is documented as
    /// present only for OpenAI compatibility and is not a usable handle.
    fn try_from(image: ImageContent) -> Result<Self, Self::Error> {
        let url = match image.source {
            MediaSource::Url(url) => url,
            MediaSource::Bytes(raw) => render_bytes(raw, image.mime),
            MediaSource::FileId(_) => {
                return Err(UnsupportedContent::new("xAI", "image file ids"));
            }
        };
        Ok(InputImageItem::new(url))
    }
}
impl TryFrom<DocumentContent> for ModelInputContentItem {
    type Error = UnsupportedContent;

    fn try_from(document: DocumentContent) -> Result<Self, Self::Error> {
        Ok(ModelInputContentItem::InputFile(InputFileItem::from(
            document,
        )))
    }
}
impl From<DocumentContent> for InputFileItem {
    fn from(document: DocumentContent) -> Self {
        let mut item = match document.source {
            MediaSource::Url(url) => InputFileItem::new().with_file_url(url),
            MediaSource::Bytes(raw) => {
                let bytes = BASE64_STANDARD.encode(raw);
                InputFileItem::new().with_file_data(bytes, document.mime.to_string())
            }
            MediaSource::FileId(file_id) => InputFileItem::new().with_file_id(file_id),
        };
        if let Some(name) = document.name {
            item = item.with_filename(name)
        }
        item
    }
}

impl From<ToolCall> for FunctionToolCall {
    fn from(call: ToolCall) -> Self {
        FunctionToolCall::new(call.id, call.arguments.to_string(), call.name)
    }
}

impl From<ToolSpec> for ModelTool {
    fn from(tool: ToolSpec) -> Self {
        match tool {
            ToolSpec::Function(func) => ModelTool::Function(FunctionTool::from(func)),
        }
    }
}
impl From<McpServerHttp> for ModelTool {
    fn from(server: McpServerHttp) -> Self {
        ModelTool::Mcp(XaiMcpTool::from(server))
    }
}
impl From<FunctionSpec> for FunctionTool {
    fn from(tool: FunctionSpec) -> Self {
        let mut function = FunctionTool::new(tool.name, tool.parameters.to_value());
        function.description = tool.description;
        function
    }
}
impl From<McpServerHttp> for XaiMcpTool {
    fn from(tool: McpServerHttp) -> Self {
        let mut mcp = XaiMcpTool::new(tool.name, tool.transport.url.expose_secret().to_owned());
        mcp.server_description = tool.description;
        mcp.allowed_tools = tool.allowed_tools;
        mcp.authorization = tool
            .transport
            .authorization
            .map(|auth| auth.expose_secret().to_owned());
        mcp
    }
}

impl From<ResponseFormat> for ModelResponseFormat {
    fn from(format: ResponseFormat) -> Self {
        match format {
            ResponseFormat::Text => ModelResponseFormat::Text,
            ResponseFormat::JsonObject => ModelResponseFormat::JsonObject,
            ResponseFormat::JsonSchema(schema) => ModelResponseFormat::JsonSchema {
                schema: schema.schema,
                name: Some(schema.name),
                description: schema.description,
                strict: schema.strict,
            },
        }
    }
}

impl From<ReasoningEffort> for Option<ReasoningConfiguration> {
    fn from(effort: ReasoningEffort) -> Self {
        let effort = match effort {
            ReasoningEffort::None => XaiReasoningEffort::None,
            ReasoningEffort::Minimal => XaiReasoningEffort::Low,
            ReasoningEffort::Low => XaiReasoningEffort::Low,
            ReasoningEffort::Medium => XaiReasoningEffort::Medium,
            ReasoningEffort::High => XaiReasoningEffort::High,
            ReasoningEffort::Xhigh => XaiReasoningEffort::High,
            ReasoningEffort::Max => XaiReasoningEffort::High,
        };

        Some(ReasoningConfiguration::new().with_effort(effort))
    }
}

impl From<ToolChoice> for ModelToolChoice {
    fn from(choice: ToolChoice) -> Self {
        match choice {
            ToolChoice::None => ModelToolChoice::Mode(ModelToolChoiceMode::None),
            ToolChoice::Auto => ModelToolChoice::Mode(ModelToolChoiceMode::Auto),
            ToolChoice::Required => ModelToolChoice::Mode(ModelToolChoiceMode::Required),
            ToolChoice::Function(name) => ModelToolChoice::Tool(ModelToolChoiceTool::new(name)),
            _ => ModelToolChoice::Mode(ModelToolChoiceMode::Auto),
        }
    }
}
