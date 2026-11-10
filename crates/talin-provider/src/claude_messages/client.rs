//! The `client` module provides functionalities for interacting with the Anthropic Claude Messages
//! API.

use std::str::FromStr;

use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Serialize;

use crate::claude_messages::client::messages::Messages;
use crate::claude_messages::config::ClientConfig;
use crate::claude_messages::types::{ApiErrorDetails, ApiErrorResponse, ErrorType};
use crate::client::http1_only_client;
use crate::{ApiClient, Provider, ProviderError};

mod messages;

/// The Claude Messages API.
#[derive(Debug, Clone, Copy)]
pub struct Api;
/// The client for the Claude Messages API.
pub type Client = ApiClient<Api>;

#[async_trait]
impl Provider for Api {
    type Config = ClientConfig;
    type Error = ApiErrorDetails;

    /// Uses an HTTP/1 only client, as issues were reported when using HTTP/2 for the streaming api.
    fn default_http_client() -> reqwest::Client {
        http1_only_client()
    }

    async fn build_request<I: Serialize + Send + Sync>(
        config: &Self::Config,
        http: &reqwest::Client,
        method: reqwest::Method,
        path: &str,
        body: Option<&I>,
        _streaming: bool,
    ) -> Result<reqwest::Request, crate::error::Error<Self::Error>> {
        let mut builder = http
            .request(method, config.url(path))
            .headers(config.headers().await?);
        if let Some(body) = body {
            builder = builder.json(body);
        }
        Ok(builder.build()?)
    }
}

impl Client {
    /// Checks whether the specific beta flag is enabled.
    pub fn is_beta_enabled(&self, flag: &str) -> bool {
        self.config.is_beta_enabled(flag)
    }

    /// Returns a handle for the messages endpoint.
    pub fn messages(&self) -> Messages<'_> {
        Messages::new(self)
    }
}

impl ProviderError for ApiErrorDetails {
    fn parse(status: StatusCode, body: &str) -> Self {
        match serde_json::from_str::<ApiErrorResponse>(body) {
            Ok(api_response) => api_response.error,
            Err(_) => details_from_status(status, body),
        }
    }

    /// Retries rate limit, API, and overloaded errors.
    fn is_retryable(&self) -> bool {
        matches!(
            self.type_,
            ErrorType::TimeoutError
                | ErrorType::RateLimitError
                | ErrorType::ApiError
                | ErrorType::OverloadedError
                | ErrorType::Unknown(_)
        )
    }
}

/// Creates [`ApiErrorDetails`] from the status of a response whose body is not an
/// [`ApiErrorResponse`].
fn details_from_status(status: StatusCode, body: &str) -> ApiErrorDetails {
    let message = (!body.is_empty())
        .then_some(body)
        .or_else(|| status.canonical_reason())
        .unwrap_or("Unknown error");

    let type_ = match status.as_u16() {
        400 => ErrorType::InvalidRequestError,
        401 => ErrorType::AuthenticationError,
        402 => ErrorType::BillingError,
        403 => ErrorType::PermissionError,
        404 => ErrorType::NotFoundError,
        408 => ErrorType::TimeoutError,
        413 => ErrorType::RequestTooLarge,
        429 => ErrorType::RateLimitError,
        500 => ErrorType::ApiError,
        529 => ErrorType::OverloadedError,
        _ => {
            ErrorType::from_str(message).unwrap_or_else(|_| ErrorType::Unknown(message.to_owned()))
        }
    };

    ApiErrorDetails {
        type_,
        message: message.to_owned(),
    }
}
