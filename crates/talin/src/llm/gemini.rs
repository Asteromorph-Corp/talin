//! Google Gemini API clients.
//!
//! Gemini exposes two distinct API surfaces, each in its own submodule:
//! - [`generate_content`]: the `generateContent` API ([`GeminiGenerateContent`]).
//! - [`interactions`]: the stateful Interactions API ([`GeminiInteractions`]), the recommended
//!   surface for new applications.
//!
//! The client structs and their option/pricing/tool types are re-exported here; the native
//! request/response types remain under [`generate_content::types`] and [`interactions::types`].

pub mod interactions;

pub use interactions::{
    GeminiInteractions, GeminiInteractionsBuilder, GeminiInteractionsRequest,
    InteractionsBuiltinTool, InteractionsOptions, InteractionsPricing,
};
