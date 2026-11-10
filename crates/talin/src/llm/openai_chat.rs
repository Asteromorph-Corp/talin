//! OpenAI chat completions API client.

mod conversion;
mod generic;
mod generic_builder;
mod helper;
mod openai;
mod openai_builder;
mod options;
mod pricing;

pub use generic::{GenericChat, GenericChatMessage};
pub use generic_builder::GenericChatBuilder;
pub use openai::OpenAIChat;
pub use openai_builder::OpenAIChatBuilder;
pub use options::ChatOptions;
pub use pricing::ChatPricing;

/// Native `async_openai` chat-completions types used in requests, responses, and options.
///
/// Re-exported so that request/response manipulation (e.g. in custom strategies) and
/// [`ChatOptions`] configuration can name these types without depending on `async_openai`
/// directly.
///
/// [`async_openai`]: https://crates.io/crates/async_openai
pub mod types {
    pub use talin_provider::openai_chat::types::*;
}
