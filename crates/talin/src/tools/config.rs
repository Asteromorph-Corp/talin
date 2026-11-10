//! Per-agent configuration for tools.
//!
//! # Example
//!
//! ```rust
//! # use async_trait::async_trait;
//! # use talin::tools::{FromConfig, ConfigSchema};
//! #
//! # struct MyClient {
//! #     api_key: String,
//! # }
//! #
//! # struct MyTool {
//! #     client: MyClient,
//! # }
//! #
//! #[derive(Default, ConfigSchema)]
//! struct MyToolConfig {
//!     api_key: String,
//! }
//!
//! #[async_trait]
//! impl FromConfig for MyTool {
//!     type Config = MyToolConfig;
//!
//!     fn from_config(config: Self::Config) -> Self {
//!         let client = MyClient {
//!             api_key: config.api_key,
//!         };
//!         Self { client }
//!     }
//! }
//! ```

pub(crate) mod assign;
pub(crate) mod probe;
pub(crate) mod schema;

pub use schema::{ConfigSchema, SchemaDef, SchemaField};

/// A tool that can be constructed from a configuration type.
pub trait FromConfig {
    /// The configuration type.
    type Config: Default;

    /// Creates the tool from the given configuration.
    fn from_config(config: Self::Config) -> Self;
}
