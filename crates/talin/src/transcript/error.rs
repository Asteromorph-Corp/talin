//! Errors from rewriting the transcript.

use std::convert::Infallible;

use thiserror::Error;
use uuid::Uuid;

use crate::llm::UnsupportedContent;

/// An error from [`AgentTranscript::modify`](super::AgentTranscript::modify).
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ModifyError {
    /// An edit referenced an entry that is not in the range being modified.
    #[error("edit references entry {0}, which is not in the range being modified")]
    UnknownEntry(Uuid),

    /// An edit produced content the provider cannot represent.
    #[error(transparent)]
    UnsupportedContent(#[from] UnsupportedContent),
}

/// An error from applying an [`EntryEditScheme`](super::edit::EntryEditScheme).
///
/// A scheme's job ends at producing provider-agnostic [`EntryEdit`](super::EntryEdit)s; whether the
/// transcript accepts them is decided when they are applied. The two failures belong to different
/// layers, so this keeps them separate rather than requiring every
/// [`EntryEditScheme::Error`](super::edit::EntryEditScheme::Error) to absorb a [`ModifyError`].
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ApplyEditError<E> {
    /// The scheme failed to produce its edits.
    #[error(transparent)]
    Scheme(E),

    /// The transcript rejected the edits the scheme produced.
    #[error(transparent)]
    Modify(#[from] ModifyError),
}

impl ApplyEditError<Infallible> {
    /// Drops the uninhabited [`Scheme`](ApplyEditError::Scheme) arm, leaving the failure that can
    /// actually occur.
    ///
    /// A scheme that cannot fail (e.g. [`RemoveToolCalls`](super::edit::RemoveToolCalls)) still
    /// surfaces an [`ApplyEditError`], because applying its edits can be rejected even when
    /// producing them cannot.
    pub fn drop_infallible(self) -> ModifyError {
        match self {
            ApplyEditError::Scheme(never) => match never {},
            ApplyEditError::Modify(error) => error,
        }
    }
}
