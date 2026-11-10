mod agent;
mod common;
mod from_config;
mod load;
mod mcp_server;
mod merge;
mod model;
mod prompt;
mod tool;
mod validate;

pub(crate) use agent::*;
pub(crate) use common::*;
pub(crate) use from_config::*;
pub(crate) use load::*;
pub(crate) use mcp_server::*;
pub(crate) use merge::Merge;
pub(crate) use model::*;
pub(crate) use prompt::*;
pub(crate) use tool::*;
pub(crate) use validate::*;

pub(crate) struct Configs {
    pub agents: std::collections::BTreeMap<String, Agent>,
    pub models: std::collections::BTreeMap<String, Model>,
    pub tools: std::collections::BTreeMap<String, Tool>,
    pub servers: std::collections::BTreeMap<String, McpServer>,
}
