use crate::claude_messages::types::{
    ApiErrorDetails, CompactionBlock, ContentBlock, ResponseMessageContentItemDelta, TextBlock,
    ThinkingBlock, ToolUseBlock,
};
use crate::error::{Error, StreamError};

/// Buffer for assembling a [`ContentBlock`] from its deltas.
pub(super) enum ContentBlockBuffer {
    Text { buf: String, item: TextBlock },
    ToolUse { buf: String, item: ToolUseBlock },
    Thinking { thinking: String, signature: String },
    Compaction { item: CompactionBlock },
    Other { item: ContentBlock },
}

impl ContentBlockBuffer {
    /// Creates a new [`ContentBlockBuffer`] for the given content item.
    pub(super) fn new(item: ContentBlock) -> Self {
        match item {
            ContentBlock::Text(item) => ContentBlockBuffer::Text {
                buf: item.text.clone(),
                item,
            },
            ContentBlock::ToolUse(item) => ContentBlockBuffer::ToolUse {
                buf: String::new(),
                item,
            },
            ContentBlock::Thinking(item) => ContentBlockBuffer::Thinking {
                thinking: item.thinking,
                signature: item.signature,
            },
            ContentBlock::Compaction(item) => ContentBlockBuffer::Compaction { item },
            other => ContentBlockBuffer::Other { item: other },
        }
    }

    /// Applies a delta to the buffer.
    pub(super) fn apply_delta(
        &mut self,
        delta: ResponseMessageContentItemDelta,
    ) -> Result<(), Error<ApiErrorDetails>> {
        match (self, delta) {
            (
                ContentBlockBuffer::Text { buf, .. },
                ResponseMessageContentItemDelta::TextDelta(delta),
            ) => {
                buf.push_str(&delta.text);
            }
            (
                ContentBlockBuffer::ToolUse { buf, .. },
                ResponseMessageContentItemDelta::InputJsonDelta(delta),
            ) => buf.push_str(&delta.partial_json),
            (
                ContentBlockBuffer::Thinking {
                    thinking: thinking_buf,
                    ..
                },
                ResponseMessageContentItemDelta::ThinkingDelta(delta),
            ) => thinking_buf.push_str(&delta.thinking),
            (
                ContentBlockBuffer::Thinking { signature, .. },
                ResponseMessageContentItemDelta::SignatureDelta(delta),
            ) => *signature = delta.signature,
            (
                ContentBlockBuffer::Compaction { item },
                ResponseMessageContentItemDelta::CompactionDelta(delta),
            ) => item.content = delta.content,
            _ => {
                let err = StreamError::invalid_event("invalid \"content_block_delta\" event");
                return Err(err.into());
            }
        }

        Ok(())
    }

    /// Finishes assembling the content item and returns it.
    pub(super) fn finish(self) -> Result<ContentBlock, Error<ApiErrorDetails>> {
        let item = match self {
            ContentBlockBuffer::Text { buf: text, item } => {
                ContentBlock::Text(TextBlock { text, ..item })
            }
            ContentBlockBuffer::ToolUse { buf, item } => {
                // Empty-input tools may emit only an empty JSON delta. In that case the
                // content_block_start already carries the complete input (usually {}).
                let input = if buf.is_empty() {
                    item.input
                } else {
                    serde_json::from_str(&buf).map_err(|e| Error::JSONDeserialize(e, buf))?
                };
                ContentBlock::ToolUse(ToolUseBlock { input, ..item })
            }
            ContentBlockBuffer::Thinking {
                thinking,
                signature,
            } => ContentBlock::Thinking(ThinkingBlock {
                thinking,
                signature,
            }),
            ContentBlockBuffer::Compaction { item } => ContentBlock::Compaction(item),
            ContentBlockBuffer::Other { item } => item,
        };
        Ok(item)
    }
}
