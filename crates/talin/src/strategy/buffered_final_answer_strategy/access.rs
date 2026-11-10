use std::convert::Infallible;
use std::sync::Arc;

use async_trait::async_trait;

use crate::llm::{LlmRequestItem, LlmRequestSystem};
use crate::strategy::buffered_final_answer_strategy::FinalAnswerBuffer;
use crate::strategy::buffered_final_answer_strategy::edit::{EDIT_FINAL_ANSWER, FinalAnswerEdit};
use crate::tools::{EmptyFunctionInput, Function, Tool};
use crate::transcript::edit::{RemoveToolCalls, SystemEditScheme};
use crate::transcript::{AgentTranscript, ApplyEditError, ModifyError};

/// How the model reads the [accumulated answer](super::BufferedFinalAnswerStrategy).
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FinalAnswerAccess {
    /// Inject the current buffer as a block at the end of the system prompt every step, so the
    /// current answer is always in view.
    System,
    /// Expose a `read_final_answer` tool the model calls to read the current buffer.
    ReadTool,
}

/// A [`FinalAnswerAccess`] mode bound to the buffer it reads.
///
/// Owns everything that decides how the model sees the answer: the read tool, the transcript
/// pruning, the system-prompt injection.
///
/// [`BufferedFinalAnswerStrategy`]: super::BufferedFinalAnswerStrategy
pub(super) struct AccessMode<B> {
    access: FinalAnswerAccess,
    content: Arc<B>,
    edit: FinalAnswerEdit,
}

impl<B: FinalAnswerBuffer + 'static> AccessMode<B> {
    pub(super) fn new(access: FinalAnswerAccess, content: Arc<B>, edit: FinalAnswerEdit) -> Self {
        Self {
            access,
            content,
            edit,
        }
    }

    /// Builds this mode's `read_final_answer` tool, or returns `None` if the mode is
    /// [`System`](FinalAnswerAccess::System).
    pub(super) fn tool(&self) -> Option<Tool> {
        match self.access {
            FinalAnswerAccess::System => None,
            FinalAnswerAccess::ReadTool => Some(Tool::from(ReadAccess {
                content: Arc::clone(&self.content),
                edit: self.edit,
            })),
        }
    }

    /// Prunes the tool calls this mode makes redundant.
    pub(super) async fn edit_transcript<S: LlmRequestSystem, I: LlmRequestItem>(
        &self,
        transcript: &mut AgentTranscript<S, I>,
    ) -> Result<(), ModifyError> {
        // The schemes themselves are `Infallible`; only applying their edits can be rejected.
        match self.access {
            // Stale reads only bloat the context; keep just the most recent one.
            FinalAnswerAccess::ReadTool => transcript
                .apply_entry_edit(&RemoveToolCalls::matching(READ_FINAL_ANSWER).except_last())
                .await
                .map_err(ApplyEditError::drop_infallible)?,
            FinalAnswerAccess::System => transcript
                .apply_entry_edit(&RemoveToolCalls::matching(EDIT_FINAL_ANSWER))
                .await
                .map_err(ApplyEditError::drop_infallible)?,
        }
        Ok(())
    }

    /// Injects the current buffer into the system prompt, for modes that need it.
    pub(super) async fn project_transcript<S: LlmRequestSystem, I: LlmRequestItem>(
        &self,
        transcript: &mut AgentTranscript<S, I>,
    ) -> Result<(), B::Error> {
        if self.access == FinalAnswerAccess::System {
            let content = self.content.read().await?;
            transcript
                .apply_system_edit(&InjectAnswer(&content, self.edit))
                .await
                .unwrap_or_else(|e| match e {});
        }
        Ok(())
    }
}

/// A tool that returns the current accumulated answer, rendered for the active edit mode.
struct ReadAccess<B> {
    content: Arc<B>,
    edit: FinalAnswerEdit,
}

const READ_FINAL_ANSWER: &str = "read_final_answer";

#[async_trait]
impl<B: FinalAnswerBuffer + 'static> Function for ReadAccess<B> {
    type Input = EmptyFunctionInput;
    type Output = String;

    fn name(&self) -> String {
        READ_FINAL_ANSWER.to_owned()
    }

    fn description(&self) -> String {
        match self.edit {
            FinalAnswerEdit::Hashline => {
                "Read the answer accumulated so far. Each line is prefixed \
                 with a `linenum:hash` anchor for use with the edit tool."
                    .to_owned()
            }
            _ => "Read the answer accumulated so far.".to_owned(),
        }
    }

    async fn call(
        &self,
        _input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
        let content = self.edit.render_buffer(&self.content.read().await?);
        Ok(content)
    }
}

/// A [`SystemEditScheme`] that appends the accumulated answer as a block at the end of the system
/// prompt.
struct InjectAnswer<'a>(&'a str, FinalAnswerEdit);

const ANSWER_HEADER: &str = "# Accumulated final answer so far";

#[async_trait]
impl SystemEditScheme for InjectAnswer<'_> {
    type Error = Infallible;

    async fn edit_system(&self, contents: &[String]) -> Result<Option<Vec<String>>, Self::Error> {
        let mut contents = contents.to_vec();
        let rendered = self.1.render_buffer(self.0);
        contents.push(format!("{ANSWER_HEADER}\n\n{rendered}",));
        Ok(Some(contents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn injects_answer_into_system_prompt() {
        let scheme = InjectAnswer("draft answer", FinalAnswerEdit::Replace);

        let out = scheme
            .edit_system(&["base prompt".to_owned()])
            .await
            .unwrap()
            .unwrap();

        assert_eq!(out[0], "base prompt");
        assert!(out[1].contains("draft answer"));
    }
}
