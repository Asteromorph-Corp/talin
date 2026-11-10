use serde::{Deserialize, Serialize};

use crate::utils::tagged_enum;
use crate::xai_responses::types::{ModelOutput, ModelResponse, OutputMessageContent, TokenLogProb};

tagged_enum! {
    /// Events returned in the stream.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type")]
    pub enum StreamEvent {
        /// Response creation event.
        #[serde(rename = "response.created")]
        Created(ResponseEvent),
        /// Response in progress event.
        #[serde(rename = "response.in_progress")]
        InProgress(ResponseEvent),
        /// Response completion event.
        #[serde(rename = "response.completed")]
        Completed(ResponseEvent),
        /// Output item added event.
        #[serde(rename = "response.output_item.added")]
        OutputItemAdded(OutputItemEvent),
        /// Output item done event.
        #[serde(rename = "response.output_item.done")]
        OutputItemDone(OutputItemEvent),
        /// Content part added event.
        #[serde(rename = "response.content_part.added")]
        ContentPartAdded(ContentPartEvent),
        /// Content part done event.
        #[serde(rename = "response.content_part.done")]
        ContentPartDone(ContentPartEvent),
        /// Output text delta event.
        #[serde(rename = "response.output_text.delta")]
        OutputTextDelta(OutputTextDeltaEvent),
        /// Output text done event.
        #[serde(rename = "response.output_text.done")]
        OutputTextDone(OutputTextDoneEvent),
    }
}

/// Event data for response status updates.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ResponseEvent {
    /// The sequence number of the event.
    pub sequence_number: Option<i32>,
    /// The response object.
    pub response: ModelResponse,
}

/// Event data for output item updates.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OutputItemEvent {
    /// The sequence number of the event.
    pub sequence_number: Option<i32>,
    /// The index of the output item.
    pub output_index: usize,
    /// The output item.
    pub item: ModelOutput,
}

/// Event data for content part updates.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContentPartEvent {
    /// The sequence number of the event.
    pub sequence_number: Option<i32>,
    /// The index of the output item.
    pub output_index: usize,
    /// The index of the content part.
    pub content_index: usize,
    /// The ID of the item.
    pub item_id: String,
    /// The content part.
    pub part: OutputMessageContent,
}

/// Event data for output text deltas.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OutputTextDeltaEvent {
    /// The sequence number of the event.
    pub sequence_number: Option<i32>,
    /// The index of the output item.
    pub output_index: usize,
    /// The index of the content part.
    pub content_index: usize,
    /// The ID of the item.
    pub item_id: String,
    /// The text delta.
    pub delta: String,
    /// Log probabilities.
    pub logprobs: Option<Vec<TokenLogProb>>,
}

/// Event data for output text completion.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OutputTextDoneEvent {
    /// The sequence number of the event.
    pub sequence_number: Option<i32>,
    /// The index of the output item.
    pub output_index: usize,
    /// The index of the content part.
    pub content_index: usize,
    /// The ID of the item.
    pub item_id: String,
    /// The final text.
    pub text: String,
}

impl ResponseEvent {
    /// Creates a [`ResponseEvent`] carrying the given response.
    pub fn new(response: ModelResponse) -> Self {
        Self {
            sequence_number: None,
            response,
        }
    }

    /// Sets the sequence number of the event.
    pub fn with_sequence_number(mut self, sequence_number: i32) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }
}

impl OutputItemEvent {
    /// Creates an [`OutputItemEvent`] for the output item at the given index.
    pub fn new(output_index: usize, item: ModelOutput) -> Self {
        Self {
            sequence_number: None,
            output_index,
            item,
        }
    }

    /// Sets the sequence number of the event.
    pub fn with_sequence_number(mut self, sequence_number: i32) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }
}

impl ContentPartEvent {
    /// Creates a [`ContentPartEvent`] for the content part at the given indices.
    pub fn new(
        output_index: usize,
        content_index: usize,
        item_id: impl Into<String>,
        part: OutputMessageContent,
    ) -> Self {
        Self {
            sequence_number: None,
            output_index,
            content_index,
            item_id: item_id.into(),
            part,
        }
    }

    /// Sets the sequence number of the event.
    pub fn with_sequence_number(mut self, sequence_number: i32) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }
}

impl OutputTextDeltaEvent {
    /// Creates an [`OutputTextDeltaEvent`] carrying the given text delta.
    pub fn new(
        output_index: usize,
        content_index: usize,
        item_id: impl Into<String>,
        delta: impl Into<String>,
    ) -> Self {
        Self {
            sequence_number: None,
            output_index,
            content_index,
            item_id: item_id.into(),
            delta: delta.into(),
            logprobs: None,
        }
    }

    /// Sets the sequence number of the event.
    pub fn with_sequence_number(mut self, sequence_number: i32) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }

    /// Sets the log probabilities of the text delta.
    pub fn with_logprobs(mut self, logprobs: Vec<TokenLogProb>) -> Self {
        self.logprobs = Some(logprobs);
        self
    }
}

impl OutputTextDoneEvent {
    /// Creates an [`OutputTextDoneEvent`] carrying the given final text.
    pub fn new(
        output_index: usize,
        content_index: usize,
        item_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            sequence_number: None,
            output_index,
            content_index,
            item_id: item_id.into(),
            text: text.into(),
        }
    }

    /// Sets the sequence number of the event.
    pub fn with_sequence_number(mut self, sequence_number: i32) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }
}
