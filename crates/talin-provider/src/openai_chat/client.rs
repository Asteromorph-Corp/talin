use async_openai::Client as OpenAIClient;
use async_openai::config::Config;

use crate::openai_chat::client::chat::ChatCompletions;

mod chat;

/// A client for the OpenAI-compatible Chat Completions API, as well as the OpenAI Responses API.
///
/// Uses [`async_openai::Client`] under the hood.
#[derive(Debug, Clone, Default)]
pub struct Client<C: Config>(pub(crate) OpenAIClient<C>);

impl<C: Config> Client<C> {
    /// Creates a new [`Client`] from an existing [`async_openai::Client`].
    pub fn new(client: OpenAIClient<C>) -> Self {
        Self(client)
    }

    /// Builds a new [`Client`] from the given HTTP client and config.
    pub fn build(http_client: reqwest::Client, config: C) -> Self {
        let client = OpenAIClient::build(http_client, config);
        Self::new(client)
    }

    /// Returns a handle for the chat completions endpoint.
    pub fn chat(&self) -> ChatCompletions<'_, C> {
        ChatCompletions::new(self)
    }
}
