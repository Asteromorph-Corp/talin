//! # xAI Responses API Client
//!
//! A strongly-typed Rust client for the xAI [Create Response](https://docs.x.ai/docs/api-reference#create-new-response) API.
//!
//! This crate provides a convenient interface for interacting with the xAI API, supporting both
//! synchronous and streaming responses. It is designed with strict strong typing to ensure that
//! "impossible states are unrepresentable".

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

#[cfg(test)]
mod spec;

pub mod types;
