//! A strongly-typed Rust client and type definitions for the [Gemini Interactions API].
//!
//! The types are regularly checked against the [official OpenAPI specification] to make sure they
//! remain up-to-date and accurate.
//!
//! [Gemini Interactions API]: https://ai.google.dev/api/interactions-api
//! [official OpenAPI specification]: https://ai.google.dev/static/api/interactions.openapi.json

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{Api, Client, Interactions};

#[cfg(feature = "client")]
mod config;
#[cfg(feature = "client")]
pub use config::ClientConfig;

#[cfg(feature = "client")]
mod stream;

#[cfg(test)]
mod spec;

pub mod types;
