use crate::error::Error;
use crate::gemini_generate_content::Client;
use crate::gemini_generate_content::stream::assemble;
use crate::gemini_generate_content::types::{
    ApiErrorDetails, GenerateContentRequest, GenerateContentResponse,
};
use crate::stream::ResponseStream;

/// A handle for interacting with the generate content endpoint.
pub struct GenerateContent<'a> {
    client: &'a Client,
    /// The model to use for generation.
    model: String,
}

impl<'a> GenerateContent<'a> {
    /// Creates a new [`GenerateContent`] instance.
    pub fn new(client: &'a Client, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    /// Makes a `POST` request to the `generateContent` endpoint.
    pub async fn generate(
        &self,
        request: &GenerateContentRequest,
    ) -> Result<GenerateContentResponse, Error<ApiErrorDetails>> {
        let path = format!("/v1beta/models/{}:generateContent", self.model);

        self.client
            .post::<_, GenerateContentResponse>(&path, request)
            .await
    }

    /// Makes a `POST` request to the `streamGenerateContent` endpoint with streaming enabled.
    pub async fn generate_stream(
        &self,
        request: &GenerateContentRequest,
    ) -> Result<ResponseStream<GenerateContentResponse, ApiErrorDetails>, Error<ApiErrorDetails>>
    {
        let path = format!("/v1beta/models/{}:streamGenerateContent", self.model);

        self.client.post_stream(&path, request).await
    }

    /// Makes a `POST` request to the `streamGenerateContent` endpoint and assembles the full
    /// response.
    pub async fn generate_with_stream(
        &self,
        request: &GenerateContentRequest,
    ) -> Result<GenerateContentResponse, Error<ApiErrorDetails>> {
        let stream = self.generate_stream(request).await?;
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
}
