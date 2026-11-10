use talin_provider::error::{ApiError, Error};
use thiserror::Error;

use crate::text::parse::ParseError;

/// Errors that can occur when interacting with LLMs.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LlmError {
    /// An error response returned by the provider's API.
    #[error(transparent)]
    Api(ApiError<String>),

    /// Network request failed.
    #[error("network request failed: {0:?}")]
    Request(#[from] reqwest::Error),

    /// Failed to deserialize the response from the provider's API.
    #[error("failed to deserialize response: {0:?}")]
    ResponseDeserialize(#[source] serde_json::Error),

    /// Supplied an invalid value for a request.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// The provider's API returned an invalid response (e.g. empty content list).
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// Failed to parse model output.
    ///
    /// Produced when the response payload is valid, but the model produced output is not in the
    /// expected format.
    #[error(transparent)]
    Parse(#[from] ParseError),

    /// The provider cannot represent a part in message content.
    #[error(transparent)]
    UnsupportedContent(#[from] UnsupportedContent),

    /// The model response was truncated, for example due to an output token limit.
    #[error(transparent)]
    Truncated(Truncation),

    /// Model refused to answer the prompt, for example due to safety filters.
    #[error("model refused to answer: {0}")]
    Refused(String),

    /// Model required user input.
    #[error("model required user input")]
    InputRequired,

    /// Error not covered by other variants.
    #[error("{0}")]
    Other(String),
}

impl LlmError {
    /// Returns whether retrying the failed request may plausibly succeed.
    pub fn is_retryable(&self) -> bool {
        match self {
            LlmError::Api(e) => matches!(e.status_code, Some(408 | 429 | 500..=599) | None),
            LlmError::Request(e) => e.is_timeout() || e.is_connect(),
            // Provider response format does not match the expected format; retrying will not help
            LlmError::ResponseDeserialize(_) => false,
            // Invalid request parameters; retrying will not help
            LlmError::InvalidRequest(_) => false,
            // Provider may return a valid response on retry.
            LlmError::InvalidResponse(_) => true,
            // The model would produce a different output.
            LlmError::Parse(_) => true,
            // The provider cannot represent the content; retrying will not help.
            LlmError::UnsupportedContent(_) => false,
            // The model may produce a different output on retry.
            LlmError::Truncated(_) => true,
            // The model may produce a different output on retry.
            LlmError::Refused(_) => true,
            // The model may produce a different output on retry.
            LlmError::InputRequired => true,
            // Retryability of other errors is unknown; assume retryable.
            LlmError::Other(_) => true,
        }
    }

    /// Constructs a new [`LlmError::InvalidResponse`] with the given message.
    pub fn invalid_response(msg: impl Into<String>) -> Self {
        LlmError::InvalidResponse(msg.into())
    }

    /// Constructs a new [`LlmError::Refused`] with the given message.
    pub fn refused(msg: impl Into<String>) -> Self {
        LlmError::Refused(msg.into())
    }

    /// Create a new [`LlmError::Other`] with the given message.
    pub fn other(msg: impl Into<String>) -> Self {
        LlmError::Other(msg.into())
    }
}

impl<E: std::fmt::Display> From<Error<E>> for LlmError {
    fn from(e: Error<E>) -> Self {
        match e {
            Error::Api(e) => LlmError::from(e),
            Error::Reqwest(e) => LlmError::Request(e),
            e @ (Error::InvalidHeader(_, _) | Error::InvalidArgument(_)) => {
                LlmError::InvalidRequest(e.to_string())
            }
            Error::JSONDeserialize(error, _) => LlmError::ResponseDeserialize(error),
            other => LlmError::Other(other.to_string()),
        }
    }
}

impl<E: std::fmt::Display> From<ApiError<E>> for LlmError {
    fn from(e: ApiError<E>) -> Self {
        LlmError::Api(e.stringified())
    }
}

/// An error indicating that a provider cannot represent a piece of message content.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("{provider} does not support {content}")]
pub struct UnsupportedContent {
    /// The provider that cannot represent the content.
    pub provider: &'static str,
    /// What could not be represented, as a noun phrase (e.g. `"image URLs"`).
    pub content: String,
}

impl UnsupportedContent {
    /// Constructs a new [`UnsupportedContent`].
    ///
    /// `content` is rendered after "{provider} does not support ", so it reads best as a plural
    /// noun phrase: `UnsupportedContent::new("Gemini", "image URLs")`.
    pub fn new(provider: &'static str, content: impl Into<String>) -> Self {
        Self {
            provider,
            content: content.into(),
        }
    }
}

/// Why a response stopped before the model finished its output.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[error("model response truncated: {}", .reason.as_deref().unwrap_or("no details provided"))]
pub struct Truncation {
    reason: Option<String>,
}

impl Truncation {
    /// Constructs a new [`Truncation`] with the given reason.
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: Some(reason.into()),
        }
    }

    /// Constructs a [`Truncation`] for a response that is truncated without a reason.
    pub fn unspecified() -> Self {
        Self { reason: None }
    }

    /// Constructs a [`Truncation`] for a response that hit the output token limit.
    pub fn max_tokens() -> Self {
        Self::new("output token limit reached")
    }
}

/// An error that is returned when a truncated response cannot be resumed.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ContinuationError {
    /// The request sets a response schema, so the model cannot resume the truncated output.
    #[error("continuation is pointless for structured output; increase `max_tokens` instead")]
    StructuredOutput,
    /// The provider does not implement continuation.
    #[error("continuation not supported")]
    Unsupported,
    /// Reason not covered by other variants.
    #[error("continuation failed: {0}")]
    Other(String),
}
