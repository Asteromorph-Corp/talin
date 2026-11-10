#![allow(dead_code)]

use talin::chain::{ChainOutput, Ctor};

#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema)]
#[serde(deny_unknown_fields)]
pub struct CustomOutputAgentOutput {
    /// The user's full name
    pub name: String,
    /// The user's location
    pub location: String,
    /// The user's email address
    pub email: String,
}
