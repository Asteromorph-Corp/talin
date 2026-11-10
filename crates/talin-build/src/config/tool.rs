use std::convert::Infallible;

use serde::Deserialize;

use crate::config::{FromConfig, InlineOrCustom, StructDef};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Tool {
    /// Optional if the tool defines `fn name` in its `#[tool]` block.
    pub name: Option<String>,
    /// Optional if the tool defines `fn description` in its `#[tool]` block.
    pub description: Option<String>,
    #[serde(alias = "fields")]
    pub input: InlineOrCustom<StructDef>,
    pub config: Option<ToolConfigDefinition>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ToolConfigDefinition(pub InlineOrCustom<StructDef>);

impl FromConfig for Tool {
    type Config = Self;
    type Error = Infallible;

    fn from_config(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(config)
    }
}
