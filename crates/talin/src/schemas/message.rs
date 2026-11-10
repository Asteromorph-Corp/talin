use itertools::{Either, Itertools};
use serde::{Deserialize, Serialize};

use crate::schemas::{
    AudioContent, ContentPart, DocumentContent, ImageContent, ToolCall, ToolOutput, ToolResult,
    VideoContent,
};
use crate::tools::{ToolError, ToolErrorFeedback};

/// A single message of an LLM interaction.
///
/// # Usage
/// ```
/// use talin::schemas::Message;
///
/// let human_message = Message::new_human_message("Hello");
/// let system_message = Message::new_system_message("System Alert");
/// let ai_message = Message::new_ai_message(None, "AI Response");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    /// A system message.
    System(SystemMessage),
    /// A user message.
    User(UserMessage),
    /// An assistant message.
    Assistant(AssistantMessage),
    /// A tool call message.
    ToolCall(ToolCallMessage),
    /// A tool result message.
    Tool(ToolMessage),
}

/// A single message of an LLM interaction.
///
/// # Usage
/// ```
/// use talin::schemas::Message;
///
/// let human_message = Message::new_human_message("Hello");
/// let system_message = Message::new_system_message("System Alert");
/// let ai_message = Message::new_ai_message(None, "AI Response");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum BodyMessage {
    /// A user message.
    User(UserMessage),
    /// An assistant message.
    Assistant(AssistantMessage),
    /// A tool call message.
    ToolCall(ToolCallMessage),
    /// A tool result message.
    Tool(ToolMessage),
}

/// A system message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    /// The text content of the message.
    pub content: String,
}

/// A user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    /// The content of the message.
    pub content: Vec<ContentPart>,
}

/// An assistant message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantMessage {
    /// The thought content of the message.
    pub thought: Option<String>,
    /// The content of the message.
    pub content: Vec<ContentPart>,
}

/// A tool call message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallMessage {
    /// Internal model reasoning attached to the message.
    pub thought: Option<String>,
    /// The message content.
    pub content: Vec<ContentPart>,
    /// The tool calls attached to the message.
    pub tool_calls: Vec<ToolCall>,
}

/// A tool output message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMessage {
    /// The id of the tool call the tool result is for.
    pub call_id: String,
    /// The name of the tool.
    pub name: String,
    /// The tool result content.
    pub output: ToolOutput,
}

/// An erroneous tool result.
#[derive(Debug, Clone)]
pub struct ToolErrorMessage {
    /// The id of the tool call the error is for.
    pub call_id: String,
    /// The name of the tool.
    pub name: String,
    /// The feedback string for the error.
    pub feedback: ToolErrorFeedback,
}

/// The outcome of a tool call, either the tool's output or its error.
pub struct ToolResultMessage {
    /// The id of the tool call the result is for.
    pub call_id: String,
    /// The name of the tool.
    pub name: String,
    /// The result of the tool call.
    pub result: ToolResult,
}

impl Message {
    /// Constructs a new system message.
    pub fn new_system_message(content: impl Into<String>) -> Self {
        Message::System(SystemMessage::new(content))
    }

    /// Constructs a new user message.
    pub fn new_human_message(content: impl Into<String>) -> Self {
        Message::User(UserMessage::new(content))
    }

    /// Constructs a new AI message.
    pub fn new_ai_message(thought: Option<String>, content: impl Into<String>) -> Self {
        Message::Assistant(AssistantMessage::new(thought, content))
    }

    /// Constructs a new tool call message.
    pub fn new_tool_call_message(
        thought: Option<String>,
        tool_calls: impl IntoIterator<Item = ToolCall>,
    ) -> Self {
        Message::ToolCall(ToolCallMessage::new(
            thought,
            tool_calls.into_iter().collect(),
        ))
    }

    /// Constructs a new tool output message.
    pub fn new_tool_message(
        call_id: impl Into<String>,
        name: impl Into<String>,
        content: ToolOutput,
    ) -> Self {
        Message::Tool(ToolMessage::new(call_id, name, content))
    }

    /// Splits messages into the contents of the system messages and the remaining body messages.
    pub(crate) fn partition(messages: Vec<Self>) -> (Vec<String>, Vec<BodyMessage>) {
        messages.into_iter().partition_map(|msg| match msg {
            Message::System(m) => Either::Left(m.content),
            Message::User(m) => Either::Right(BodyMessage::User(m)),
            Message::Assistant(m) => Either::Right(BodyMessage::Assistant(m)),
            Message::ToolCall(m) => Either::Right(BodyMessage::ToolCall(m)),
            Message::Tool(m) => Either::Right(BodyMessage::Tool(m)),
        })
    }
}

impl BodyMessage {
    /// Returns the message content.
    ///
    /// This does not include tool calls.
    pub fn content(&self) -> &[ContentPart] {
        match self {
            BodyMessage::User(message) => &message.content,
            BodyMessage::Assistant(message) => &message.content,
            BodyMessage::ToolCall(message) => &message.content,
            BodyMessage::Tool(message) => &message.output.content,
        }
    }
}

impl SystemMessage {
    /// Creates a new [`SystemMessage`] with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl UserMessage {
    /// Creates a new [`UserMessage`] with the given text content.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentPart::Text(text.into())],
        }
    }

    /// Creates a [`UserMessage`] with the given content parts.
    pub fn from_parts(parts: impl IntoIterator<Item = ContentPart>) -> Self {
        Self {
            content: parts.into_iter().collect(),
        }
    }

    /// Appends images to the message content.
    pub fn with_images(self, images: Vec<ImageContent>) -> Self {
        self.with_media(images.into_iter().map(ContentPart::Image))
    }

    /// Appends audio clips to the message content.
    pub fn with_audios(self, audios: Vec<AudioContent>) -> Self {
        self.with_media(audios.into_iter().map(ContentPart::Audio))
    }

    /// Appends video clips to the message content.
    pub fn with_videos(self, videos: Vec<VideoContent>) -> Self {
        self.with_media(videos.into_iter().map(ContentPart::Video))
    }

    /// Appends documents to the message content.
    pub fn with_documents(self, documents: Vec<DocumentContent>) -> Self {
        self.with_media(documents.into_iter().map(ContentPart::Document))
    }

    /// Appends media parts to the message content.
    pub fn with_media(mut self, media: impl IntoIterator<Item = ContentPart>) -> Self {
        self.content.extend(media);
        self
    }
}

impl AssistantMessage {
    /// Creates a new [`AssistantMessage`] with the given text content.
    pub fn new(thought: Option<String>, text: impl Into<String>) -> Self {
        Self {
            thought,
            content: vec![ContentPart::Text(text.into())],
        }
    }

    /// Creates a new [`AssistantMessage`] with the given content parts.
    pub fn from_parts(
        thought: Option<String>,
        parts: impl IntoIterator<Item = ContentPart>,
    ) -> Self {
        Self {
            thought,
            content: parts.into_iter().collect(),
        }
    }

    /// Appends images to the message content.
    pub fn with_images(self, images: Vec<ImageContent>) -> Self {
        self.with_media(images.into_iter().map(ContentPart::Image))
    }

    /// Appends audio clips to the message content.
    pub fn with_audios(self, audios: Vec<AudioContent>) -> Self {
        self.with_media(audios.into_iter().map(ContentPart::Audio))
    }

    /// Appends video clips to the message content.
    pub fn with_videos(self, videos: Vec<VideoContent>) -> Self {
        self.with_media(videos.into_iter().map(ContentPart::Video))
    }

    /// Appends documents to the message content.
    pub fn with_documents(self, documents: Vec<DocumentContent>) -> Self {
        self.with_media(documents.into_iter().map(ContentPart::Document))
    }

    /// Appends media parts to the message content.
    pub fn with_media(mut self, media: impl IntoIterator<Item = ContentPart>) -> Self {
        self.content.extend(media);
        self
    }
}

impl ToolCallMessage {
    /// Creates a new [`ToolCallMessage`] with the given tool calls.
    pub fn new(thought: Option<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            thought,
            content: vec![],
            tool_calls,
        }
    }

    /// Creates a new [`ToolCallMessage`] with the given content parts and tool calls.
    pub fn from_parts(
        thought: Option<String>,
        parts: impl IntoIterator<Item = ContentPart>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            thought,
            content: parts.into_iter().collect(),
            tool_calls,
        }
    }

    /// Split the message into its content and tool calls.
    pub(crate) fn split(self) -> (AssistantMessage, Vec<ToolCall>) {
        let message = AssistantMessage::from_parts(self.thought, self.content);
        (message, self.tool_calls)
    }
}

impl ToolMessage {
    /// Creates a new [`ToolMessage`] with the given tool call id, tool name, and output.
    pub fn new(call_id: impl Into<String>, name: impl Into<String>, output: ToolOutput) -> Self {
        Self {
            call_id: call_id.into(),
            name: name.into(),
            output,
        }
    }
}

impl ToolErrorMessage {
    /// Creates a new [`ToolErrorMessage`] with the given tool call id, tool name, and error.
    pub fn new(call_id: impl Into<String>, name: impl Into<String>, error: ToolError) -> Self {
        Self {
            call_id: call_id.into(),
            name: name.into(),
            feedback: error.feedback(),
        }
    }

    /// Renders the error to a human-readable string.
    pub fn render(&self) -> String {
        format!("Tool `{}` failed: {}", self.name, self.feedback)
    }

    /// Converts the error into a [`ToolMessage`] whose output is the [rendered](Self::render)
    /// error.
    pub(crate) fn as_output(&self) -> ToolMessage {
        let output = ToolOutput::new(vec![ContentPart::Text(self.render())]);
        ToolMessage::new(self.call_id.clone(), self.name.clone(), output)
    }
}

impl ToolResultMessage {
    /// Creates a [`ToolResultMessage`] with the given tool call id, tool name, and result.
    pub fn new(call_id: impl Into<String>, name: impl Into<String>, result: ToolResult) -> Self {
        Self {
            call_id: call_id.into(),
            name: name.into(),
            result,
        }
    }
}

impl From<BodyMessage> for Message {
    fn from(message: BodyMessage) -> Self {
        match message {
            BodyMessage::User(m) => Message::User(m),
            BodyMessage::Assistant(m) => Message::Assistant(m),
            BodyMessage::ToolCall(m) => Message::ToolCall(m),
            BodyMessage::Tool(m) => Message::Tool(m),
        }
    }
}

impl From<SystemMessage> for Message {
    fn from(message: SystemMessage) -> Self {
        Message::System(message)
    }
}

impl From<UserMessage> for Message {
    fn from(message: UserMessage) -> Self {
        Message::User(message)
    }
}

impl From<AssistantMessage> for Message {
    fn from(message: AssistantMessage) -> Self {
        Message::Assistant(message)
    }
}

impl From<ToolMessage> for Message {
    fn from(message: ToolMessage) -> Self {
        Message::Tool(message)
    }
}

impl<T: Into<String>> From<T> for SystemMessage {
    fn from(content: T) -> Self {
        SystemMessage::new(content)
    }
}

impl<T: Into<String>> From<T> for UserMessage {
    fn from(content: T) -> Self {
        UserMessage::new(content)
    }
}

impl<T: Into<String>> From<T> for AssistantMessage {
    fn from(content: T) -> Self {
        AssistantMessage::new(None, content)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::System(message) => {
                writeln!(f, "System message:\n{message}")
            }
            Message::User(message) => writeln!(f, "User message:\n{message}"),
            Message::Assistant(message) => writeln!(f, "Assistant message:\n{message}"),
            Message::ToolCall(message) => writeln!(f, "Tool call message:\n{message}"),
            Message::Tool(message) => writeln!(f, "Tool message:\n{message}"),
        }
    }
}

impl std::fmt::Display for SystemMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl std::fmt::Display for UserMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.content.is_empty() {
            return write!(f, "(Empty message)");
        }

        for part in &self.content {
            writeln!(f, "{part}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for AssistantMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.content.is_empty() && self.thought.is_none() {
            return write!(f, "(Empty message)");
        }

        if let Some(thought) = &self.thought {
            writeln!(f, "Thought: {thought}")?;
        }
        for part in &self.content {
            writeln!(f, "{part}")?;
        }

        Ok(())
    }
}

impl std::fmt::Display for ToolCallMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(thought) = &self.thought {
            writeln!(f, "{thought}")?;
        }

        writeln!(f, "Tool calls:")?;
        for call in &self.tool_calls {
            writeln!(f, "\n{call}")?;
        }

        Ok(())
    }
}

impl std::fmt::Display for ToolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tool `{}` output:", self.name)?;
        writeln!(f, "---")?;
        for item in &self.output.content {
            writeln!(f, "{item}")?;
            writeln!(f, "---")?;
        }
        Ok(())
    }
}
