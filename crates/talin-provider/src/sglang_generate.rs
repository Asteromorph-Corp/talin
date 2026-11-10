//! A strongly-typed Rust client and type definitions for the SGLang Generate API.
//!
//! The API documentation is lacking for the SGLang API, so the type definitions may be inaccurate.

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{Api, Client};

#[cfg(feature = "client")]
mod config;
#[cfg(feature = "client")]
pub use config::ClientConfig;

pub mod types;
