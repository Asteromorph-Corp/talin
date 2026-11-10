use async_trait::async_trait;

use crate::chain::OutputCtor;
use crate::llm::Llm;
use crate::strategy::Strategy;
use crate::transcript::edit::{EntryEditScheme, SystemEditScheme};
use crate::transcript::{AgentTranscript, ApplyEditError};

macro_rules! apply_edit_strategy {
    ($(#[$meta:meta])* $name:ident, $bound:ident, $method:ident, $apply:ident, $error:ty) => {
        $(#[$meta])*
        pub struct $name<E: $bound>(E);

        impl<E: $bound> $name<E> {
            /// Creates the strategy from the given edit scheme.
            pub fn new(edit_scheme: E) -> Self {
                Self(edit_scheme)
            }
        }

        #[async_trait]
        impl<L: Llm, O: OutputCtor, E: $bound> Strategy<L, O> for $name<E>
        where
            E::Error: 'static,
        {
            type Output = ();
            type Error = $error;

            async fn $method(
                &mut self,
                transcript: &mut AgentTranscript<L::System, L::Item>,
            ) -> Result<(), Self::Error> {
                transcript.$apply(&self.0).await?;
                Ok(())
            }

            async fn finalize(self) -> Result<Self::Output, Self::Error> {
                Ok(())
            }
        }
    };
    ($(#[$meta:meta])* dual $name:ident, $method:ident) => {
        $(#[$meta])*
        pub struct $name<E>(E)
        where
            E: EntryEditScheme,
            E: SystemEditScheme<Error = <E as EntryEditScheme>::Error>;

        impl<E> $name<E>
        where
            E: EntryEditScheme,
            E: SystemEditScheme<Error = <E as EntryEditScheme>::Error>,
        {
            /// Creates the strategy from the given edit scheme.
            pub fn new(edit_scheme: E) -> Self {
                Self(edit_scheme)
            }
        }

        #[async_trait]
        impl<L: Llm, O: OutputCtor, E, Err> Strategy<L, O> for $name<E>
        where
            E: EntryEditScheme<Error = Err> + SystemEditScheme<Error = Err>,
            Err: std::error::Error + Send + Sync + 'static,
        {
            type Output = ();
            type Error = ApplyEditError<Err>;

            async fn $method(
                &mut self,
                transcript: &mut AgentTranscript<L::System, L::Item>,
            ) -> Result<(), Self::Error> {
                transcript.apply_edits(&self.0).await?;
                Ok(())
            }

            async fn finalize(self) -> Result<Self::Output, Self::Error> {
                Ok(())
            }
        }
    };
}

apply_edit_strategy!(
    /// A [`Strategy`] that **permanently** applies an [`EntryEditScheme`] to the transcript's
    /// entries.
    ///
    /// The edit scheme is applied via [`edit_transcript`](Strategy::edit_transcript), so the edits
    /// are preserved across steps. Use [`ApplyEntryProjectionStrategy`] instead if you want
    /// the ephemeral edits.
    ///
    /// No return value.
    ApplyEntryEditStrategy,
    EntryEditScheme,
    edit_transcript,
    apply_entry_edit,
    ApplyEditError<E::Error>
);
apply_edit_strategy!(
    /// A [`Strategy`] that **ephemerally** applies an [`EntryEditScheme`] to the transcript's
    /// entries.
    ///
    /// The edit scheme is applied via [`project_transcript`](Strategy::project_transcript), so the
    /// edits are not preserved across steps. Use [`ApplyEntryEditStrategy`] instead if you
    /// want the permanent edits.
    ///
    /// No return value.
    ApplyEntryProjectionStrategy,
    EntryEditScheme,
    project_transcript,
    apply_entry_edit,
    ApplyEditError<E::Error>
);
apply_edit_strategy!(
    /// A [`Strategy`] that **permanently** applies a [`SystemEditScheme`] to the transcript's
    /// system prompt.
    ///
    /// The edit scheme is applied via [`edit_transcript`](Strategy::edit_transcript), so the edits
    /// are preserved across steps. Use [`ApplySystemProjectionStrategy`] instead if you want
    /// the ephemeral edits.
    ///
    /// No return value.
    ApplySystemEditStrategy,
    SystemEditScheme,
    edit_transcript,
    apply_system_edit,
    E::Error
);
apply_edit_strategy!(
    /// A [`Strategy`] that **ephemerally** applies a [`SystemEditScheme`] to the transcript's
    /// system prompt.
    ///
    /// The edit scheme is applied via [`project_transcript`](Strategy::project_transcript), so the
    /// edits are not preserved across steps. Use [`ApplySystemEditStrategy`] instead if you
    /// want the permanent edits.
    ///
    /// No return value.
    ApplySystemProjectionStrategy,
    SystemEditScheme,
    project_transcript,
    apply_system_edit,
    E::Error
);
apply_edit_strategy!(
    /// A [`Strategy`] that **permanently** applies a hybrid edit scheme (implementing both [`EntryEditScheme`] and [`SystemEditScheme`]) to the transcript.
    ///
    /// The edit scheme is applied via [`edit_transcript`](Strategy::edit_transcript), so the edits are preserved across steps.
    /// Use [`ApplyProjectionsStrategy`] instead if you want the ephemeral edits.
    ///
    /// No return value.
    dual ApplyEditsStrategy, edit_transcript
);
apply_edit_strategy!(
    /// A [`Strategy`] that **ephemerally** applies a hybrid edit scheme (implementing both [`EntryEditScheme`] and [`SystemEditScheme`]) to the transcript.
    ///
    /// The edit scheme is applied via [`project_transcript`](Strategy::project_transcript), so the edits are not preserved across steps.
    /// Use [`ApplyEditsStrategy`] instead if you want the permanent edits.
    ///
    /// No return value.
    dual ApplyProjectionsStrategy, project_transcript
);
