use std::convert::Infallible;

use async_trait::async_trait;
use talin_provider::openai_chat::Config;
use talin_provider::openai_chat::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage,
};

use crate::chain::OutputCtor;
use crate::llm::openai_chat::GenericChatMessage;
use crate::llm::{GenericChat, OpenAIChat};
use crate::strategy::Strategy;

/// A [`Strategy`] that converts system messages into assistant messages.
///
/// This is useful for models that do not support system messages, such as OpenAI's GPT-3.5-turbo
/// and GPT-4.
///
/// No return value.
#[derive(Default, Clone)]
pub struct SystemToAssistantStrategy;

#[async_trait]
impl<O: OutputCtor, C: Config + Clone + Send + Sync + 'static> Strategy<GenericChat<C>, O>
    for SystemToAssistantStrategy
{
    type Output = ();
    type Error = Infallible;

    async fn process_materialized(
        &mut self,
        system: Vec<String>,
        items: Vec<GenericChatMessage>,
    ) -> Result<(Vec<String>, Vec<GenericChatMessage>), Self::Error> {
        let items = assistant_messages(system)
            .map(GenericChatMessage)
            .chain(items)
            .collect();
        Ok((vec![], items))
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

#[async_trait]
impl<O: OutputCtor, C: Config + Clone + Send + Sync + 'static> Strategy<OpenAIChat<C>, O>
    for SystemToAssistantStrategy
{
    type Output = ();
    type Error = Infallible;

    async fn process_materialized(
        &mut self,
        system: Vec<String>,
        items: Vec<ChatCompletionRequestMessage>,
    ) -> Result<(Vec<String>, Vec<ChatCompletionRequestMessage>), Self::Error> {
        let items = assistant_messages(system).chain(items).collect();
        Ok((vec![], items))
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

/// Renders each system prompt as an assistant message.
fn assistant_messages(
    system: Vec<String>,
) -> impl Iterator<Item = ChatCompletionRequestMessage> + Send {
    system.into_iter().map(|content| {
        // Use deprecated field `function_call` for construction.
        #[allow(deprecated)]
        ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
            content: Some(ChatCompletionRequestAssistantMessageContent::Text(content)),
            refusal: None,
            name: None,
            audio: None,
            tool_calls: None,
            function_call: None,
        })
    })
}
