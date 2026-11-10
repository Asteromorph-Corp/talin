use std::collections::HashSet;
use std::ops::Range;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{CompactionModel, RequestParts};
use crate::llm::{Llm, LlmError, LlmRequestItem, LlmRequestSystem};
use crate::schemas::BodyMessage;
use crate::transcript::{AgentTranscript, Entry};

/// Chooses the last entry of the prefix to summarize, identified by its UUID.
#[async_trait]
pub trait CompactionSelector<L: Llm>: Send + Sync {
    /// `None` skips compaction. A cutoff must leave every client tool exchange intact.
    async fn select(
        &mut self,
        llm: &L,
        transcript: &AgentTranscript<L::System, L::Item>,
        parts: &RequestParts<L>,
    ) -> Result<Option<Uuid>, LlmError>;
}

/// Compacts through one selected UUID, at most once while that entry remains compacted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionCutoff(
    /// The last message to summarize.
    pub Uuid,
);

#[async_trait]
impl<L: Llm> CompactionSelector<L> for CompactionCutoff {
    async fn select(
        &mut self,
        _llm: &L,
        _transcript: &AgentTranscript<L::System, L::Item>,
        _parts: &RequestParts<L>,
    ) -> Result<Option<Uuid>, LlmError> {
        Ok(Some(self.0))
    }
}

/// Selects a prefix using provider token counts, retaining recent native records verbatim.
///
/// Requires a token-count endpoint: Anthropic Messages and OpenAI Responses. OpenRouter and xAI
/// have no request token-count endpoint (tested 2026-10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// Compact when the active history reaches this many input tokens.
    pub threshold: u32,
    /// Target token budget for the retained suffix, including system and tool definitions.
    /// At least the newest complete exchange is retained, even when it exceeds this budget.
    pub retain_tokens: u32,
}

impl TokenBudget {
    /// Creates a token-count trigger and a retained-context budget.
    pub fn new(threshold: u32, retain_tokens: u32) -> Self {
        Self {
            threshold,
            retain_tokens,
        }
    }
}

#[async_trait]
impl<L: CompactionModel> CompactionSelector<L> for TokenBudget {
    async fn select(
        &mut self,
        llm: &L,
        transcript: &AgentTranscript<L::System, L::Item>,
        parts: &RequestParts<L>,
    ) -> Result<Option<Uuid>, LlmError> {
        let mut candidates = cutoffs(transcript);
        candidates.retain(|(_, end)| {
            transcript.entries()[*end..]
                .iter()
                .any(|entry| matches!(entry, Entry::Message(..) | Entry::ToolError(..)))
        });
        if candidates.is_empty() {
            return Ok(None);
        }
        let request = parts.build(llm, llm.materialize(transcript.clone()));
        if llm.count_tokens(&request).await? < self.threshold {
            return Ok(None);
        }
        let mut selected = None;
        for (id, end) in candidates.into_iter().rev() {
            let context = select_context(transcript.clone(), end..transcript.entries().len());
            let tokens = llm.count_tokens(&parts.build(llm, context)).await?;
            if selected.is_some() && tokens > self.retain_tokens {
                break;
            }
            selected = Some(id);
            if tokens > self.retain_tokens {
                break;
            }
        }
        Ok(selected)
    }
}

/// Selects a prefix by message count. Native response records and tool exchanges remain whole.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCount {
    /// Compact when at least this many canonical messages are active.
    pub threshold: usize,
    /// Retain at least this many recent canonical messages.
    pub retain_messages: usize,
}

#[async_trait]
impl<L: Llm> CompactionSelector<L> for MessageCount {
    async fn select(
        &mut self,
        _llm: &L,
        transcript: &AgentTranscript<L::System, L::Item>,
        _parts: &RequestParts<L>,
    ) -> Result<Option<Uuid>, LlmError> {
        let entries = transcript.entries();
        let active = compaction_start(transcript)..entries.len();
        let count = entries[active.clone()]
            .iter()
            .filter(|entry| matches!(entry, Entry::Message(..) | Entry::ToolError(..)))
            .count();
        if count < self.threshold {
            return Ok(None);
        }
        Ok(cutoffs(transcript).into_iter().rev().find_map(|(id, end)| {
            let retained = entries[end..]
                .iter()
                .filter(|entry| matches!(entry, Entry::Message(..) | Entry::ToolError(..)))
                .count();
            (retained >= self.retain_messages.max(1)).then_some(id)
        }))
    }
}

fn cutoffs<S: LlmRequestSystem, B: LlmRequestItem>(
    transcript: &AgentTranscript<S, B>,
) -> Vec<(Uuid, usize)> {
    let mut pending = HashSet::new();
    let mut candidates = Vec::new();
    let active = compaction_start(transcript)..transcript.entries().len();
    let entries = transcript.entries();
    for index in active {
        let id = match &entries[index] {
            Entry::Message(id, record) => {
                match record.canonical() {
                    BodyMessage::ToolCall(message) => {
                        pending.extend(message.tool_calls.iter().map(|call| call.id.as_str()));
                    }
                    BodyMessage::Tool(message) => {
                        pending.remove(message.call_id.as_str());
                    }
                    _ => {}
                }
                *id
            }
            Entry::ToolError(id, record) => {
                pending.remove(record.canonical().call_id.as_str());
                *id
            }
            _ => continue,
        };
        if pending.is_empty() {
            candidates.push((id, index + 1));
        }
    }
    candidates
}

pub(super) fn compaction_range<S: LlmRequestSystem, B: LlmRequestItem>(
    transcript: &AgentTranscript<S, B>,
    after: Uuid,
) -> Result<Option<Range<usize>>, LlmError> {
    let start = compaction_start(transcript);
    let Some(index) = transcript.entries().iter().position(|entry| match entry {
        Entry::Message(id, _) | Entry::ToolError(id, _) | Entry::Native(id, _) => *id == after,
        _ => false,
    }) else {
        return Err(LlmError::InvalidRequest(format!(
            "unknown compaction cutoff {after}"
        )));
    };
    if index < start {
        return Ok(None);
    }
    if !cutoffs(transcript).iter().any(|(id, _)| *id == after) {
        return Err(LlmError::InvalidRequest(
            "compaction cannot split a client tool exchange".into(),
        ));
    }
    Ok(Some(start..index + 1))
}

fn compaction_start<S: LlmRequestSystem, B: LlmRequestItem>(
    transcript: &AgentTranscript<S, B>,
) -> usize {
    transcript
        .entries()
        .iter()
        .rposition(|entry| matches!(entry, Entry::CompactionMarker { .. }))
        .map_or(0, |index| index + 1)
}

pub(super) fn select_context<S: LlmRequestSystem, B: LlmRequestItem>(
    transcript: AgentTranscript<S, B>,
    range: Range<usize>,
) -> (S, Vec<B>) {
    let (system, entries) = transcript.into_parts();
    let items = entries
        .into_iter()
        .take(range.end)
        .skip(range.start)
        .filter_map(Entry::into_native)
        .collect();
    (system.into_parts().1, items)
}
