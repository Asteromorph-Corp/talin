use std::str::FromStr;

use crate::error::PromptError;

/// A reference to one of the *base* (non-override) agent prompts.
///
/// Only valid inside a per-model-type prompt override, where it expands to the
/// fully-resolved content of the agent's default prompt. This lets an override
/// wrap or extend the original prompt instead of restating it, e.g.
/// `system_prompt = "{{base.system_prompt}}\n\nAdditionally, ..."`.
///
/// Unlike [`RuntimeVariable`](crate::config::RuntimeVariable)s, base variables
/// are substituted at build time and never surface as generated fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BaseVariable {
    SystemPrompt,
    UserPrompt,
}

impl BaseVariable {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            BaseVariable::SystemPrompt => "system_prompt",
            BaseVariable::UserPrompt => "user_prompt",
        }
    }
}

impl FromStr for BaseVariable {
    type Err = PromptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "system_prompt" => Ok(BaseVariable::SystemPrompt),
            "user_prompt" => Ok(BaseVariable::UserPrompt),
            other => Err(PromptError::UnknownBaseVariable(other.to_owned())),
        }
    }
}
