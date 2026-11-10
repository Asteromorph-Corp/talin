//! Error types shared by every provider client.

use eventsource_stream::EventStreamError;
use reqwest::StatusCode;
use thiserror::Error;

use crate::ProviderError;

/// Errors that can occur when interacting with a provider API.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error<E> {
    /// An error response returned by the provider's API.
    #[error(transparent)]
    Api(#[from] ApiError<E>),

    /// Network request failed.
    #[error("network request failed: {0:?}")]
    Reqwest(#[from] reqwest::Error),

    /// Reading a streaming response failed.
    #[error("streaming response failed: {0:?}")]
    StreamError(#[from] StreamError),

    /// Failed to deserialize the response from the provider's API.
    #[error("failed to deserialize response: {0}\nRaw response: {1}")]
    JSONDeserialize(#[source] serde_json::Error, String),

    /// Supplied an invalid value for a request header.
    #[error("invalid value for the header `{0}`: {1:?}")]
    InvalidHeader(&'static str, String),

    /// Supplied an invalid value for a request body field.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// An error that does not fit into any of the other categories.
    #[error("{0}")]
    Other(String),
}

impl<E> Error<E> {
    /// Creates a new [`Error::InvalidHeader`] from the given header name and value.
    pub fn invalid_header(header: &'static str, value: impl Into<String>) -> Self {
        Error::InvalidHeader(header, value.into())
    }

    /// Creates a new [`Error::InvalidArgument`] from the given description.
    pub fn invalid_argument(arg: impl Into<String>) -> Self {
        Error::InvalidArgument(arg.into())
    }
}

impl<E> From<EventStreamError<reqwest::Error>> for Error<E> {
    fn from(error: EventStreamError<reqwest::Error>) -> Self {
        match error {
            EventStreamError::Transport(e) => Error::Reqwest(e),
            other => Error::StreamError(StreamError::from(other)),
        }
    }
}

/// An error response returned by the API.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ApiError<E> {
    /// The HTTP status code of the error response.
    pub status_code: Option<u16>,

    /// The body of the error response.
    pub body: E,
}

impl<E> ApiError<E> {
    /// Creates a new [`ApiError`] from the parsed body of the response.
    pub fn new(body: E) -> Self {
        Self {
            status_code: None,
            body,
        }
    }

    /// Creates a new [`ApiError`] from the response status and its parsed body.
    pub fn with_status(status: StatusCode, body: E) -> Self {
        Self {
            status_code: Some(status.as_u16()),
            body,
        }
    }
}

impl ApiError<String> {
    /// Creates a new [`ApiError`] with a `String` body.
    pub fn raw(status: StatusCode, body: String) -> Self {
        Self {
            status_code: Some(status.as_u16()),
            body,
        }
    }
}

impl<E: std::fmt::Display> ApiError<E> {
    /// Converts the [`ApiError`] into a simplified version with a `String` body.
    pub fn stringified(&self) -> ApiError<String> {
        ApiError {
            status_code: self.status_code,
            body: self.body.to_string(),
        }
    }
}

impl<E: ProviderError> ApiError<E> {
    pub(crate) fn is_retryable(&self) -> bool {
        matches!(self.status_code, Some(408 | 429 | 500..=599) | None) || self.body.is_retryable()
    }
}

impl<E: std::fmt::Display> std::fmt::Display for ApiError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "API error")?;
        if let Some(code) = self.status_code {
            write!(f, "[{code}]")?;
        }
        write!(f, ": {}", self.body)
    }
}
impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for ApiError<E> {}

/// Errors that can occur specifically when reading from a streaming response.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StreamError {
    // Boxed because of large error variant
    /// An error from the underlying `eventsource-stream`.
    #[error("{0}")]
    EventSource(String),

    /// The event received from the stream was invalid or malformed.
    #[error("invalid event: {0}")]
    InvalidEvent(String),

    /// The stream was closed unexpectedly.
    #[error("stream closed unexpectedly")]
    UnexpectedClose,
}

impl StreamError {
    /// Creates a new [`StreamError::InvalidEvent`] with the given explanation.
    pub(crate) fn invalid_event(explanation: impl Into<String>) -> Self {
        StreamError::InvalidEvent(explanation.into())
    }
}

impl From<EventStreamError<reqwest::Error>> for StreamError {
    fn from(e: EventStreamError<reqwest::Error>) -> Self {
        StreamError::EventSource(e.to_string())
    }
}
