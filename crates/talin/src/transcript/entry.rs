use uuid::Uuid;

use crate::llm::{LlmRequestItem, LlmRequestSystem, UnsupportedContent};
use crate::schemas::{AssistantMessage, BodyMessage, LlmStep, ToolErrorMessage, TruncatedStep};

/// An entry of a transcript, generic over the provider-native item type `N`.
#[derive(Debug, Clone)]
pub enum Entry<N> {
    /// Message entry holding the canonical message and the original provider-specific item.
    Message(Uuid, MessageRecord<N>),
    /// A tool error result.
    ///
    /// Only produced for providers whose driver can represent an erroneous tool result
    /// (see [`LlmRequestItem::from_tool_error`]).
    ToolError(Uuid, ToolErrorRecord<N>),
    /// Provider-native content without a canonical message.
    Native(Uuid, N),
    /// A marker indicating the beginning of a step.
    ///
    /// A step refers to a single tool call and its results.
    StepMarker,
    /// A marker indicating the beginning of a turn.
    ///
    /// A turn refers to a single user query, the subsequent tool calls, and the model answer.
    TurnMarker,
    /// Records that a compaction happened at this position.
    ///
    /// Advisory unless `boundary` is set: native compaction items decide what requests omit. A
    /// boundary marker, used for summaries that are not a native compaction, makes
    /// [`Llm::materialize`](crate::llm::Llm::materialize) start requests after it.
    CompactionMarker {
        /// Whether requests start after this marker.
        boundary: bool,
    },
}

impl<N> Entry<N> {
    /// Consumes the entry, returning its native content, or `None` for a marker.
    pub fn into_native(self) -> Option<N> {
        match self {
            Self::Message(_, record) => Some(record.native),
            Self::ToolError(_, record) => Some(record.native),
            Self::Native(_, native) => Some(native),
            Self::StepMarker | Self::TurnMarker | Self::CompactionMarker { .. } => None,
        }
    }
}

/// Provider-agnostic canonical representation paired with the original provider-specific item.
#[derive(Debug, Clone)]
pub struct Record<C, N> {
    pub(crate) canonical: C,
    pub(crate) native: N,
}

impl<C, N> Record<C, N> {
    /// The provider-agnostic representation of this record.
    pub fn canonical(&self) -> &C {
        &self.canonical
    }

    /// The original provider-specific representation of this record.
    pub fn native(&self) -> &N {
        &self.native
    }

    /// Consumes the record, returning its canonical and native representations.
    pub fn into_parts(self) -> (C, N) {
        (self.canonical, self.native)
    }
}

/// A [`Record`] of the system prompt contents.
pub type SystemRecord<N> = Record<Vec<String>, N>;
/// A [`Record`] of a [`BodyMessage`].
pub type MessageRecord<N> = Record<BodyMessage, N>;
/// A [`Record`] of a model output.
pub type ModelRecord<N> = Record<LlmStep, N>;
/// A [`Record`] of a tool error.
pub type ToolErrorRecord<N> = Record<ToolErrorMessage, N>;

/// A model output cut short by the output token limit, ready to be continued from.
pub type TruncatedRecord<N> = Record<TruncatedStep, N>;

impl<N> From<MessageRecord<N>> for Entry<N> {
    fn from(record: MessageRecord<N>) -> Self {
        Entry::Message(Uuid::new_v4(), record)
    }
}
impl<N> From<ToolErrorRecord<N>> for Entry<N> {
    fn from(record: ToolErrorRecord<N>) -> Self {
        Entry::ToolError(Uuid::new_v4(), record)
    }
}
impl<N> TryFrom<BodyMessage> for Entry<N>
where
    N: LlmRequestItem,
{
    type Error = UnsupportedContent;

    fn try_from(message: BodyMessage) -> Result<Self, Self::Error> {
        Ok(Entry::from(MessageRecord::new(message)?))
    }
}
impl<N> From<ModelRecord<N>> for MessageRecord<N> {
    fn from(record: ModelRecord<N>) -> Self {
        Self {
            canonical: BodyMessage::from(record.canonical),
            native: record.native,
        }
    }
}

impl<N: LlmRequestSystem> SystemRecord<N> {
    /// Creates a new [`SystemRecord`] with the given contents.
    ///
    /// The native representation is built from the contents.
    pub fn new(contents: Vec<String>) -> Self {
        let native = N::from_contents(contents.clone());
        Self {
            canonical: contents,
            native,
        }
    }

    /// Replaces the contents of the record.
    ///
    /// The native representation is rebuilt from the new contents, using the previous
    /// representation as a base.
    pub fn modified(mut self, new_contents: Vec<String>) -> Self {
        self.native = N::from_contents_with(new_contents.clone(), self.native);
        self.canonical = new_contents;
        self
    }
}

impl<N: LlmRequestItem> MessageRecord<N> {
    /// Creates a new [`MessageRecord`] with the given message.
    ///
    /// The native representation is built from the message.
    ///
    /// Returns error if the message holds content the provider does not support.
    pub fn new(message: BodyMessage) -> Result<Self, UnsupportedContent> {
        let native = N::from_message(message.clone())?;
        Ok(Self {
            canonical: message,
            native,
        })
    }

    /// Replaces the message of the record.
    ///
    /// The native representation is rebuilt from the new message, using the previous
    /// representation as a base.
    ///
    /// Returns error if the new message holds content the provider does not support.
    pub fn modified(
        mut self,
        new_message: BodyMessage,
        retain: bool,
    ) -> Result<Self, UnsupportedContent> {
        self.native = N::from_message_with(new_message.clone(), self.native, retain)?;
        self.canonical = new_message;
        Ok(self)
    }
}

impl<N: LlmRequestItem> ModelRecord<N> {
    /// Returns the text that can be continued, excluding incomplete client calls.
    /// Native reasoning is retained.
    pub fn into_truncated(self) -> Result<Option<TruncatedRecord<N>>, UnsupportedContent> {
        let is_tool_call = matches!(self.canonical, LlmStep::ToolCall(_));
        let message = match self.canonical {
            LlmStep::Finish(message) if !message.content.is_empty() => message,
            LlmStep::ToolCall(message) if !message.content.is_empty() => {
                AssistantMessage::from_parts(message.thought, message.content)
            }
            _ => return Ok(None),
        };
        let canonical = TruncatedStep(message);
        let native = if is_tool_call {
            N::from_message_with(
                BodyMessage::Assistant(canonical.0.clone()),
                self.native,
                true,
            )?
        } else {
            self.native
        };
        Ok(Some(Record { canonical, native }))
    }
}

impl<N: LlmRequestItem> TruncatedRecord<N> {
    /// Combines a partial response and its continuation, preserving their native content.
    pub fn concat(
        self,
        continuation: ModelRecord<N>,
    ) -> Result<ModelRecord<N>, UnsupportedContent> {
        let canonical = self.canonical.concat(continuation.canonical);
        let native = self.native.concat(continuation.native)?;
        Ok(Record { canonical, native })
    }
}

/// A read-only view of a single transcript entry, as returned by
/// [`AgentTranscript::view`](super::AgentTranscript::view).
#[derive(Debug, Clone, Copy)]
pub enum EntryView<'a> {
    /// A message entry, addressable by its `uuid`.
    Message(Uuid, &'a BodyMessage),
    /// A tool error entry, addressable by its `uuid`.
    ToolError(Uuid, &'a ToolErrorMessage),
    /// Provider-native content, inspectable through
    /// [`AgentTranscript::entries`](super::AgentTranscript::entries).
    Native(Uuid),
    /// A marker indicating the beginning of a step.
    ///
    /// A step refers to a single tool call and its results.
    StepMarker,
    /// A marker indicating the beginning of a turn.
    ///
    /// A turn refers to a single user query, the subsequent tool calls, and the model answer.
    TurnMarker,
    /// A compaction marker; see [`Entry::CompactionMarker`].
    CompactionMarker {
        /// Whether requests start after this marker.
        boundary: bool,
    },
}

/// An edit applied to a transcript range by
/// [`AgentTranscript::modify`](super::AgentTranscript::modify).
pub enum EntryEdit {
    /// Keep the message, tool error, or native entry with the given `uuid` unchanged.
    Keep(Uuid),
    /// Replace the entry with the given `uuid` with a message.
    ///
    /// The boolean flags whether to reuse the original provider-specific item as a base.
    /// It has no effect when the target is a tool error.
    Replace(Uuid, BodyMessage, bool),
    /// Insert a new message with a fresh `uuid`.
    Insert(BodyMessage),
    /// Place a step marker at this position.
    ///
    /// A step refers to a single tool call and its results.
    StepMarker,
    /// Place a turn marker at this position.
    ///
    /// A turn refers to a single user query, the subsequent tool calls, and the model answer.
    TurnMarker,
    /// Place a compaction marker at this position; see [`Entry::CompactionMarker`].
    CompactionMarker {
        /// Whether requests start after this marker.
        boundary: bool,
    },
}

impl From<EntryView<'_>> for EntryEdit {
    fn from(view: EntryView<'_>) -> Self {
        match view {
            EntryView::Message(id, _) | EntryView::ToolError(id, _) | EntryView::Native(id) => {
                EntryEdit::Keep(id)
            }
            EntryView::StepMarker => EntryEdit::StepMarker,
            EntryView::TurnMarker => EntryEdit::TurnMarker,
            EntryView::CompactionMarker { boundary } => EntryEdit::CompactionMarker { boundary },
        }
    }
}
