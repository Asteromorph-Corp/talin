use reqwest::Method;

use crate::error::Error;
use crate::gemini_interactions::Client;
use crate::gemini_interactions::stream::assemble;
use crate::gemini_interactions::types::{
    CreateInteractionRequest, ErrorMessage, Interaction, InteractionSseStreamEnvelope,
};
use crate::stream::ResponseStream;

/// A handle for interacting with the interactions endpoint.
pub struct Interactions<'a> {
    client: &'a Client,
}

impl<'a> Interactions<'a> {
    /// Creates a new [`Interactions`] accessor.
    pub(super) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Creates a new interaction and waits for it to complete.
    pub async fn create(
        &self,
        request: &CreateInteractionRequest,
    ) -> Result<Interaction, Error<ErrorMessage>> {
        self.client
            .send_json(Method::POST, "/interactions", Some(&request))
            .await
    }

    /// Creates a new interaction and returns a stream of server-sent events.
    pub async fn create_stream(
        &self,
        request: &CreateInteractionRequest,
    ) -> Result<ResponseStream<InteractionSseStreamEnvelope, ErrorMessage>, Error<ErrorMessage>>
    {
        self.client
            .send_stream(Method::POST, "/interactions", Some(&request))
            .await
    }

    /// Creates a new interaction by streaming it and assembling the final [`Interaction`].
    pub async fn create_with_stream(
        &self,
        request: &CreateInteractionRequest,
    ) -> Result<Interaction, Error<ErrorMessage>> {
        let stream = self.create_stream(request).await?;
        let response = assemble(stream).await;
        match response {
            Ok(response) => Ok(response),
            Err(err) => {
                // Log the error and return partial response
                if let Some(partial) = err.response {
                    tracing::warn!(error = %err.error, "Stream failed with error, returning partial response");
                    Ok(*partial)
                } else {
                    Err(err.error)
                }
            }
        }
    }

    /// Retrieves an interaction by its ID.
    pub async fn get(&self, id: impl AsRef<str>) -> Result<Interaction, Error<ErrorMessage>> {
        let path = format!("/interactions/{}", id.as_ref());
        self.client.send_json(Method::GET, &path, None::<&()>).await
    }

    /// Retrieves an interaction by its ID as a stream of server-sent events.
    ///
    /// If `last_event_id` is provided, the stream resumes from the chunk after that event.
    pub async fn get_stream(
        &self,
        id: impl AsRef<str>,
        last_event_id: Option<&str>,
    ) -> Result<ResponseStream<InteractionSseStreamEnvelope, ErrorMessage>, Error<ErrorMessage>>
    {
        let mut path = format!("/interactions/{}?stream=true", id.as_ref());
        if let Some(event_id) = last_event_id {
            path.push_str(&format!("&last_event_id={event_id}"));
        }
        self.client
            .send_stream(Method::GET, &path, None::<&()>)
            .await
    }

    /// Cancels an in-progress background interaction by its ID.
    pub async fn cancel(&self, id: impl AsRef<str>) -> Result<Interaction, Error<ErrorMessage>> {
        let path = format!("/interactions/{}/cancel", id.as_ref());
        self.client
            .send_json(Method::POST, &path, None::<&()>)
            .await
    }
}
