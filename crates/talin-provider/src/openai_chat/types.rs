//! The `types` module define types used in OpenAI chat completions API requests and responses.
//!
//! The types are re-exported from the [`async_openai`] crate, except for [`ChatRequest`] which
//! extends the [`CreateChatCompletionRequest`] with additional fields for "OpenAI-compatible" APIs.
//!
//! [`ChatRequest`]: crate::openai_chat::types::ChatRequest
//! [`CreateChatCompletionRequest`]: async_openai::types::chat::CreateChatCompletionRequest

mod request;

pub use async_openai::types::chat::*;
pub use request::ChatRequest;
