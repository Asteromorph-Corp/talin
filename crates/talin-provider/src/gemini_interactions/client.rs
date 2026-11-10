//! The `client` module provides functionality for interacting with the Gemini Interactions
//! API.

use async_trait::async_trait;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::gemini_interactions::ClientConfig;
use crate::gemini_interactions::types::ErrorMessage;
use crate::{ApiClient, Provider, ProviderError};

mod interactions;

pub use interactions::Interactions;

/// The Gemini Interactions API.
#[derive(Debug, Clone, Copy)]
pub struct Api;

/// A client for the Gemini Interactions API.
pub type Client = ApiClient<Api>;

#[async_trait]
impl Provider for Api {
    type Config = ClientConfig;
    type Error = ErrorMessage;

    /// `path` is a resource path beginning with `/`; the base URL and API version are prepended.
    async fn build_request<I: Serialize + Send + Sync>(
        config: &ClientConfig,
        http: &reqwest::Client,
        method: Method,
        path: &str,
        body: Option<&I>,
        streaming: bool,
    ) -> Result<reqwest::Request, Error<ErrorMessage>> {
        let mut builder = http
            .request(method, config.url(path))
            .headers(config.headers(streaming)?);
        if let Some(body) = body {
            builder = builder.json(body);
        }
        builder.build().map_err(Error::from)
    }
}

impl Client {
    /// Returns a handle for the interactions endpoints.
    pub fn interactions(&self) -> Interactions<'_> {
        Interactions::new(self)
    }
}

/// Wrapper for an API error response body (`{ "error": ... }`).
#[derive(Deserialize)]
struct ApiErrorResponse {
    /// The error details.
    error: ErrorMessage,
}

impl ProviderError for ErrorMessage {
    fn parse(status: StatusCode, body: &str) -> Self {
        match serde_json::from_str::<ApiErrorResponse>(body) {
            Ok(api_response) => api_response.error,
            Err(_) => {
                let message = (!body.is_empty())
                    .then_some(body)
                    .or_else(|| status.canonical_reason())
                    .unwrap_or("Unknown error");

                ErrorMessage {
                    code: None,
                    message: Some(message.to_owned()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_error_response_body() {
        let body = r#"{"error":{"code":"not_found","message":"Interaction not found"}}"#;
        let details = ErrorMessage::parse(StatusCode::NOT_FOUND, body);

        assert_eq!(details.message.as_deref(), Some("Interaction not found"));
        assert_eq!(details.code.as_deref(), Some("not_found"));
    }
}
