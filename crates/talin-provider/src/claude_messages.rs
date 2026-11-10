//! A strongly-typed Rust client and type definitions for the [Anthropic Claude Messages API].
//!
//! The types are regularly checked against the [official documentation] to make sure they
//! remain up-to-date and accurate.
//!
//! [Anthropic Claude Messages API]: https://docs.anthropic.com/en/api/messages
//! [official documentation]: https://platform.claude.com/docs/en/api/messages.md

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "client")]
mod config;
#[cfg(all(feature = "client", feature = "vertex"))]
pub use config::VertexConfig;
#[cfg(feature = "client")]
pub use config::{AnthropicConfig, ClientConfig, beta_headers};

#[cfg(feature = "client")]
mod stream;

#[cfg(test)]
mod spec;

pub mod types;
