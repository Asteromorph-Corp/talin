//! The `types` module defines the types used in Claude Messages API requests and responses.
//!
//! The types are based on the [Claude API spec](https://docs.claude.com/en/api/messages).

mod cache_control;
mod citation;
mod compaction;
mod container;
mod content;
mod context_management;
mod count_tokens;
mod diagnostics;
mod error_response;
mod metadata;
mod model;
mod output_config;
mod request;
mod response;
mod service_tier;
mod stop_details;
mod stop_reason;
mod stream_response;
mod text_or_items;
mod thinking;
mod tool;
mod tool_choice;
mod usage;

pub use cache_control::*;
pub use citation::*;
pub use compaction::*;
pub use container::*;
pub use content::*;
pub use context_management::*;
pub use count_tokens::*;
pub use diagnostics::*;
pub use error_response::*;
pub use metadata::*;
pub use model::*;
pub use output_config::*;
pub use request::*;
pub use response::*;
pub use service_tier::*;
pub use stop_details::*;
pub use stop_reason::*;
pub use stream_response::*;
pub use text_or_items::*;
pub use thinking::*;
pub use tool::*;
pub use tool_choice::*;
pub use usage::*;

#[cfg(feature = "mcp-client-2025-11-20")]
mod mcp_server;

#[cfg(feature = "mcp-client-2025-11-20")]
pub use mcp_server::*;
