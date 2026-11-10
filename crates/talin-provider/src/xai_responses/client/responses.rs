use crate::error::Error;
use crate::stream::ResponseStream;
use crate::xai_responses::client::Client;
use crate::xai_responses::stream::assemble;
use crate::xai_responses::types::{ModelRequest, ModelResponse, ResponseStatus, StreamEvent};

/// A client for the `/responses` endpoint.
pub struct Responses<'a> {
    client: &'a Client,
}

impl<'a> Responses<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new response.
    ///
    /// If `stream` is set to `true` in the request, this method will handle the stream assembly
    /// and return the accumulated response.
    ///
    /// # Arguments
    ///
    /// * `request` - The request body.
    pub async fn create(
        &self,
        request: &ModelRequest,
    ) -> Result<ModelResponse, Error<serde_json::Value>> {
        if request.stream.unwrap_or(false) {
            let stream = self.create_stream(request).await?;
            let response = assemble(stream).await;
            match response {
                Ok(response) => Ok(response),
                Err(partial) => {
                    if let Some(mut partial) = partial.response {
                        tracing::warn!(error = ?partial.error, "Stream failed with error, returning partial response");
                        // Mark as incomplete if not already
                        if partial.status != ResponseStatus::Incomplete {
                            partial.status = ResponseStatus::Incomplete;
                        }
                        Ok(*partial)
                    } else {
                        Err(partial.error)
                    }
                }
            }
        } else {
            self.client
                .post::<_, ModelResponse>("/responses", request)
                .await
        }
    }

    /// Create a new response stream.
    ///
    /// # Arguments
    ///
    /// * `request` - The request body.
    pub async fn create_stream(
        &self,
        request: &ModelRequest,
    ) -> Result<ResponseStream<StreamEvent, serde_json::Value>, Error<serde_json::Value>> {
        if request.stream.is_none() || request.stream == Some(false) {
            return Err(Error::invalid_argument(
                "`stream` should be `true` for `create_stream`.",
            ));
        }

        self.client.post_stream("/responses", request).await
    }
}
