use talin_provider::gemini_interactions::types::{
    AllowedTools, AudioContent as GeminiAudioContent, AudioMimeType as GeminiAudioMimeType,
    Content, DocumentContent as GeminiDocumentContent, DocumentMimeType as GeminiDocumentMimeType,
    Function, FunctionCallStep, FunctionResultStep, FunctionResultSubContent, FunctionResultValue,
    ImageContent as GeminiImageContent, ImageMimeType as GeminiImageMimeType, Step, TextContent,
    ThinkingLevel, ToolChoice as GeminiToolChoice, ToolChoiceConfig, ToolMode,
    VideoContent as GeminiVideoContent, VideoMimeType as GeminiVideoMimeType,
};

use crate::llm::UnsupportedContent;
use crate::schemas::{
    AssistantMessage, AudioContent, AudioMimeType, BodyMessage, ContentPart, DocumentContent,
    DocumentMimeType, ImageContent, ImageMimeType, LlmStep, MediaSource, ReasoningEffort, ToolCall,
    ToolCallMessage, ToolChoice, ToolMessage, UserMessage, VideoContent, VideoMimeType,
};
use crate::tools::FunctionSpec;

impl From<FunctionSpec> for Function {
    fn from(spec: FunctionSpec) -> Self {
        Function::new(spec.name)
            .with_description(spec.description.unwrap_or_default())
            .with_parameters(spec.parameters)
    }
}

impl TryFrom<BodyMessage> for Vec<Step> {
    type Error = UnsupportedContent;

    fn try_from(message: BodyMessage) -> Result<Self, Self::Error> {
        match message {
            BodyMessage::User(msg) => Vec::<Step>::try_from(msg),
            BodyMessage::Assistant(msg) => Vec::<Step>::try_from(msg),
            BodyMessage::ToolCall(msg) => Vec::<Step>::try_from(msg),
            BodyMessage::Tool(msg) => Vec::<Step>::try_from(msg),
        }
    }
}

impl TryFrom<UserMessage> for Vec<Step> {
    type Error = UnsupportedContent;

    fn try_from(message: UserMessage) -> Result<Self, Self::Error> {
        let parts = message
            .content
            .into_iter()
            .map(Content::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(vec![Step::user_input(parts)])
    }
}

impl TryFrom<AssistantMessage> for Vec<Step> {
    type Error = UnsupportedContent;

    fn try_from(message: AssistantMessage) -> Result<Self, Self::Error> {
        let parts = message
            .content
            .into_iter()
            .map(Content::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(vec![Step::model_output(parts)])
    }
}

impl TryFrom<ToolCallMessage> for Vec<Step> {
    type Error = UnsupportedContent;

    fn try_from(message: ToolCallMessage) -> Result<Self, Self::Error> {
        let (message, tool_calls) = message.split();
        let item = Self::try_from(message)?;
        let tool_calls = tool_calls.into_iter().map(|call| {
            Step::FunctionCall(FunctionCallStep::new(call.id, call.name, call.arguments))
        });
        Ok(item.into_iter().chain(tool_calls).collect::<Vec<_>>())
    }
}

impl TryFrom<ToolMessage> for Vec<Step> {
    type Error = UnsupportedContent;

    fn try_from(message: ToolMessage) -> Result<Self, Self::Error> {
        let result = if let Some(text) = message.output.as_text() {
            FunctionResultValue::Text(text.to_owned())
        } else {
            let items = message.output.content;
            if !items
                .iter()
                .all(|item| matches!(item, ContentPart::Text(_) | ContentPart::Image(_)))
            {
                return Err(UnsupportedContent::new(
                    "Gemini `/interactions`",
                    "tool result must be text or image content",
                ));
            }
            let items = items
                .into_iter()
                .map(FunctionResultSubContent::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            FunctionResultValue::Blocks(items)
        };

        let step = FunctionResultStep::new(message.call_id, result).with_name(message.name);
        Ok(vec![Step::FunctionResult(step)])
    }
}

impl TryFrom<ContentPart> for Content {
    type Error = UnsupportedContent;

    fn try_from(part: ContentPart) -> Result<Self, Self::Error> {
        match part {
            ContentPart::Text(text) => Ok(Content::Text(TextContent::new(text))),
            ContentPart::Image(image) => Ok(Content::Image(GeminiImageContent::try_from(image)?)),
            ContentPart::Audio(audio) => Ok(Content::Audio(GeminiAudioContent::try_from(audio)?)),
            ContentPart::Video(video) => Ok(Content::Video(GeminiVideoContent::try_from(video)?)),
            ContentPart::Document(document) => Ok(Content::Document(
                GeminiDocumentContent::try_from(document)?,
            )),
        }
    }
}
impl TryFrom<ContentPart> for FunctionResultSubContent {
    type Error = UnsupportedContent;

    fn try_from(part: ContentPart) -> Result<Self, Self::Error> {
        match part {
            ContentPart::Text(text) => Ok(FunctionResultSubContent::Text(TextContent::new(text))),
            ContentPart::Image(image) => Ok(FunctionResultSubContent::Image(
                GeminiImageContent::try_from(image)?,
            )),
            ContentPart::Audio(_) => Err(UnsupportedContent::new(
                "Gemini `/interactions`",
                "audio as function result",
            )),
            ContentPart::Video(_) => Err(UnsupportedContent::new(
                "Gemini `/interactions`",
                "video as function result",
            )),
            ContentPart::Document(_) => Err(UnsupportedContent::new(
                "Gemini `/interactions`",
                "document as function result",
            )),
        }
    }
}

impl TryFrom<ImageContent> for GeminiImageContent {
    type Error = UnsupportedContent;

    fn try_from(image: ImageContent) -> Result<Self, Self::Error> {
        let mime = GeminiImageMimeType::from(image.mime);
        let mut out = GeminiImageContent::new().with_mime_type(mime);
        match image.source {
            MediaSource::Bytes(raw) => out = out.with_data(raw.to_vec()),
            MediaSource::FileId(file_id) => out = out.with_uri(file_id),
            MediaSource::Url(uri) => out = out.with_uri(uri),
        }
        Ok(out)
    }
}
impl TryFrom<AudioContent> for GeminiAudioContent {
    type Error = UnsupportedContent;

    fn try_from(audio: AudioContent) -> Result<Self, Self::Error> {
        let mime = GeminiAudioMimeType::from(audio.mime);
        let mut out = GeminiAudioContent::new().with_mime_type(mime);
        match audio.source {
            MediaSource::Bytes(raw) => out = out.with_data(raw.to_vec()),
            MediaSource::FileId(file_id) => out = out.with_uri(file_id),
            MediaSource::Url(uri) => out = out.with_uri(uri),
        }
        Ok(out)
    }
}
impl TryFrom<VideoContent> for GeminiVideoContent {
    type Error = UnsupportedContent;

    fn try_from(video: VideoContent) -> Result<Self, Self::Error> {
        let mime = GeminiVideoMimeType::try_from(video.mime)?;
        let mut out = GeminiVideoContent::new().with_mime_type(mime);
        match video.source {
            MediaSource::Bytes(raw) => out = out.with_data(raw.to_vec()),
            MediaSource::FileId(file_id) => out = out.with_uri(file_id),
            MediaSource::Url(uri) => out = out.with_uri(uri),
        }
        Ok(out)
    }
}
impl TryFrom<DocumentContent> for GeminiDocumentContent {
    type Error = UnsupportedContent;

    fn try_from(document: DocumentContent) -> Result<Self, Self::Error> {
        let mime = GeminiDocumentMimeType::try_from(document.mime)?;
        let mut out = GeminiDocumentContent::new().with_mime_type(mime);
        match document.source {
            MediaSource::Bytes(raw) => out = out.with_data(raw.to_vec()),
            MediaSource::FileId(file_id) => out = out.with_uri(file_id),
            MediaSource::Url(uri) => out = out.with_uri(uri),
        }
        Ok(out)
    }
}

impl From<Vec<Step>> for LlmStep {
    fn from(steps: Vec<Step>) -> Self {
        let mut thoughts = Vec::new();
        let mut texts = Vec::new();
        let mut tool_calls = Vec::new();

        for step in steps {
            match step {
                Step::Thought(thought) => {
                    for content in thought.summary.into_iter().flatten() {
                        if let Content::Text(text) = content {
                            thoughts.push(text.text);
                        }
                    }
                }
                Step::ModelOutput(output) => {
                    for content in output.content.into_iter().flatten() {
                        if let Content::Text(text) = content {
                            texts.push(text.text);
                        }
                    }
                }
                Step::FunctionCall(call) => {
                    tool_calls.push(ToolCall::new(
                        Some(call.id),
                        call.name,
                        Some(call.arguments),
                    ));
                }
                _ => {}
            }
        }
        LlmStep::from_parts(thoughts, texts, tool_calls)
    }
}

impl From<ReasoningEffort> for ThinkingLevel {
    fn from(effort: ReasoningEffort) -> Self {
        match effort {
            ReasoningEffort::None => ThinkingLevel::Minimal,
            ReasoningEffort::Minimal => ThinkingLevel::Minimal,
            ReasoningEffort::Low => ThinkingLevel::Low,
            ReasoningEffort::Medium => ThinkingLevel::Medium,
            ReasoningEffort::High => ThinkingLevel::High,
            ReasoningEffort::Xhigh => ThinkingLevel::High,
            ReasoningEffort::Max => ThinkingLevel::High,
        }
    }
}

impl From<ToolChoice> for GeminiToolChoice {
    fn from(choice: ToolChoice) -> Self {
        match choice {
            ToolChoice::None => GeminiToolChoice::Mode(ToolMode::None),
            ToolChoice::Auto => GeminiToolChoice::Mode(ToolMode::Auto),
            ToolChoice::Required => GeminiToolChoice::Mode(ToolMode::Any),
            ToolChoice::Function(name) | ToolChoice::Mcp(name, _) => GeminiToolChoice::Config(
                ToolChoiceConfig::new(AllowedTools::new(ToolMode::Any).with_tools(vec![name])),
            ),
            ToolChoice::ProviderSpecific(id) => {
                log::warn!("Provider specific tool choice {id} not recognized, defaulting to auto");
                GeminiToolChoice::Mode(ToolMode::Auto)
            }
        }
    }
}

impl From<ImageMimeType> for GeminiImageMimeType {
    fn from(image_type: ImageMimeType) -> Self {
        match image_type {
            ImageMimeType::Jpeg => GeminiImageMimeType::Jpeg,
            ImageMimeType::Png => GeminiImageMimeType::Png,
            ImageMimeType::Gif => GeminiImageMimeType::Gif,
            ImageMimeType::Webp => GeminiImageMimeType::Webp,
        }
    }
}

impl From<AudioMimeType> for GeminiAudioMimeType {
    fn from(audio_type: AudioMimeType) -> Self {
        match audio_type {
            AudioMimeType::Mpeg => GeminiAudioMimeType::Mpeg,
            AudioMimeType::Mp3 => GeminiAudioMimeType::Mp3,
            AudioMimeType::Wav => GeminiAudioMimeType::Wav,
            AudioMimeType::Ogg => GeminiAudioMimeType::Ogg,
        }
    }
}

impl TryFrom<VideoMimeType> for GeminiVideoMimeType {
    type Error = UnsupportedContent;

    fn try_from(video_type: VideoMimeType) -> Result<Self, Self::Error> {
        match video_type {
            VideoMimeType::Mp4 => Ok(GeminiVideoMimeType::Mp4),
            VideoMimeType::Webm => Ok(GeminiVideoMimeType::Webm),
            VideoMimeType::Ogg => Err(UnsupportedContent::new(
                "Gemini `/interactions`",
                ".ogg video",
            )),
            VideoMimeType::Mpeg => Ok(GeminiVideoMimeType::Mpeg),
        }
    }
}

impl TryFrom<DocumentMimeType> for GeminiDocumentMimeType {
    type Error = UnsupportedContent;

    fn try_from(document_type: DocumentMimeType) -> Result<Self, Self::Error> {
        match document_type {
            DocumentMimeType::Pdf => Ok(GeminiDocumentMimeType::Pdf),
            DocumentMimeType::Doc => Err(UnsupportedContent::new(
                "Gemini `/interactions`",
                ".doc document",
            )),
            DocumentMimeType::Xls => Err(UnsupportedContent::new(
                "Gemini `/interactions`",
                ".xls document",
            )),
            DocumentMimeType::Txt => Err(UnsupportedContent::new(
                "Gemini `/interactions`",
                ".txt document",
            )),
        }
    }
}
