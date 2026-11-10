use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::tagged_enum;

/// Request-level diagnostics. Currently carries the previous response id for prompt-cache
/// divergence reporting.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct DiagnosticsParam {
    /// The `id` (`msg_...`) from this client's previous Messages response.
    ///
    /// The server compares that request's prompt fingerprint against this one and returns
    /// [`Diagnostics::cache_miss_reason`] when the prompt-cache prefix could not be reused. Pass
    /// `null` on the first turn to opt in without a prior message to compare.
    ///
    /// Must be at most 256 characters long.
    pub previous_message_id: Option<String>,
}

impl DiagnosticsParam {
    /// Creates a new [`DiagnosticsParam`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`previous_message_id`](Self::previous_message_id) to compare against.
    pub fn with_previous_message_id(mut self, previous_message_id: impl Into<String>) -> Self {
        self.previous_message_id = Some(previous_message_id.into());
        self
    }
}

/// Request-level diagnostics: why the prompt cache could not fully reuse the prefix of the request
/// named by [`DiagnosticsParam::previous_message_id`].
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Diagnostics {
    /// Explains why the prompt cache could not fully reuse the prefix from the request identified
    /// by [`DiagnosticsParam::previous_message_id`].
    ///
    /// `None` means diagnosis is still pending: the response was serialized before the background
    /// comparison completed.
    #[cfg_attr(test, schema(required = true))]
    pub cache_miss_reason: Option<CacheMissReason>,
}

tagged_enum! {
    /// Why the prompt cache could not fully reuse the prefix of the previous request.
    #[derive(Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum CacheMissReason {
        /// The model differs from the previous request.
        ModelChanged(CacheMissedInputTokens),
        /// The system prompt differs from the previous request.
        SystemChanged(CacheMissedInputTokens),
        /// The tools differ from the previous request.
        ToolsChanged(CacheMissedInputTokens),
        /// The messages differ from the previous request.
        MessagesChanged(CacheMissedInputTokens),
        /// The previous request could not be found.
        PreviousMessageNotFound {},
        /// The diagnosis is unavailable.
        Unavailable {},
    }
}

/// The size of a prompt-cache miss.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CacheMissedInputTokens {
    /// Approximate number of input tokens that would have been read from cache had the prefix
    /// matched the previous request.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub cache_missed_input_tokens: u32,
}
