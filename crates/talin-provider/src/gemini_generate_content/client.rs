//! The `client` module provides functionalities for interacting with the Gemini Generate Content
//! API.

use async_trait::async_trait;
use reqwest::{Method, StatusCode};
use serde::Serialize;

use crate::error::Error;
use crate::gemini_generate_content::ClientConfig;
use crate::gemini_generate_content::client::generate_content::GenerateContent;
use crate::gemini_generate_content::types::{ApiErrorDetails, ApiErrorResponse};
use crate::{ApiClient, Provider, ProviderError};

mod generate_content;

/// The Gemini Generate Content API.
#[derive(Debug, Clone, Copy)]
pub struct Api;
/// The client for the Gemini Generate Content API.
pub type Client = ApiClient<Api>;

impl Client {
    /// Returns a handle for the generate content endpoint.
    pub fn generate_content(&self, model: impl Into<String>) -> GenerateContent<'_> {
        GenerateContent::new(self, model)
    }
}

#[async_trait]
impl Provider for Api {
    type Config = ClientConfig;
    type Error = ApiErrorDetails;

    async fn build_request<I: Serialize + Send + Sync>(
        config: &ClientConfig,
        http: &reqwest::Client,
        method: Method,
        path: &str,
        body: Option<&I>,
        streaming: bool,
    ) -> Result<reqwest::Request, Error<ApiErrorDetails>> {
        let mut builder = http
            .request(method, config.url(path))
            .headers(config.headers()?);
        if let Some(body) = body {
            builder = builder.json(body);
        }
        if streaming {
            builder = builder.query(&[("alt", "sse")]);
        }
        builder.build().map_err(Error::from)
    }
}

impl ProviderError for ApiErrorDetails {
    fn parse(status: StatusCode, body: &str) -> Self {
        match serde_json::from_str::<ApiErrorResponse>(body) {
            Ok(api_response) => api_response.error,
            Err(_) => {
                let message = (!body.is_empty())
                    .then_some(body)
                    .or_else(|| status.canonical_reason())
                    .unwrap_or("Unknown error");

                ApiErrorDetails {
                    code: Some(i32::from(status.as_u16())),
                    message: Some(message.to_owned()),
                    status: None,
                    details: None,
                }
            }
        }
    }

    /// Retries timeout (`408`), rate limit (`429`), and server errors (`5xx`).
    ///
    /// If the error code is not available, it is assumed to be retryable.
    fn is_retryable(&self) -> bool {
        matches!(self.code, Some(408 | 429 | 500..=599) | None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_api_error_response() {
        let body =
            r#"{"error":{"code":400,"message":"Invalid argument","status":"INVALID_ARGUMENT"}}"#;
        let details = ApiErrorDetails::parse(StatusCode::BAD_REQUEST, body);

        assert_eq!(details.message, Some("Invalid argument".to_string()));
        assert_eq!(details.status, Some("INVALID_ARGUMENT".to_string()));
    }

    #[test]
    fn test_rate_limited_error() {
        let body = r#"{"error":{"code":429,"message":"Rate limit exceeded","status":"RESOURCE_EXHAUSTED"}}"#;
        let details = ApiErrorDetails::parse(StatusCode::TOO_MANY_REQUESTS, body);

        assert_eq!(details.message, Some("Rate limit exceeded".to_string()));
        assert_eq!(details.status, Some("RESOURCE_EXHAUSTED".to_string()));
    }

    #[test]
    fn test_is_retryable() {
        for (status, retryable) in [
            (StatusCode::TOO_MANY_REQUESTS, true),
            (StatusCode::INTERNAL_SERVER_ERROR, true),
            (StatusCode::UNAUTHORIZED, false),
        ] {
            let details = ApiErrorDetails::parse(status, "");
            assert_eq!(details.is_retryable(), retryable, "{status}");
        }
    }
}
