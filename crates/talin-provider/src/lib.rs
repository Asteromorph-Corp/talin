//! A strongly-typed Rust client and type definitions for LLM provider APIs.
//!
//! Different providers use different API formats for their request / response payloads. This crate
//! provides the Rust types for different providers.
//!
//! While it is possible to use this crate directly, it is not designed with that in mind. Instead,
//! it is recommended to use the `talin` crate, which provides the unified interface for
//! different providers built on top of this crate.

#![allow(clippy::enum_variant_names)] // The variant names must match the provider's API names and are not under our control.

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::ApiClient;

#[cfg(feature = "client")]
mod provider;
#[cfg(feature = "client")]
pub use provider::{Provider, ProviderError};

#[cfg(feature = "client")]
pub mod error;

#[cfg(feature = "client")]
mod stream;
#[cfg(feature = "client")]
pub use stream::ResponseStream;

#[cfg(feature = "vertex")]
pub mod vertex;

pub mod claude_messages;
pub mod gemini_generate_content;
pub mod gemini_interactions;
pub mod openai_chat;
pub mod openai_responses;
pub mod sglang_generate;
pub mod xai_responses;

#[cfg(test)]
mod spec;
mod utils;
