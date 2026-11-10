use async_trait::async_trait;

use crate::chain::OutputCtor;
use crate::llm::Llm;
use crate::schemas::{AssistantMessage, BodyMessage, ToolCallMessage};
use crate::strategy::Strategy;
use crate::transcript::{AgentTranscript, EntryEdit, EntryView, ModifyError, Scope};

/// A [`Strategy`] that drops the `thought` field from messages.
///
/// For chat completions models without dedicated support for reasoning, the `thought` part of the
/// message is often returned as a part of the message content, delimited by special tokens.
/// Including this in subsequent requests can often degrade response quality.
/// This strategy prevents this by removing the `thought` field from messages before they are sent
/// to the model.
///
/// The edit is ephemeral: it is applied to a projection of the transcript, so the stored transcript
/// keeps the reasoning and only the outgoing request loses it.
///
/// Because the messages are rebuilt from their canonical form, this also discards provider-side
/// reasoning blocks (Claude's `thinking`, the Responses API's `reasoning` items) for providers that
/// have them.
///
/// No return value.
#[derive(Default, Clone)]
pub struct DropThoughtStrategy;

#[async_trait]
impl<L: Llm, O: OutputCtor> Strategy<L, O> for DropThoughtStrategy {
    type Output = ();
    type Error = ModifyError;

    async fn project_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        let edits = transcript
            .full_view()
            .into_iter()
            .map(|entry| {
                let EntryView::Message(id, message) = entry else {
                    return EntryEdit::from(entry);
                };
                // `retain = false` rebuilds the provider item from the stripped message, so a
                // provider that stores reasoning natively loses it along with the canonical field.
                match message {
                    BodyMessage::Assistant(m) if m.thought.is_some() => {
                        let message = BodyMessage::Assistant(AssistantMessage {
                            thought: None,
                            ..m.clone()
                        });
                        EntryEdit::Replace(id, message, false)
                    }
                    BodyMessage::ToolCall(m) if m.thought.is_some() => {
                        let message = BodyMessage::ToolCall(ToolCallMessage {
                            thought: None,
                            ..m.clone()
                        });
                        EntryEdit::Replace(id, message, false)
                    }
                    _ => EntryEdit::from(entry),
                }
            })
            .collect();

        transcript.modify(edits, &Scope::full())
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use talin_provider::openai_chat::types::{
        ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestMessage,
    };

    use super::*;
    use crate::chain::StringCtor;
    use crate::llm::GenericChat;
    use crate::llm::openai_chat::GenericChatMessage;
    use crate::schemas::{ToolCall, UserMessage};
    use crate::transcript::AgentTranscript;

    type Transcript = AgentTranscript<Vec<String>, GenericChatMessage>;

    async fn project(transcript: &mut Transcript) {
        <DropThoughtStrategy as Strategy<GenericChat, StringCtor>>::project_transcript(
            &mut DropThoughtStrategy,
            transcript,
        )
        .await
        .unwrap();
    }

    fn thoughts(transcript: &Transcript) -> Vec<Option<String>> {
        transcript
            .full_view()
            .into_iter()
            .filter_map(|entry| match entry {
                EntryView::Message(_, BodyMessage::Assistant(m)) => Some(m.thought.clone()),
                EntryView::Message(_, BodyMessage::ToolCall(m)) => Some(m.thought.clone()),
                _ => None,
            })
            .collect()
    }

    #[tokio::test]
    async fn strips_thought() {
        let call = ToolCall::new(Some("call_1".to_owned()), "search", None);
        let mut transcript = Transcript::new(
            Vec::new(),
            vec![
                BodyMessage::User(UserMessage::new("hi")),
                BodyMessage::Assistant(AssistantMessage::new(Some("pondering".into()), "hello")),
                BodyMessage::ToolCall(ToolCallMessage::new(Some("deciding".into()), vec![call])),
            ],
        )
        .unwrap();

        project(&mut transcript).await;

        assert_eq!(thoughts(&transcript), [None, None]);
    }

    #[tokio::test]
    async fn keeps_message_content() {
        let mut transcript = Transcript::new(
            Vec::new(),
            vec![BodyMessage::Assistant(AssistantMessage::new(
                Some("pondering".into()),
                "hello",
            ))],
        )
        .unwrap();

        project(&mut transcript).await;

        let (_system, items) = transcript.materialize();
        let ChatCompletionRequestMessage::Assistant(m) = &items[0].0 else {
            panic!("expected an assistant message");
        };
        let ChatCompletionRequestAssistantMessageContent::Text(content) =
            m.content.as_ref().expect("Expected content")
        else {
            panic!("expected text content");
        };
        assert_eq!(content, "hello");
    }
}
