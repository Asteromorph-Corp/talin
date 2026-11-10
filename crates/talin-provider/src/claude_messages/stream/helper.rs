use futures::StreamExt;

use crate::claude_messages::stream::buffer::ContentBlockBuffer;
use crate::claude_messages::types::{
    ApiErrorDetails, ContentBlock, MessagesResponse, RawContentBlockStartEvent,
    RawMessageStreamEvent,
};
use crate::error::{Error, StreamError};
use crate::stream::ResponseStream;

/// Receives events from the stream until a `"message_start"` event is received.
///
/// # Errors
/// Returns an error if:
/// - The stream yields an error
/// - An event other than `"message_start"` or `"ping"` appears before `"message_start"`
/// - The stream closes before a `"message_start"` event is received.
pub(super) async fn get_initial_message(
    stream: &mut ResponseStream<RawMessageStreamEvent, ApiErrorDetails>,
) -> Result<MessagesResponse, Error<ApiErrorDetails>> {
    while let Some(evt) = stream.next().await {
        let evt = evt?;

        match evt {
            RawMessageStreamEvent::MessageStart(message_start) => return Ok(message_start.message),
            RawMessageStreamEvent::Unknown(other) => {
                tracing::warn!("Ignored unknown event: {other:?}");
                continue;
            }

            other => {
                return Err(StreamError::InvalidEvent(format!(
                    "received \"{}\" event before \"message_start\" event",
                    other.event_type()
                ))
                .into());
            }
        };
    }

    Err(StreamError::UnexpectedClose.into())
}

/// Assembles a complete [`ContentBlock`] from the stream, starting from
/// a [`RawContentBlockStartEvent`].
pub(super) async fn assemble_content_block(
    stream: &mut ResponseStream<RawMessageStreamEvent, ApiErrorDetails>,
    index: u32,
    cb_start: RawContentBlockStartEvent,
) -> Result<ContentBlock, Error<ApiErrorDetails>> {
    let mut buf = ContentBlockBuffer::new(cb_start.content_block);
    while let Some(evt) = stream.next().await {
        let evt = evt?;
        match evt {
            RawMessageStreamEvent::ContentBlockDelta(delta) => {
                check_content_block_index("content_block_delta", delta.index, index)?;
                buf.apply_delta(delta.delta)?;
            }
            RawMessageStreamEvent::ContentBlockStop(stop) => {
                check_content_block_index("content_block_stop", stop.index, index)?;
                return buf.finish();
            }
            RawMessageStreamEvent::Unknown(other) => {
                tracing::warn!("Ignored unknown event: {other:?}");
                continue;
            }

            other => {
                let err = StreamError::invalid_event(format!(
                    "received \"{}\" inside an open content block before \"content_block_stop\"",
                    other.event_type()
                ));
                return Err(err.into());
            }
        }
    }

    let err = StreamError::invalid_event("stream ended before \"content_block_stop\" event");
    Err(err.into())
}

/// Checks that the given index matches the expected index for the event type.
pub(super) fn check_content_block_index(
    event_type: &str,
    index: u32,
    expected: u32,
) -> Result<(), StreamError> {
    if index != expected {
        return Err(StreamError::invalid_event(format!(
            "invalid \"{event_type}\" index (expected {expected}, received {index})"
        )));
    }
    Ok(())
}
