use async_openai::config::Config;
use async_openai::error::ApiError;
use async_openai::types::responses::{CreateResponse, Response};

use crate::error::Error;
use crate::openai_responses::Client;
use crate::openai_responses::types::{
    CompactResource, CompactResponseRequest, TokenCountsBody, TokenCountsResource,
};

/// A handle for interacting with the responses endpoint.
pub struct Responses<'a, C: Config> {
    client: &'a Client<C>,
}

impl<'a, C: Config> Responses<'a, C> {
    pub fn new(client: &'a Client<C>) -> Self {
        Responses { client }
    }

    pub async fn generate(&self, request: CreateResponse) -> Result<Response, Error<ApiError>> {
        let response = self.client.0.responses().create(request).await?;
        Ok(response)
    }
    /// Counts input tokens without generating a response.
    pub async fn count_tokens(
        &self,
        request: TokenCountsBody,
    ) -> Result<TokenCountsResource, Error<ApiError>> {
        Ok(self
            .client
            .0
            .responses()
            .get_input_token_counts(request)
            .await?)
    }

    /// Compacts a conversation into the provider's replacement item vector.
    pub async fn compact(
        &self,
        request: CompactResponseRequest,
    ) -> Result<CompactResource, Error<ApiError>> {
        Ok(self.client.0.responses().compact_byot(request).await?)
    }
}
