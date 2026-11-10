//! A strongly-typed Rust client and type definitions for the [Gemini Generate Content API].
//!
//! [Gemini Generate Content API]: https://ai.google.dev/api/generate-content

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{Api, Client};

#[cfg(feature = "client")]
mod config;
#[cfg(feature = "client")]
pub use config::ClientConfig;

#[cfg(feature = "client")]
mod stream;

pub mod types;
