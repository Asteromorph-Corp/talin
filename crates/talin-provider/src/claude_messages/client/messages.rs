use crate::claude_messages::Client;
use crate::claude_messages::stream::assemble;
use crate::claude_messages::types::{
    ApiErrorDetails, CountTokensRequest, MessageTokensCount, MessagesRequest, MessagesResponse,
    RawMessageStreamEvent, StopReason,
};
use crate::error::Error;
use crate::stream::ResponseStream;

/// A handle for interacting with the messages endpoint.
pub struct Messages<'a> {
    client: &'a Client,
}

impl<'a> Messages<'a> {
    /// Creates a new [`Messages`] instance.
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Counts a prompt without generating a response.
    pub async fn count_tokens(
        &self,
        request: &CountTokensRequest,
    ) -> Result<MessageTokensCount, Error<ApiErrorDetails>> {
        self.client
            .post(&self.client.config.count_tokens_path(), request)
            .await
    }

    /// Makes a `POST` request to the `/messages` endpoint.
    pub async fn generate(
        &self,
        request: &MessagesRequest,
    ) -> Result<MessagesResponse, Error<ApiErrorDetails>> {
        if request.stream.is_none_or(|stream| !stream) {
            let path = self
                .client
                .config
                .messages_path(request.model.as_str(), request.stream.unwrap_or(false));
            let request = self.client.config.messages_body(request)?;
            return self.client.post(&path, &request).await;
        }

        // Stream is set to `true`
        let stream = self.generate_stream(request).await?;
        let message = assemble(stream).await;
        match message {
            Ok(message) => Ok(message),
            Err(err) => {
                if let Some(mut partial) = err.response {
                    // Log the error and return partial message with the `"pause_turn"` stop reason.
                    tracing::warn!(error = %err.error, "Stream failed with error, returning partial message");
                    partial.stop_reason = Some(StopReason::PauseTurn);
                    Ok(*partial)
                } else {
                    Err(err.error)
                }
            }
        }
    }

    /// Makes a `POST` request to the `/messages` endpoint with streaming enabled.
    pub async fn generate_stream(
        &self,
        request: &MessagesRequest,
    ) -> Result<ResponseStream<RawMessageStreamEvent, ApiErrorDetails>, Error<ApiErrorDetails>>
    {
        if request.stream.is_none_or(|stream| !stream) {
            return Err(Error::invalid_argument(
                "`stream` should be `true` for `generate_stream`.",
            ));
        }

        let path = self
            .client
            .config
            .messages_path(request.model.as_str(), request.stream.unwrap_or(false));
        let request = self.client.config.messages_body(request)?;
        self.client.post_stream(&path, &request).await
    }
}
