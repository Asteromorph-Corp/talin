use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::selection::select_context;
use super::{CompactionSelector, CompactionStrategy, RequestParts, compaction_range};
use crate::llm::{Llm, LlmError};
use crate::schemas::WithUsage;
use crate::transcript::AgentTranscript;

/// Summarizes the selected history without producing an agent action.
#[async_trait]
pub trait Compactor<L: Llm>: Send + Sync {
    /// Returns native content to replace the selected prefix in outgoing requests.
    /// `context` is the lowered prefix; build the summarization request from it with `parts`.
    /// `None` leaves history unchanged; usage is recorded in either case.
    async fn compact(
        &mut self,
        llm: &L,
        context: (L::System, Vec<L::Item>),
        parts: &RequestParts<L>,
    ) -> Result<WithUsage<Option<L::Item>>, LlmError>;
}

/// Compacts selected history before generation, preserving original entries and UUIDs.
///
/// Works with every LLM. Endpoint support then depends on the parts: an application-defined
/// [`Compactor`] with [`MessageCount`](super::MessageCount) or
/// [`CompactionCutoff`](super::CompactionCutoff) worked on every endpoint tested (2026-10),
/// including OpenRouter and Chat Completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCompactStrategy<C, S> {
    /// Produces the replacement native content.
    pub compactor: C,
    /// Chooses the prefix to summarize.
    pub selector: S,
}

impl<C, S> CustomCompactStrategy<C, S> {
    /// Creates a strategy with an application-defined compactor and selection policy.
    pub fn new(compactor: C, selector: S) -> Self {
        Self {
            compactor,
            selector,
        }
    }
}

#[async_trait]
impl<L, C, S> CompactionStrategy<L> for CustomCompactStrategy<C, S>
where
    L: Llm,
    C: Compactor<L>,
    S: CompactionSelector<L>,
{
    async fn compact(
        &mut self,
        llm: &L,
        transcript: &mut AgentTranscript<L::System, L::Item>,
        parts: &RequestParts<L>,
    ) -> Result<WithUsage<bool>, LlmError> {
        let Some(after) = self.selector.select(llm, transcript, parts).await? else {
            return Ok(WithUsage::new(false, Default::default()));
        };
        let Some(range) = compaction_range(transcript, after)? else {
            return Ok(WithUsage::new(false, Default::default()));
        };
        let selected = select_context(transcript.clone(), range);
        let result = self.compactor.compact(llm, selected, parts).await?;
        let changed = match result.content {
            Some(native) => transcript.insert_compaction(after, native),
            None => false,
        };
        Ok(WithUsage::new(changed, result.usage))
    }
}
