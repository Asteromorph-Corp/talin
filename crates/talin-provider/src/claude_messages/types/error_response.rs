use std::convert::Infallible;
use std::str::FromStr;

use serde::Deserialize;

use crate::utils::string_enum;

/// An error response from the Claude API.
///
/// See [`Errors`](https://docs.claude.com/en/api/errors)
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ApiErrorResponse {
    /// The id of the request that caused the error.
    pub request_id: Option<String>,

    /// Error details.
    pub error: ApiErrorDetails,
}

/// API error details.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ApiErrorDetails {
    /// The error type.
    #[serde(rename = "type")]
    pub type_: ErrorType,

    /// A human-readable error message.
    pub message: String,
}

string_enum! {
    /// The possible error types for the Messages endpoint.
    #[serde(rename_all = "snake_case")]
    pub enum ErrorType {
        /// `"invalid_request_error"` (HTTP 400)
        InvalidRequestError,

        /// `"authentication_error"` (HTTP 401)
        AuthenticationError,

        /// `"billing_error"` (HTTP 402)
        BillingError,

        /// `"permission_error"` (HTTP 403)
        PermissionError,

        /// `"not_found_error"` (HTTP 404)
        NotFoundError,

        /// `"timeout_error"` (HTTP 408)
        TimeoutError,

        /// `"request_too_large"` (HTTP 413)
        RequestTooLarge,

        /// `"rate_limit_error"` (HTTP 429)
        RateLimitError,

        /// `"api_error"` (HTTP 500)
        ApiError,

        /// `"overloaded_error"` (HTTP 529)
        OverloadedError,
    }
}

impl ApiErrorResponse {
    /// Creates a new [`ApiErrorResponse`] from the given error details.
    pub fn new(error: ApiErrorDetails) -> Self {
        Self {
            request_id: None,
            error,
        }
    }

    /// Sets the [`request_id`](Self::request_id) of the request that caused the error.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl ApiErrorDetails {
    /// Creates a new [`ApiErrorDetails`] from the given type and message.
    pub fn new(type_: ErrorType, message: impl Into<String>) -> Self {
        Self {
            type_,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::InvalidRequestError => write!(f, "invalid_request_error"),
            ErrorType::AuthenticationError => write!(f, "authentication_error"),
            ErrorType::BillingError => write!(f, "billing_error"),
            ErrorType::PermissionError => write!(f, "permission_error"),
            ErrorType::NotFoundError => write!(f, "not_found_error"),
            ErrorType::TimeoutError => write!(f, "timeout_error"),
            ErrorType::RequestTooLarge => write!(f, "request_too_large"),
            ErrorType::RateLimitError => write!(f, "rate_limit_error"),
            ErrorType::ApiError => write!(f, "api_error"),
            ErrorType::OverloadedError => write!(f, "overloaded_error"),
            ErrorType::Unknown(s) => write!(f, "{s}"),
        }
    }
}

impl FromStr for ErrorType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.to_lowercase().replace(" ", "_");
        let type_ = match normalized.as_str() {
            "invalid_request_error" => ErrorType::InvalidRequestError,
            "authentication_error" => ErrorType::AuthenticationError,
            "billing_error" => ErrorType::BillingError,
            "permission_error" => ErrorType::PermissionError,
            "not_found_error" => ErrorType::NotFoundError,
            "timeout_error" => ErrorType::TimeoutError,
            "request_too_large" => ErrorType::RequestTooLarge,
            "rate_limit_error" => ErrorType::RateLimitError,
            "api_error" => ErrorType::ApiError,
            "overloaded_error" => ErrorType::OverloadedError,
            _ => ErrorType::Unknown(normalized),
        };
        Ok(type_)
    }
}

impl std::fmt::Display for ApiErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.type_, self.message)
    }
}
