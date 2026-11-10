use std::convert::Infallible;

use async_trait::async_trait;

use crate::chain::OutputCtor;
use crate::llm::Llm;
use crate::schemas::{LlmStep, Message, ToolOutput, ToolResult, ToolResultMessage};
use crate::strategy::Strategy;

/// A [`Strategy`] that records all messages and outputs during the agent execution.
///
/// This can be useful to inspect the conversation history after the execution has completed.
///
/// Returns a vector of all recorded messages upon finalization.
#[derive(Default)]
pub struct RecordStrategy {
    record: Vec<Message>,
}

impl RecordStrategy {
    /// Creates a new `RecordStrategy`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl<L: Llm, O: OutputCtor> Strategy<L, O> for RecordStrategy {
    type Output = Vec<Message>;
    type Error = Infallible;

    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, Self::Error> {
        self.record.extend(messages.clone());
        Ok(messages)
    }

    async fn process_llm_step(&mut self, output: LlmStep) -> Result<LlmStep, Self::Error> {
        self.record.push(Message::from(output.clone()));
        Ok(output)
    }

    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, Self::Error> {
        let content = match &message.result {
            Ok(output) => output.clone(),
            Err(error) => ToolOutput::from(error.feedback().to_string()),
        };
        self.record.push(Message::new_tool_message(
            message.call_id,
            message.name,
            content,
        ));
        Ok(message.result)
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(self.record)
    }
}
