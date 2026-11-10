use std::collections::HashMap;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use secrecy::ExposeSecret;
use talin_provider::claude_messages::types::{
    Base64ImageSource, Base64ImageType, Base64PDFSource, Base64PDFType, ContentBlock,
    ContentBlockParam, DocumentBlockParam, DocumentBlockParamSource, FileDocumentSource,
    FileImageSource, ImageBlockParam, ImageSource, JSONOutputFormat, McpServer, McpServerUrl,
    McpToolConfig, McpToolset, MessageParam, OutputEffort, TextBlockParam, TextOrItems,
    ThinkingConfigAdaptive, ThinkingConfigParam, ThinkingDisplay, Tool,
    ToolChoice as ClaudeToolChoice, ToolResultBlockParam, ToolResultBlockParamContent, ToolUnion,
    ToolUseBlock, ToolUseBlockParam, URLImageSource, URLPDFSource,
};

use crate::llm::UnsupportedContent;
use crate::mcp::McpServerHttp;
use crate::schemas::{
    AssistantMessage, BodyMessage, ContentPart, DocumentContent, DocumentMimeType, ImageContent,
    ImageMimeType, LlmStep, MediaSource, ReasoningEffort, ResponseFormat, ToolCall,
    ToolCallMessage, ToolChoice, ToolMessage, UserMessage,
};
use crate::tools::FunctionSpec;

impl From<ToolCall> for ToolUseBlockParam {
    fn from(call: ToolCall) -> Self {
        ToolUseBlockParam::new(call.id, call.name, call.arguments)
    }
}

impl From<ToolUseBlock> for ToolCall {
    fn from(call: ToolUseBlock) -> Self {
        ToolCall {
            id: call.id,
            name: call.name,
            arguments: call.input,
        }
    }
}

impl TryFrom<BodyMessage> for MessageParam {
    type Error = UnsupportedContent;

    fn try_from(message: BodyMessage) -> Result<Self, Self::Error> {
        match message {
            BodyMessage::User(message) => MessageParam::try_from(message),
            BodyMessage::Assistant(message) => MessageParam::try_from(message),
            BodyMessage::ToolCall(message) => MessageParam::try_from(message),
            BodyMessage::Tool(message) => MessageParam::try_from(message),
        }
    }
}

impl TryFrom<UserMessage> for MessageParam {
    type Error = UnsupportedContent;

    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        let content = map_content(message.content)?;
        Ok(MessageParam::user(content))
    }
}
impl TryFrom<AssistantMessage> for MessageParam {
    type Error = UnsupportedContent;

    fn try_from(message: AssistantMessage) -> Result<Self, Self::Error> {
        let content = map_content(message.content)?;
        Ok(MessageParam::assistant(content))
    }
}
impl TryFrom<ToolCallMessage> for MessageParam {
    type Error = UnsupportedContent;

    fn try_from(message: ToolCallMessage) -> Result<Self, Self::Error> {
        let items = message
            .content
            .into_iter()
            .map(ContentBlockParam::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let tool_calls = message
            .tool_calls
            .into_iter()
            .map(|call| ContentBlockParam::ToolUse(ToolUseBlockParam::from(call)));
        let items = items.into_iter().chain(tool_calls).collect();
        Ok(MessageParam::assistant(TextOrItems::Items(items)))
    }
}
impl TryFrom<ToolMessage> for MessageParam {
    type Error = UnsupportedContent;

    fn try_from(message: ToolMessage) -> Result<Self, Self::Error> {
        let content = if let Some(text) = message.output.as_text() {
            TextOrItems::Text(text.to_owned())
        } else {
            let items = message
                .output
                .content
                .into_iter()
                .map(|part| match part {
                    ContentPart::Text(text) => {
                        Ok(ToolResultBlockParamContent::Text(TextBlockParam::new(text)))
                    }
                    ContentPart::Image(image) => Ok(ToolResultBlockParamContent::Image(
                        ImageBlockParam::try_from(image)?,
                    )),
                    ContentPart::Audio(_) => {
                        Err(UnsupportedContent::new("Claude `/messages`", "audio items"))
                    }
                    ContentPart::Video(_) => {
                        Err(UnsupportedContent::new("Claude `/messages`", "video items"))
                    }
                    ContentPart::Document(document) => Ok(ToolResultBlockParamContent::Document(
                        DocumentBlockParam::try_from(document)?,
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            TextOrItems::Items(items)
        };
        let item = ContentBlockParam::ToolResult(
            ToolResultBlockParam::new(message.call_id).with_content(content),
        );

        Ok(MessageParam::user(TextOrItems::Items(vec![item])))
    }
}
impl TryFrom<ContentPart> for ContentBlockParam {
    type Error = UnsupportedContent;

    fn try_from(part: ContentPart) -> Result<Self, Self::Error> {
        let content = match part {
            ContentPart::Text(text) => ContentBlockParam::Text(TextBlockParam::new(text)),
            ContentPart::Image(image) => {
                ContentBlockParam::Image(ImageBlockParam::try_from(image)?)
            }
            ContentPart::Audio(_) => {
                return Err(UnsupportedContent::new("Claude `/messages`", "audio items"));
            }
            ContentPart::Video(_) => {
                return Err(UnsupportedContent::new("Claude `/messages`", "video items"));
            }
            ContentPart::Document(document) => {
                ContentBlockParam::Document(DocumentBlockParam::try_from(document)?)
            }
        };
        Ok(content)
    }
}

fn map_content(
    content: Vec<ContentPart>,
) -> Result<TextOrItems<ContentBlockParam>, UnsupportedContent> {
    if matches!(content.as_slice(), [ContentPart::Text(_)]) {
        let Some(ContentPart::Text(text)) = content.into_iter().next() else {
            unreachable!("Text content checked");
        };
        return Ok(TextOrItems::Text(text));
    }

    let items = content
        .into_iter()
        .map(ContentBlockParam::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TextOrItems::Items(items))
}

impl TryFrom<ImageContent> for ImageBlockParam {
    type Error = UnsupportedContent;

    fn try_from(image: ImageContent) -> Result<Self, Self::Error> {
        let source = match image.source {
            MediaSource::Url(url) => ImageSource::Url(URLImageSource::new(url)),
            MediaSource::Bytes(raw) => ImageSource::Base64(Base64ImageSource::new(
                raw.to_vec(),
                Base64ImageType::from(image.mime),
            )),
            MediaSource::FileId(file_id) => ImageSource::File(FileImageSource::new(file_id)),
        };
        Ok(ImageBlockParam::new(source))
    }
}

impl From<ImageMimeType> for Base64ImageType {
    fn from(media_type: ImageMimeType) -> Self {
        match media_type {
            ImageMimeType::Jpeg => Base64ImageType::Jpeg,
            ImageMimeType::Png => Base64ImageType::Png,
            ImageMimeType::Gif => Base64ImageType::Gif,
            ImageMimeType::Webp => Base64ImageType::Webp,
        }
    }
}

impl TryFrom<DocumentContent> for DocumentBlockParam {
    type Error = UnsupportedContent;

    fn try_from(document: DocumentContent) -> Result<Self, Self::Error> {
        let source = match document.source {
            MediaSource::Url(url) => DocumentBlockParamSource::Url(URLPDFSource::new(url)),
            MediaSource::Bytes(raw) => {
                let data = BASE64_STANDARD.encode(raw);
                let media_type = Base64PDFType::try_from(document.mime)?;
                DocumentBlockParamSource::Base64(Base64PDFSource::new(data, media_type))
            }
            MediaSource::FileId(file_id) => {
                DocumentBlockParamSource::File(FileDocumentSource::new(file_id))
            }
        };
        let mut item = DocumentBlockParam::new(source);
        item.title = document.name;
        Ok(item)
    }
}

impl TryFrom<DocumentMimeType> for Base64PDFType {
    type Error = UnsupportedContent;

    fn try_from(mime: DocumentMimeType) -> Result<Self, Self::Error> {
        match mime {
            DocumentMimeType::Pdf => Ok(Base64PDFType::Pdf),
            DocumentMimeType::Doc => Err(UnsupportedContent::new(
                "Claude `/messages`",
                ".doc document items",
            )),
            DocumentMimeType::Xls => Err(UnsupportedContent::new(
                "Claude `/messages`",
                ".xls document items",
            )),
            DocumentMimeType::Txt => Err(UnsupportedContent::new(
                "Claude `/messages`",
                ".txt document items",
            )),
        }
    }
}

impl From<Vec<ContentBlock>> for LlmStep {
    fn from(content: Vec<ContentBlock>) -> Self {
        let mut thoughts = Vec::new();
        let mut texts = Vec::new();
        let mut tool_calls = Vec::new();

        for item in content {
            match item {
                ContentBlock::Text(text) => texts.push(text.text),
                ContentBlock::Thinking(thinking) => thoughts.push(thinking.thinking),
                ContentBlock::ToolUse(tool_use) => tool_calls.push(ToolCall::from(tool_use)),
                ContentBlock::RedactedThinking(_) => (),
                ContentBlock::Unknown(other) => {
                    log::warn!("Unexpected message item: {other}")
                }
                _ => {}
            }
        }
        LlmStep::from_parts(thoughts, texts, tool_calls)
    }
}

impl From<FunctionSpec> for ToolUnion {
    fn from(tool: FunctionSpec) -> Self {
        let mut custom = Tool::new(tool.name, tool.parameters);
        custom.description = tool.description;
        ToolUnion::Custom(custom)
    }
}

impl From<McpServerHttp> for (McpServer, McpToolset) {
    fn from(tool: McpServerHttp) -> Self {
        let (default_config, configs) = match tool.allowed_tools {
            Some(names) => {
                let configs = names
                    .into_iter()
                    .map(|name| (name, McpToolConfig::enabled(true)))
                    .collect::<HashMap<_, _>>();
                (McpToolConfig::enabled(false), Some(configs))
            }
            None => (McpToolConfig::enabled(true), None),
        };

        let mut server_url = McpServerUrl::new(
            tool.name.clone(),
            tool.transport.url.expose_secret().to_owned(),
        );
        server_url.authorization_token = tool
            .transport
            .authorization
            .map(|auth| auth.expose_secret().to_owned());
        let server = McpServer::Url(server_url);

        let mut toolset = McpToolset::new(tool.name).with_default_config(default_config);
        toolset.configs = configs;
        (server, toolset)
    }
}

impl From<ResponseFormat> for Option<JSONOutputFormat> {
    fn from(format: ResponseFormat) -> Self {
        match format {
            ResponseFormat::Text => None,
            ResponseFormat::JsonObject => Some(JSONOutputFormat::from(schemars::schema_for!(
                serde_json::Value
            ))),
            ResponseFormat::JsonSchema(schema) => Some(JSONOutputFormat::from(schema.schema)),
        }
    }
}

pub(super) fn map_reasoning_effort(
    effort: ReasoningEffort,
) -> (Option<ThinkingConfigParam>, Option<OutputEffort>) {
    let config_adaptive = ThinkingConfigParam::Adaptive(
        ThinkingConfigAdaptive::new().with_display(ThinkingDisplay::Summarized),
    );

    match effort {
        ReasoningEffort::None => (Some(ThinkingConfigParam::Disabled), None),
        ReasoningEffort::Minimal => (Some(config_adaptive), Some(OutputEffort::Low)),
        ReasoningEffort::Low => (Some(config_adaptive), Some(OutputEffort::Low)),
        ReasoningEffort::Medium => (Some(config_adaptive), Some(OutputEffort::Medium)),
        ReasoningEffort::High => (Some(config_adaptive), Some(OutputEffort::High)),
        ReasoningEffort::Xhigh => (Some(config_adaptive), Some(OutputEffort::XHigh)),
        ReasoningEffort::Max => (Some(config_adaptive), Some(OutputEffort::Max)),
    }
}

impl From<ToolChoice> for ClaudeToolChoice {
    fn from(choice: ToolChoice) -> Self {
        match choice {
            ToolChoice::None => ClaudeToolChoice::None,
            ToolChoice::Auto => ClaudeToolChoice::auto(),
            ToolChoice::Required => ClaudeToolChoice::any(),
            ToolChoice::Function(name) => ClaudeToolChoice::tool(name),
            ToolChoice::Mcp(name, _) => ClaudeToolChoice::tool(name),
            ToolChoice::ProviderSpecific(id) => {
                log::warn!("Provider specific tool choice {id} not recognized");
                ClaudeToolChoice::auto()
            }
        }
    }
}
