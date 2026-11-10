use async_trait::async_trait;
use reqwest::{Method, StatusCode};
use serde::Serialize;

use crate::error::Error;

/// The provider-specific behaviour of an [`ApiClient`](crate::ApiClient).
#[async_trait]
pub trait Provider {
    /// The configuration of the client.
    type Config;

    /// The body of an error response returned by the API.
    type Error: ProviderError;

    /// The default HTTP client to use for requests.
    fn default_http_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    /// Builds a request for the resource `path`, with an optional JSON `body`.
    async fn build_request<I: Serialize + Send + Sync>(
        config: &Self::Config,
        http: &reqwest::Client,
        method: Method,
        path: &str,
        body: Option<&I>,
        streaming: bool,
    ) -> Result<reqwest::Request, Error<Self::Error>>;
}

/// An API error response returned by the provider.
pub trait ProviderError: Send + Sync + 'static {
    /// Parse error from the status code and body of a non-success response.
    fn parse(status: StatusCode, body: &str) -> Self;

    /// Returns whether a request that failed should be retried.
    fn is_retryable(&self) -> bool {
        false
    }
}

impl ProviderError for serde_json::Value {
    fn parse(_status: StatusCode, body: &str) -> Self {
        serde_json::from_str(body).unwrap_or_else(|_| serde_json::json!({ "error": body }))
    }
}
