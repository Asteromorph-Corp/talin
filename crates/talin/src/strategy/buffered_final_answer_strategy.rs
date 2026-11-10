//! Strategy and helpers for accumulating the final answer in a buffer instead of returning it
//! immediately.

use std::sync::Arc;

use async_trait::async_trait;

use crate::chain::OutputCtor;
use crate::llm::Llm;
use crate::schemas::{AssistantMessage, ContentPart, LlmStep};
use crate::strategy::Strategy;
use crate::strategy::buffered_final_answer_strategy::access::AccessMode;
use crate::strategy::buffered_final_answer_strategy::submit::{
    SUBMIT_FINAL_ANSWER, SubmitFinalAnswer,
};
use crate::tools::{Tool, ToolResolver};
use crate::transcript::{AgentTranscript, ModifyError};

mod access;
mod buffer;
mod edit;
mod hashline;
mod submit;

pub use access::FinalAnswerAccess;
pub use buffer::{FinalAnswerBuffer, InMemoryBuffer};
pub use edit::FinalAnswerEdit;

/// An error from a [`BufferedFinalAnswerStrategy`].
///
/// The strategy both reads/writes the answer buffer and rewrites the transcript, so it can fail in
/// either layer. Kept separate for the same reason as
/// [`ApplyEditError`](crate::transcript::ApplyEditError).
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum BufferedFinalAnswerError<E> {
    /// The answer buffer failed to read or write.
    #[error(transparent)]
    Buffer(E),

    /// The transcript rejected the strategy's edits.
    #[error(transparent)]
    Modify(#[from] ModifyError),
}

/// A [`Strategy`] that lets the model build up a final answer incrementally in a shared buffer.
///
/// The model is given a tool to edit the buffer (and, optionally, to read it back) and a
/// `submit_final_answer` tool. When the model calls `submit_final_answer`, the strategy intercepts
/// the call and finishes the run with the accumulated buffer as the final answer — the tool itself
/// takes no input, so the answer is whatever has been accumulated so far.
///
/// How the model sees the current buffer is governed by [`FinalAnswerAccess`]: either the buffer is
/// injected into the system prompt every step, or it is fetched on demand via a read tool. How it
/// *edits* the buffer is governed by [`FinalAnswerEdit`].
///
/// Where the buffer *lives* is governed by [`FinalAnswerBuffer`]; it defaults to
/// [`InMemoryBuffer`], but any read/write storage — a file, redis, a database row — works.
///
/// No return value.
pub struct BufferedFinalAnswerStrategy<B = InMemoryBuffer> {
    content: Arc<B>,
    access: AccessMode<B>,
    tools: Vec<Tool>,
}

impl BufferedFinalAnswerStrategy<InMemoryBuffer> {
    /// Creates a strategy with an empty in-memory buffer, the given [access](FinalAnswerAccess) and
    /// [edit](FinalAnswerEdit) modes.
    pub fn new(access: FinalAnswerAccess, edit: FinalAnswerEdit) -> Self {
        Self::with_buffer(InMemoryBuffer::default(), access, edit)
    }
}

impl<B: FinalAnswerBuffer + 'static> BufferedFinalAnswerStrategy<B> {
    /// Creates a strategy backed by `buffer`, with the given [access](FinalAnswerAccess) and
    /// [edit](FinalAnswerEdit) modes.
    ///
    /// The buffer is not cleared, so a pre-populated one resumes from whatever it already holds.
    pub fn with_buffer(buffer: B, access: FinalAnswerAccess, edit: FinalAnswerEdit) -> Self {
        let content = Arc::new(buffer);
        let access = AccessMode::new(access, Arc::clone(&content), edit);
        let tools = [
            access.tool(),
            Some(edit.tool(Arc::clone(&content))),
            Some(Tool::from(SubmitFinalAnswer)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        Self {
            content,
            access,
            tools,
        }
    }
}

#[async_trait]
impl<L: Llm, O: OutputCtor, B: FinalAnswerBuffer + 'static> Strategy<L, O>
    for BufferedFinalAnswerStrategy<B>
{
    type Output = ();
    type Error = BufferedFinalAnswerError<B::Error>;

    fn additional_tools<'a>(&'a self) -> impl ToolResolver<'a> + 'a {
        self.tools.as_slice()
    }

    async fn edit_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        self.access.edit_transcript(transcript).await?;
        Ok(())
    }

    async fn project_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        self.access
            .project_transcript(transcript)
            .await
            .map_err(BufferedFinalAnswerError::Buffer)?;
        Ok(())
    }

    async fn process_llm_step(&mut self, step: LlmStep) -> Result<LlmStep, Self::Error> {
        if let LlmStep::ToolCall(msg) = &step
            && msg.tool_calls.iter().any(|c| c.name == SUBMIT_FINAL_ANSWER)
        {
            let content = self.content.read().await.map_err(Self::Error::Buffer)?;
            return Ok(LlmStep::Finish(AssistantMessage {
                thought: msg.thought.clone(),
                content: vec![ContentPart::Text(content)],
            }));
        }
        Ok(step)
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{ContentExt, ToolCall};

    #[tokio::test]
    async fn submit_finishes_with_accumulated_answer() {
        let mut strategy =
            BufferedFinalAnswerStrategy::new(FinalAnswerAccess::ReadTool, FinalAnswerEdit::Replace);
        strategy
            .content
            .write("the answer".to_owned())
            .await
            .unwrap();

        let step = LlmStep::new_tool_calls(
            Some("done".to_owned()),
            [ToolCall::new(Some("c1".into()), SUBMIT_FINAL_ANSWER, None)],
        );

        let finished =
            Strategy::<crate::llm::OpenAIChat, crate::chain::StringCtor>::process_llm_step(
                &mut strategy,
                step,
            )
            .await
            .unwrap();

        let LlmStep::Finish(msg) = finished else {
            panic!("expected the submit call to finish the run");
        };
        assert_eq!(msg.content.to_text().as_deref(), Some("the answer"));
        assert_eq!(msg.thought.as_deref(), Some("done"));
    }

    #[tokio::test]
    async fn other_tool_calls_pass_through() {
        let mut strategy =
            BufferedFinalAnswerStrategy::new(FinalAnswerAccess::ReadTool, FinalAnswerEdit::Replace);

        let step = LlmStep::new_tool_calls(
            None,
            [ToolCall::new(Some("c1".into()), "edit_final_answer", None)],
        );

        let out = Strategy::<crate::llm::OpenAIChat, crate::chain::StringCtor>::process_llm_step(
            &mut strategy,
            step,
        )
        .await
        .unwrap();

        assert!(matches!(out, LlmStep::ToolCall(_)));
    }
}
