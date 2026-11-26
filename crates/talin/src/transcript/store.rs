//! Stores that persist a transcript while a session runs.

use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::llm::{LlmRequestItem, LlmRequestSystem};
use crate::transcript::AgentTranscript;

mod file;
pub use file::{FileStore, FileStoreError};

/// A persistent store for the [agent transcript](crate::transcript::AgentTranscript).
///
/// A [runner](crate::agent::AgentRunner) calls [`checkpoint`](Self::checkpoint):
/// - after the turn's messages are recorded
/// - after each model response
/// - after each step completes through [`AgentRunner::step`] or [`PendingStep::complete`].
///
/// If you resolve a step manually with [`PendingStep::complete_with`] or [`PendingStep::discard`],
/// or edit the transcript, the transcript would be persisted at the next checkpoint. You can also
/// call [`AgentSession::checkpoint`] manually to persist the transcript.
///
/// [`AgentRunner::step`]: crate::agent::AgentRunner::step
/// [`PendingStep::complete`]: crate::agent::PendingStep::complete
/// [`PendingStep::complete_with`]: crate::agent::PendingStep::complete_with
/// [`PendingStep::discard`]: crate::agent::PendingStep::discard
/// [`AgentSession::checkpoint`]: crate::agent::AgentSession::checkpoint
#[async_trait]
pub trait TranscriptStore: Send + Sync {
    /// Persists the current state of the transcript, serialized as a JSON document.
    async fn checkpoint_raw(&mut self, json: &str) -> Result<(), StoreError>;

    /// Returns the last checkpointed JSON document, or `None` if nothing is stored yet.
    async fn restore_raw(&mut self) -> Result<Option<String>, StoreError>;
}

/// Typed convenience over [`TranscriptStore`].
///
/// The provider types `S` and `B` are chosen by the caller, so a store can persist and restore a
/// transcript of any provider.
#[async_trait]
pub trait TranscriptStoreExt: TranscriptStore {
    /// Persists the current state of the transcript.
    async fn checkpoint<S, B>(
        &mut self,
        transcript: &AgentTranscript<S, B>,
    ) -> Result<(), StoreError>
    where
        S: LlmRequestSystem + Serialize + Sync,
        B: LlmRequestItem + Serialize + Sync,
    {
        let json = serde_json::to_string(transcript)?;
        self.checkpoint_raw(&json).await
    }

    /// Restores the last checkpointed transcript, or `None` if nothing is stored yet.
    ///
    /// To resume with a different provider than the one that wrote the checkpoint, restore with
    /// the original types and then [`migrate`](AgentTranscript::migrate).
    async fn restore<S, B>(&mut self) -> Result<Option<AgentTranscript<S, B>>, StoreError>
    where
        S: LlmRequestSystem + DeserializeOwned,
        B: LlmRequestItem + DeserializeOwned,
    {
        match self.restore_raw().await? {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }
}

impl<T: TranscriptStore + ?Sized> TranscriptStoreExt for T {}

/// The error a [`TranscriptStore`] fails with.
pub type StoreError = Box<dyn std::error::Error + Send + Sync>;
