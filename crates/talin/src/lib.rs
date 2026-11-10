//! # Talin
//!
//! A Rust library for building AI agent systems with collaborative capabilities,
//! advanced strategies, and tool integration.
//!
//! This crate provides:
//! - **Agent System**: Build agents that can collaborate with each other using coworkers
//! - **Strategies**: Various execution strategies for agents including diary tracking, delegation,
//!   and tagging
//! - **Tools**: Scaffold for building custom tools, and pre-built collaboration tools

mod any;
mod macros;
mod text;
mod utils;

// Lets derive macros emit `::talin::…` paths inside this crate too.
extern crate self as talin;

pub use talin_macros::tool;

pub mod agent;
pub mod chain;
pub mod llm;
pub mod mcp;
pub mod schemas;
pub mod skill;
pub mod strategy;
pub mod template;
pub mod tools;
pub mod transcript;

#[doc(hidden)]
pub mod __private {
    pub use async_trait;
    pub use chrono;
    pub use log;
    pub use reqwest;
    pub use schemars;
    pub use serde;
    pub use serde_json;

    pub use crate::navigate;
    pub use crate::text::parse::macro_support::{ParseError, extract_jsons, parse_json};
    pub use crate::tools::config::assign::{
        MapInsertElem, PeelOption, SetOption, VecPushElem, Wrapper,
    };
    pub use crate::tools::config::probe::{ConfigProbe, ConfigProbeFallback};
    pub use crate::tools::config::schema::Supplied;

    /// Type alias helper for macro expansion.
    pub type Type<T> = T;
}
