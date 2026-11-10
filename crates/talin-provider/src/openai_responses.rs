//! A strongly-typed Rust client and type definitions for the [OpenAI Responses API].
//!
//! Most types are re-exported from [`async_openai`]; the client is also built on top of
//! [`async_openai::Client`]. The native compaction response uses input items so retained user
//! messages can round-trip alongside the compaction item.
//!
//! [OpenAI Responses API]: https://developers.openai.com/api/reference/responses/overview

#[cfg(feature = "client")]
pub use async_openai::config::{AzureConfig, Config, OpenAIConfig};

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::Client;

mod compaction;

/// Native [`async_openai`] Responses-API types used in requests, responses, and options.
pub mod types {
    pub use async_openai::types::mcp::*;
    pub use async_openai::types::responses::*;

    pub use super::compaction::CompactResource;
}
