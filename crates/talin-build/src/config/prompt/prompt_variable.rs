use std::str::FromStr;

use heck::ToSnakeCase;

use crate::config::{BaseVariable, InputVariable, PresetVariable};
use crate::error::PromptError;

#[derive(Debug, Clone)]
pub(crate) enum PromptVariable {
    Input(Box<InputVariable>),
    Config(String),
    Preset(PresetVariable),
    Env(String),
    Base(BaseVariable),
}

impl PromptVariable {
    /// The canonical name of the variable.
    ///
    /// Non-input variables are prefixed with an underscore to avoid name collisions.
    pub(crate) fn name(&self) -> String {
        match self {
            PromptVariable::Input(var) => var.ident.to_string(),
            PromptVariable::Config(name) => format!("_config_{name}"),
            PromptVariable::Preset(var) => format!("_preset_{}", var.name()),
            PromptVariable::Env(name) => format!("_env_{}", name.to_snake_case()),
            PromptVariable::Base(var) => format!("_base_{}", var.name()),
        }
    }
}

impl FromStr for PromptVariable {
    type Err = PromptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // ---------- Legacy syntax ----------
        if let Some(config_key) = s.strip_prefix('!') {
            return Ok(PromptVariable::Config(config_key.trim().to_owned()));
        }
        if let Some(preset_key) = s.strip_prefix('#') {
            return Ok(PromptVariable::Preset(preset_key.trim().parse()?));
        }

        if let Some((namespace, body)) = s.split_once('.') {
            return match namespace {
                "input" => Ok(PromptVariable::Input(Box::new(body.parse()?))),
                "config" => Ok(PromptVariable::Config(body.trim().to_owned())),
                "preset" => Ok(PromptVariable::Preset(body.trim().parse()?)),
                "env" => Ok(PromptVariable::Env(body.trim().to_owned())),
                "base" => Ok(PromptVariable::Base(body.trim().parse()?)),
                other => Err(PromptError::UnknownVariableNamespace(other.to_owned())),
            };
        }

        Ok(PromptVariable::Input(Box::new(s.parse()?)))
    }
}
