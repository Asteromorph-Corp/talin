use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::agent::AgentBuiltinTools;
use crate::config::{
    AgentBuiltinTool, AgentLocalTool, AgentMcpToolGrouped, AgentTool, AgentToolbox, CallOptions,
    FromConfig, Merge, ModelType, Prompt,
};
use crate::error::AgentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentOverrides<T> {
    pub model_type: BTreeMap<ModelType, T>,
}

impl<T> Default for AgentOverrides<T> {
    fn default() -> Self {
        Self {
            model_type: BTreeMap::default(),
        }
    }
}

/// A single model type's override, fully resolved and ready for codegen.
///
/// Each field is the value to substitute for the agent's default *in the
/// per-provider `AgentProfile` impl for this model type*. A `None`/empty
/// field means "no override — fall through to the trait default". This is the
/// resolved counterpart of [`RawAgentOverrideValues`]: prompts have been
/// resolved to their final content and tools partitioned by kind.
#[derive(Debug, Clone, Default)]
pub(crate) struct AgentOverrideValues {
    /// Resolved system prompt content. `None` ⇒ use the default `system_prompt()`.
    pub system_prompt: Option<String>,
    /// Resolved user prompt content. `None` ⇒ use the default `user_prompt()`.
    pub user_prompt: Option<String>,
    /// Override for `AgentOptions::max_iterations`.
    pub max_iterations: Option<usize>,
    /// Override for `AgentOptions::max_consecutive_fails`.
    pub max_consecutive_fails: Option<usize>,
    /// Override for `AgentOptions::max_continuations`.
    pub max_continuations: Option<usize>,
    /// Extra local tools to append to the default tool set for this provider.
    pub local_tools: Vec<AgentLocalTool>,
    /// Extra provider built-in tools to append for this provider.
    pub builtin_tools: Vec<AgentBuiltinTool>,
    /// Extra MCP servers (with their allowed tools) to append to the default servers for this
    /// provider.
    pub mcp_servers: Vec<AgentMcpToolGrouped>,
    /// Call options for the model.
    pub call_options: Option<CallOptions>,
}

impl AgentOverrideValues {
    /// Whether this override changes the (non-builtin) tool set.
    pub(crate) fn overrides_tools(&self) -> bool {
        !self.local_tools.is_empty()
    }

    /// Whether this override changes the MCP server set.
    pub(crate) fn overrides_mcp_servers(&self) -> bool {
        !self.mcp_servers.is_empty()
    }

    /// Whether this override changes any agent option.
    pub(crate) fn overrides_options(&self) -> bool {
        self.max_iterations.is_some()
            || self.max_consecutive_fails.is_some()
            || self.max_continuations.is_some()
    }
}

/// The parsed prompts of an override, the input to prompt resolution.
///
/// Built directly from a [`RawAgentOverrideValues`]; once resolved, the prompt
/// content lands in [`AgentOverrideValues`].
pub(crate) struct AgentPromptsOverride {
    pub system_prompt: Option<Prompt>,
    pub user_prompt: Option<Prompt>,
    pub prompts: BTreeMap<String, Prompt>,
}

impl AgentPromptsOverride {
    /// Parses the override's prompt strings into [`Prompt`]s for resolution.
    pub(crate) fn from_raw(raw: &RawAgentOverrideValues) -> Result<Self, AgentError> {
        let system_prompt = raw
            .system_prompt
            .clone()
            .map(Prompt::from_config)
            .transpose()
            .map_err(AgentError::invalid_system_prompt)?;
        let user_prompt = raw
            .user_prompt
            .clone()
            .map(Prompt::from_config)
            .transpose()
            .map_err(AgentError::invalid_user_prompt)?;
        let prompts = raw
            .prompts
            .iter()
            .map(|(key, value)| match Prompt::from_config(value.clone()) {
                Ok(prompt) => Ok((key.clone(), prompt)),
                Err(e) => Err(AgentError::InvalidPrompt(format!("prompts.{key}"), e)),
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            system_prompt,
            user_prompt,
            prompts,
        })
    }
}

/// Raw, deserialized per-model-type override (the `[agent.if.model_type.<provider>]`
/// table). Merged during inheritance, then resolved into [`AgentOverrideValues`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawAgentOverrideValues {
    pub system_prompt: Option<String>,
    pub user_prompt: Option<String>,
    pub max_iterations: Option<usize>,
    pub max_consecutive_fails: Option<usize>,
    pub max_continuations: Option<usize>,
    #[serde(default)]
    pub tools: Vec<AgentTool>,
    #[serde(default)]
    pub toolboxes: Vec<AgentToolbox>,
    pub builtin_tools: Option<AgentBuiltinTools>,
    #[serde(default)]
    pub prompts: BTreeMap<String, String>,
    /// `model` is the legacy name for this field.
    #[serde(alias = "model")]
    pub call_options: Option<CallOptions>,
}

impl Merge<Self> for AgentOverrides<RawAgentOverrideValues> {
    fn merge(self, other: Self) -> Self {
        let mut overrides = BTreeMap::new();
        for model_type in ModelType::ALL {
            let values = self.model_type.get(&model_type).cloned();
            let other = other.model_type.get(&model_type).cloned();
            if let Some(values) = values.merge(other) {
                overrides.insert(model_type, values);
            }
        }
        Self {
            model_type: overrides,
        }
    }
}

impl Merge<Self> for RawAgentOverrideValues {
    fn merge(self, other: Self) -> Self {
        Self {
            system_prompt: self.system_prompt.or(other.system_prompt),
            user_prompt: self.user_prompt.or(other.user_prompt),
            max_iterations: self.max_iterations.or(other.max_iterations),
            max_consecutive_fails: self.max_consecutive_fails.or(other.max_consecutive_fails),
            max_continuations: self.max_continuations.or(other.max_continuations),
            tools: self.tools.merge(other.tools),
            toolboxes: self.toolboxes.merge(other.toolboxes),
            builtin_tools: self.builtin_tools.or(other.builtin_tools),
            prompts: self.prompts.merge(other.prompts),
            call_options: self.call_options.merge(other.call_options),
        }
    }
}
