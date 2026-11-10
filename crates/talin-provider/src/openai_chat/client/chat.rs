use async_openai::config::Config;
use async_openai::error::ApiError;
use async_openai::types::chat::{CreateChatCompletionResponse, CreateChatCompletionStreamResponse};

use crate::error::Error;
use crate::openai_chat::Client;
use crate::openai_chat::stream::assemble;
use crate::openai_chat::types::ChatRequest;

/// A handle for interacting with the chat completions endpoint.
pub struct ChatCompletions<'a, C: Config> {
    client: &'a Client<C>,
}

impl<'a, C: Config> ChatCompletions<'a, C> {
    pub fn new(client: &'a Client<C>) -> Self {
        ChatCompletions { client }
    }

    pub async fn generate(
        &self,
        request: &ChatRequest,
    ) -> Result<CreateChatCompletionResponse, Error<ApiError>> {
        let response = if request.inner.stream.unwrap_or_default() {
            let stream = self
                .client
                .0
                .chat()
                .create_stream_byot::<_, CreateChatCompletionStreamResponse>(request)
                .await?;
            assemble(stream).await?
        } else {
            self.client
                .0
                .chat()
                .create_byot::<_, CreateChatCompletionResponse>(request)
                .await?
        };

        Ok(response)
    }
}
