//! Transcript edit schemes.
//!
//! An edit scheme rewrites a transcript in one of two independent regions, each its own trait:
//! - [`EntryEditScheme`] rewrites the transcript entries (and chooses their
//!   [scope](EntryEditScheme::scope)).
//! - [`SystemEditScheme`] rewrites the system prompt contents.
//!
//! Entry schemes are applied with
//! [`AgentTranscript::apply_entry_edit`](super::AgentTranscript::apply_entry_edit) and system
//! schemes with [`AgentTranscript::apply_system_edit`](super::AgentTranscript::apply_system_edit).
//! A scheme that spans both can be applied at once with
//! [`AgentTranscript::apply_edits`](super::AgentTranscript::apply_edits).
//!
//! The module also exposes concrete schemes:
//! - [`Interpolate`] replaces placeholders in the transcript with their resolved values.
//! - [`RemoveToolCalls`] removes the tool calls selected by a [`ToolCallSelector`], each with its
//!   paired result or error. Build one with [`RemoveToolCalls::matching`] to select by filter, or
//!   [`RemoveToolCalls::errored`] to select the calls a tool error resolved.

use async_trait::async_trait;

use crate::transcript::{EntryEdit, EntryView, Scope};

mod interpolate;
mod remove_tool_calls;

pub use interpolate::{Interpolate, PlaceholderResolver};
pub(crate) use remove_tool_calls::remove_errored_calls;
pub use remove_tool_calls::{Errored, Matching, RemoveToolCalls, ToolCallFilter, ToolCallSelector};

/// Specification for rewriting the transcript entries.
///
/// Applied by [`AgentTranscript::apply_entry_edit`](super::AgentTranscript::apply_entry_edit).
#[async_trait]
pub trait EntryEditScheme: Send + Sync {
    /// The error returned when computing the edits fails.
    type Error: std::error::Error + Send + Sync;

    /// The scope that [`edit`](EntryEditScheme::edit) targets.
    fn scope(&self) -> Scope {
        Scope::full()
    }

    /// Maps the ordered entry views to the full ordered edit sequence, in the vocabulary of
    /// [`EntryEdit`]. The default implementation leaves the transcript unchanged.
    ///
    /// Refer to [`AgentTranscript::modify`](super::AgentTranscript::modify) for the semantics of
    /// the edit sequence.
    async fn edit(&self, views: Vec<EntryView<'_>>) -> Result<Vec<EntryEdit>, Self::Error> {
        Ok(views.into_iter().map(EntryEdit::from).collect())
    }
}

/// Specification for rewriting the system prompt contents.
///
///
/// Applied by [`AgentTranscript::apply_system_edit`](super::AgentTranscript::apply_system_edit).
#[async_trait]
pub trait SystemEditScheme: Send + Sync {
    /// The error returned when rewriting the system prompt fails.
    type Error: std::error::Error + Send + Sync;

    /// Rewrites the system prompt contents. `Ok(None)` leaves them unchanged.
    async fn edit_system(&self, contents: &[String]) -> Result<Option<Vec<String>>, Self::Error>;
}

#[async_trait]
impl<F, E> EntryEditScheme for F
where
    F: Fn(Vec<EntryView<'_>>) -> Result<Vec<EntryEdit>, E> + Send + Sync,
    E: std::error::Error + Send + Sync,
{
    type Error = E;

    async fn edit(&self, views: Vec<EntryView<'_>>) -> Result<Vec<EntryEdit>, Self::Error> {
        self(views)
    }
}
