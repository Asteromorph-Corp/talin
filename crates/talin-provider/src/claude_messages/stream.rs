use futures::StreamExt;

use crate::claude_messages::stream::helper::{
    assemble_content_block, check_content_block_index, get_initial_message,
};
use crate::claude_messages::types::{
    ApiErrorDetails, CacheCreation, MessageDeltaUsage, MessagesResponse, OutputTokensDetails,
    RawMessageDeltaEvent, RawMessageStreamEvent, ServerToolUsage, Usage,
};
use crate::error::StreamError;
use crate::stream::{Merge, PartialResponse, PreferNew, ResponseStream, or_partial};

mod buffer;
mod helper;

/// Assembles a complete [`MessagesResponse`] from a stream of [`RawMessageStreamEvent`]s.
#[allow(clippy::result_large_err)]
pub(super) async fn assemble(
    mut stream: ResponseStream<RawMessageStreamEvent, ApiErrorDetails>,
) -> Result<MessagesResponse, PartialResponse<MessagesResponse, ApiErrorDetails>> {
    let mut message = or_partial!(get_initial_message(&mut stream).await);

    while let Some(evt) = stream.next().await {
        let evt = or_partial!(evt, message);

        match evt {
            // Initial message is already received, and should not be received again.
            RawMessageStreamEvent::MessageStart(_) => {
                let err = StreamError::invalid_event("`\"message_start\" received multiple times.");
                return Err(PartialResponse::new(Some(message), err));
            }
            RawMessageStreamEvent::MessageDelta(delta) => message.merge(delta),
            RawMessageStreamEvent::MessageStop => break,

            RawMessageStreamEvent::ContentBlockStart(cb_start) => {
                let expected = message.content.len() as u32;
                or_partial!(
                    check_content_block_index("content_block_start", cb_start.index, expected),
                    message
                );
                let content_item = or_partial!(
                    assemble_content_block(&mut stream, cb_start.index, cb_start).await,
                    message
                );
                message.content.push(content_item)
            }
            RawMessageStreamEvent::ContentBlockDelta(_) => {
                let err = StreamError::invalid_event("orphan \"content_block_delta\"");
                return Err(PartialResponse::new(Some(message), err));
            }
            RawMessageStreamEvent::ContentBlockStop(_) => {
                let err = StreamError::invalid_event("orphan \"content_block_stop\"");
                return Err(PartialResponse::new(Some(message), err));
            }

            RawMessageStreamEvent::Unknown(other) => {
                tracing::warn!("Ignored unknown event: {other:?}")
            }
        }
    }

    Ok(message)
}

/// Merges a [`RawMessageDeltaEvent`] into a [`MessagesResponse`].
impl Merge<RawMessageDeltaEvent> for MessagesResponse {
    fn merge(&mut self, rhs: RawMessageDeltaEvent) {
        self.stop_reason.merge(PreferNew(rhs.delta.stop_reason));
        self.stop_sequence.merge(PreferNew(rhs.delta.stop_sequence));
        self.usage.merge(rhs.usage);
    }
}

impl Merge<MessageDeltaUsage> for Usage {
    fn merge(&mut self, rhs: MessageDeltaUsage) {
        // Message delta usage counts are cumulative totals, not deltas; thus we use `.or`.
        // <https://platform.claude.com/docs/en/build-with-claude/streaming#event-types>
        self.cache_creation_input_tokens = rhs
            .cache_creation_input_tokens
            .or(self.cache_creation_input_tokens);
        self.cache_read_input_tokens = rhs.cache_read_input_tokens.or(self.cache_read_input_tokens);
        self.input_tokens = rhs.input_tokens.unwrap_or(self.input_tokens);
        self.output_tokens = rhs.output_tokens;
        self.iterations = rhs.iterations.or(self.iterations.take());
        self.output_tokens_details = rhs
            .output_tokens_details
            .or(self.output_tokens_details.take());
        self.server_tool_use = rhs.server_tool_use.or(self.server_tool_use.take());
    }
}

impl Merge<Self> for MessageDeltaUsage {
    fn merge(&mut self, rhs: Self) {
        self.cache_creation_input_tokens = rhs
            .cache_creation_input_tokens
            .or(self.cache_creation_input_tokens);
        self.cache_read_input_tokens = rhs.cache_read_input_tokens.or(self.cache_read_input_tokens);
        self.input_tokens = rhs.input_tokens.or(self.input_tokens);
        self.output_tokens = rhs.output_tokens;
        self.iterations = rhs.iterations.or(self.iterations.take());
        self.output_tokens_details = rhs
            .output_tokens_details
            .or(self.output_tokens_details.take());
        self.server_tool_use = rhs.server_tool_use.or(self.server_tool_use.take());
    }
}

/// Merges a [`Usage`] into another [`Usage`].
impl Merge<Self> for Usage {
    fn merge(&mut self, rhs: Self) {
        self.cache_creation.merge(rhs.cache_creation);
        self.cache_creation_input_tokens
            .merge(rhs.cache_creation_input_tokens);
        self.cache_read_input_tokens
            .merge(rhs.cache_read_input_tokens);
        self.input_tokens += rhs.input_tokens;
        self.output_tokens += rhs.output_tokens;
        if let Some(iterations) = rhs.iterations {
            self.iterations
                .get_or_insert_with(Vec::new)
                .extend(iterations);
        }
        self.output_tokens_details.merge(rhs.output_tokens_details);
        self.server_tool_use.merge(rhs.server_tool_use);
        if self.service_tier.is_none() {
            self.service_tier = rhs.service_tier;
        }
    }
}

/// Merges an [`CacheCreation`] into another [`CacheCreation`].
impl Merge<Self> for CacheCreation {
    fn merge(&mut self, rhs: Self) {
        self.ephemeral_1h_input_tokens += rhs.ephemeral_1h_input_tokens;
        self.ephemeral_5m_input_tokens += rhs.ephemeral_5m_input_tokens;
    }
}

impl Merge<Self> for OutputTokensDetails {
    fn merge(&mut self, rhs: Self) {
        self.thinking_tokens += rhs.thinking_tokens;
    }
}

/// Merges a [`ServerToolUsage`] into another [`ServerToolUsage`].
impl Merge<Self> for ServerToolUsage {
    fn merge(&mut self, rhs: Self) {
        self.web_search_requests += rhs.web_search_requests;
    }
}

#[cfg(test)]
mod tests {
    use futures::stream;
    use serde_json::json;

    use super::*;
    use crate::claude_messages::types::{ContentBlock, ContentBlockParam, StopReason};

    #[tokio::test]
    async fn assembles_live_empty_input_tool_stream() {
        // Captured from a live API probe of the archive tool: the sole input delta is "".
        let events = include_str!("stream/fixtures/empty-tool-input.sse")
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .map(|data| {
                Ok(serde_json::from_str::<RawMessageStreamEvent>(data)
                    .expect("valid captured event"))
            });
        let response = assemble(Box::pin(stream::iter(events)))
            .await
            .unwrap_or_else(|partial| panic!("stream assembly failed: {}", partial.error));
        assert!(matches!(response.stop_reason, Some(StopReason::ToolUse)));
        let [ContentBlock::ToolUse(tool)] = response.content.as_slice() else {
            panic!("expected the streamed tool call to survive");
        };
        assert_eq!(tool.name, "LoadArchive");
        assert_eq!(tool.input, json!({}));
    }

    #[tokio::test]
    async fn assembles_recorded_compaction_content_and_usage() {
        use crate::claude_messages::types::ResponseMessageContentItemDelta;

        let events: Vec<RawMessageStreamEvent> = include_str!("stream/fixtures/compaction.sse")
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .map(|data| serde_json::from_str(data).expect("valid captured event"))
            .collect();
        let initial = events
            .iter()
            .find_map(|event| match event {
                RawMessageStreamEvent::ContentBlockStart(start) => match &start.content_block {
                    ContentBlock::Compaction(block) => Some(block.clone()),
                    _ => None,
                },
                _ => None,
            })
            .expect("the live stream contains a compaction start");
        let summary = events
            .iter()
            .rev()
            .find_map(|event| match event {
                RawMessageStreamEvent::ContentBlockDelta(delta) => match &delta.delta {
                    ResponseMessageContentItemDelta::CompactionDelta(compaction) => {
                        Some(compaction.content.clone())
                    }
                    _ => None,
                },
                _ => None,
            })
            .expect("the live stream contains a compaction delta");
        let final_delta = events
            .iter()
            .rev()
            .find_map(|event| match event {
                RawMessageStreamEvent::MessageDelta(delta) => Some(delta.clone()),
                _ => None,
            })
            .expect("the live stream contains final usage");

        let stream = events.into_iter().map(Ok);
        let response = assemble(Box::pin(stream::iter(stream)))
            .await
            .unwrap_or_else(|partial| panic!("stream assembly failed: {}", partial.error));
        let [ContentBlock::Compaction(actual)] = response.content.as_slice() else {
            panic!("the captured compaction was lost during assembly");
        };
        let mut expected = initial;
        expected.content = summary;
        let expected: ContentBlockParam = ContentBlock::Compaction(expected).into();
        let actual: ContentBlockParam = ContentBlock::Compaction(actual.clone()).into();
        assert_eq!(
            serde_json::to_string(&actual).unwrap(),
            serde_json::to_string(&expected).unwrap(),
            "stream assembly changed the native summary or its opaque fields"
        );
        assert_eq!(response.stop_reason, final_delta.delta.stop_reason);
        assert_eq!(
            response.usage.output_tokens,
            final_delta.usage.output_tokens
        );
        assert_eq!(
            Some(response.usage.input_tokens),
            final_delta.usage.input_tokens
        );
        assert_eq!(
            serde_json::to_string(&response.usage.iterations).unwrap(),
            serde_json::to_string(&final_delta.usage.iterations).unwrap(),
            "stream assembly lost the provider's per-iteration compaction usage"
        );
    }
}
