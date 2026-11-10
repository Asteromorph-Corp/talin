//! A strongly-typed Rust client and type definitions for the [OpenAI Chat Completions API].
//!
//! The types are re-exported as-is from [`async_openai`]; the client is also built on top of
//! [`async_openai::Client`].
//!
//! [OpenAI Chat Completions API]: https://developers.openai.com/api/reference/chat-completions/overview

#[cfg(feature = "client")]
pub use async_openai::config::{AzureConfig, Config, OpenAIConfig};

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "client")]
mod error;

#[cfg(feature = "client")]
mod stream;

/// Native [`async_openai`] chat-completions types used in requests, responses, and options.
pub mod types;
