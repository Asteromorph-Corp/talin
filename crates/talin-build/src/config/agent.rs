use std::collections::BTreeMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::config::agent::agent_builtin_tool::AgentBuiltinTools;
use crate::config::agent::agent_overrides::RawAgentOverrideValues;
use crate::config::{
    CallOptions, FromConfig, InlineOrCustom, InputVariable, Merge, Model, ModelPreset, ModelType,
    Prompt, RuntimeVariable,
};
use crate::error::{AgentError, OverrideError};
use crate::resolution::resolve_prompts;

mod agent_builtin_tool;
mod agent_local_tool;
mod agent_mcp_tool;
mod agent_model;
mod agent_overrides;
mod agent_response_format;
mod agent_skill;
mod agent_tool;
mod agent_toolbox;
mod attachments;

pub(crate) use agent_builtin_tool::AgentBuiltinTool;
pub(crate) use agent_local_tool::{
    AgentLocalTool, AgentLocalToolConfig, AgentLocalToolConfigNested,
};
pub(crate) use agent_mcp_tool::{AgentMcpTool, AgentMcpToolGrouped};
pub(crate) use agent_model::{AgentModel, ModelField};
pub(crate) use agent_overrides::{AgentOverrideValues, AgentOverrides, AgentPromptsOverride};
pub(crate) use agent_response_format::AgentResponseFormat;
pub(crate) use agent_tool::AgentTool;
pub(crate) use agent_toolbox::AgentToolbox;
pub(crate) use attachments::Attachments;

pub(crate) use crate::config::agent::agent_response_format::SchemaMode;

#[derive(Debug, Clone)]
pub(crate) struct Agent {
    /// Default system prompt content (the `AgentProfile::system_prompt()` default body), if any.
    pub system_prompt: Option<String>,
    /// Default user prompt content (the `AgentProfile::user_prompt()` default body).
    pub user_prompt: String,
    pub attachments: Attachments,
    /// All runtime variables referenced by the default prompts *and* any
    /// per-model-type override prompts (overrides may only add non-input vars).
    pub variables: Vec<RuntimeVariable>,
    pub max_iterations: Option<usize>,
    pub max_consecutive_fails: Option<usize>,
    pub max_continuations: Option<usize>,
    pub local_tools: Vec<AgentLocalTool>,
    pub builtin_tools: Vec<AgentBuiltinTool>,
    /// The names of the skills the agent can load, looked up in the skills home at runtime.
    pub skills: Vec<String>,
    pub mcp_servers: Vec<AgentMcpToolGrouped>,
    #[allow(dead_code)]
    pub toolboxes: Vec<AgentToolbox>,
    pub response_format: Option<InlineOrCustom<AgentResponseFormat>>,
    pub schema_mode: SchemaMode,
    pub model: Option<(AgentModel, ModelPreset)>,
    /// Call options applied to all models via `AgentProfile::call_options()`.
    pub call_options: CallOptions,
    pub fingerprint: String,
    /// Per-model-type overrides, fully resolved and ready for codegen. Each entry
    /// is emitted as method overrides on the matching per-provider `AgentProfile` impl.
    pub overrides: BTreeMap<ModelType, AgentOverrideValues>,
}

/// Raw deserialized agent configuration from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentConfig {
    #[serde(default, rename = "abstract")]
    pub abstract_: bool,
    pub extends: Option<String>,
    #[deprecated(note = "No longer used")]
    #[serde(default)]
    pub role: String,
    pub system_prompt: Option<String>,
    pub user_prompt: Option<String>,
    #[serde(default)]
    pub attachments: Attachments,
    #[serde(default, alias = "extra_input_fields")]
    pub extra_variables: BTreeMap<String, String>,
    pub max_iterations: Option<usize>,
    pub max_consecutive_fails: Option<usize>,
    pub max_continuations: Option<usize>,
    #[serde(default)]
    pub coworkers: Vec<String>,
    #[serde(default)]
    pub tools: Vec<AgentTool>,
    #[serde(default)]
    pub toolboxes: Vec<AgentToolbox>,
    pub builtin_tools: Option<AgentBuiltinTools>,
    #[serde(default)]
    pub skills: Vec<String>,
    pub response_format: Option<InlineOrCustom<AgentResponseFormat>>,
    pub schema_mode: Option<SchemaMode>,
    #[serde(default)]
    pub custom_output: bool,
    /// The identity of the model used by `Agent::default()`.
    pub model: Option<ModelField>,
    /// Call options applied to all models.
    pub call_options: Option<CallOptions>,
    #[serde(default)]
    pub prompts: BTreeMap<String, String>,
    #[serde(default, rename = "if")]
    pub overrides: AgentOverrides<RawAgentOverrideValues>,
}

impl Agent {
    pub(crate) fn from_config(
        id: &str,
        config: AgentConfig,
        models: &BTreeMap<String, Model>,
        shared_prompts: &BTreeMap<String, Prompt>,
    ) -> Result<Self, AgentError> {
        let fingerprint = config.fingerprint();

        let system_prompt = config
            .system_prompt
            .map(Prompt::from_config)
            .transpose()
            .map_err(AgentError::invalid_system_prompt)?;
        let user_prompt = config
            .user_prompt
            .map(Prompt::from_config)
            .transpose()
            .map_err(AgentError::invalid_user_prompt)?
            .unwrap_or_else(Prompt::default_user_prompt);

        let extra_variables: Vec<InputVariable> = config
            .extra_variables
            .into_iter()
            .map(|(name, type_)| InputVariable::new(&name, Some(type_.as_str())))
            .try_collect()
            .map_err(AgentError::InvalidInputVariable)?;

        let local_prompts: BTreeMap<String, Prompt> = config
            .prompts
            .into_iter()
            .map(|(key, value)| match Prompt::from_config(value) {
                Ok(prompt) => Ok((key, prompt)),
                Err(e) => Err(AgentError::InvalidPrompt(format!("prompts.{key}"), e)),
            })
            .try_collect()?;

        let prompt_overrides = config
            .overrides
            .model_type
            .iter()
            .map(|(model_type, raw)| Ok((*model_type, AgentPromptsOverride::from_raw(raw)?)))
            .collect::<Result<BTreeMap<_, _>, AgentError>>()?;
        let resolved_prompts = resolve_prompts(
            id,
            system_prompt.as_ref(),
            &user_prompt,
            &local_prompts,
            shared_prompts,
            &extra_variables,
            prompt_overrides,
        )?;

        let (local_tools, mcp_tools, mut builtin_tools) = AgentTool::partition_tools(config.tools);
        builtin_tools.extend(config.builtin_tools.into_iter().flatten());
        let mcp_servers = AgentMcpTool::group_tools(mcp_tools)?;

        for skill in &config.skills {
            agent_skill::validate_skill_name(skill)?;
        }
        // An agent inheriting a skill its parent lists must not list it twice.
        let skills = config.skills.into_iter().unique().collect();

        let (model, call_options) = config
            .model
            .map_or((None, None), |m| (m.model, m.call_options));

        let model = match model {
            Some(model) => {
                let preset = match &model {
                    AgentModel::Inline(inline) => inline.model_preset(),
                    AgentModel::Reference(id) => models
                        .get(id)
                        .ok_or_else(|| AgentError::missing_model(id))?
                        .model_preset(),
                };
                Some((model, preset))
            }
            None => None,
        };
        let call_options = match (config.call_options, call_options) {
            (Some(explicit), Some(legacy)) => explicit.merge(legacy),
            (Some(options), None) | (None, Some(options)) => options,
            (None, None) => CallOptions::default(),
        };

        let resolved_overrides = resolve_overrides(
            resolved_prompts.system_overrides,
            resolved_prompts.user_overrides,
            config.overrides,
        )?;

        let response_format = config.response_format.or_else(|| {
            if config.custom_output {
                Some(InlineOrCustom::Custom)
            } else {
                None
            }
        });
        let response_format_schema = config.schema_mode.unwrap_or(if response_format.is_some() {
            SchemaMode::JsonSchema
        } else {
            SchemaMode::Prompt
        });

        Ok(Agent {
            system_prompt: resolved_prompts.system_prompt,
            user_prompt: resolved_prompts.user_prompt,
            variables: resolved_prompts.variables.into_iter().collect(),
            attachments: config.attachments,
            max_iterations: config.max_iterations,
            max_consecutive_fails: config.max_consecutive_fails.or(Some(5)),
            max_continuations: config.max_continuations,
            local_tools,
            mcp_servers,
            builtin_tools,
            skills,
            toolboxes: config.toolboxes,
            response_format,
            schema_mode: response_format_schema,
            model,
            call_options,
            fingerprint,
            overrides: resolved_overrides,
        })
    }
}

impl AgentConfig {
    fn fingerprint(&self) -> String {
        let mut bytes = b"agent_fingerprint:v1\0".to_vec();
        bytes.extend(serde_json::to_vec(self).expect("AgentConfig is serializable"));
        let hash = blake3::hash(&bytes);
        hash.to_hex().to_string()
    }
}

impl Merge<Self> for ModelField {
    /// Merges by part, so a child that sets only `model` keeps the parent's call options
    /// and a child that sets only `model.<option>` keeps the parent's model.
    fn merge(self, parent: Self) -> Self {
        ModelField {
            model: self.model.or(parent.model),
            call_options: self.call_options.merge(parent.call_options),
        }
    }
}

impl Merge<Self> for AgentConfig {
    fn merge(self, parent: Self) -> Self {
        #[allow(deprecated)]
        AgentConfig {
            abstract_: self.abstract_ && parent.abstract_,
            extends: None,   // Don't inherit the parent's extends
            role: self.role, // deprecated field, use child's value
            system_prompt: self.system_prompt.or(parent.system_prompt),
            user_prompt: self.user_prompt.or(parent.user_prompt),
            attachments: self.attachments.merge(parent.attachments),
            extra_variables: self.extra_variables.merge(parent.extra_variables),
            max_iterations: self.max_iterations.or(parent.max_iterations),
            max_consecutive_fails: self.max_consecutive_fails.or(parent.max_consecutive_fails),
            max_continuations: self.max_continuations.or(parent.max_continuations),
            coworkers: self.coworkers.merge(parent.coworkers),
            tools: self.tools.merge(parent.tools),
            toolboxes: self.toolboxes.merge(parent.toolboxes),
            builtin_tools: self.builtin_tools.or(parent.builtin_tools),
            skills: self.skills.merge(parent.skills),
            response_format: self.response_format.or(parent.response_format),
            schema_mode: self.schema_mode.or(parent.schema_mode),
            custom_output: self.custom_output || parent.custom_output,
            model: self.model.merge(parent.model),
            call_options: self.call_options.merge(parent.call_options),
            prompts: self.prompts.merge(parent.prompts),
            overrides: self.overrides.merge(parent.overrides),
        }
    }
}

/// Combines resolved override prompts with the raw tool/option overrides into one
/// [`AgentOverrideValues`] per model type, ready for codegen.
fn resolve_overrides(
    mut system_overrides: BTreeMap<ModelType, String>,
    mut user_overrides: BTreeMap<ModelType, String>,
    overrides: AgentOverrides<RawAgentOverrideValues>,
) -> Result<BTreeMap<ModelType, AgentOverrideValues>, AgentError> {
    overrides
        .model_type
        .into_iter()
        .map(|(model_type, raw)| {
            let system_prompt = system_overrides.remove(&model_type);
            let user_prompt = user_overrides.remove(&model_type);

            let (local_tools, mcp_tools, mut builtin_tools) = AgentTool::partition_tools(raw.tools);
            builtin_tools.extend(raw.builtin_tools.into_iter().flatten());
            let mcp_servers = AgentMcpTool::group_tools(mcp_tools)?;

            for tool in &builtin_tools {
                if tool.model_type() != model_type {
                    return Err(AgentError::InvalidOverride(
                        model_type,
                        OverrideError::BuiltinToolMismatch(tool.model_type()),
                    ));
                }
            }

            Ok((
                model_type,
                AgentOverrideValues {
                    system_prompt,
                    user_prompt,
                    max_iterations: raw.max_iterations,
                    max_consecutive_fails: raw.max_consecutive_fails,
                    max_continuations: raw.max_continuations,
                    local_tools,
                    mcp_servers,
                    builtin_tools,
                    call_options: raw.call_options,
                },
            ))
        })
        .try_collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const PROMPT: &str = "system_prompt = \"You are a helpful assistant.\"\n";

    fn parse(config: &str) -> AgentConfig {
        toml::from_str(&format!("{PROMPT}{config}")).expect("valid agent config")
    }

    /// The [`ModelField`] deserialized from the agent's `model` key.
    fn model_field(config: &AgentConfig) -> &ModelField {
        config.model.as_ref().expect("a model field")
    }

    fn reference(field: &ModelField) -> &str {
        match field.model.as_ref().expect("a model") {
            AgentModel::Reference(id) => id,
            AgentModel::Inline(_) => panic!("expected a model reference"),
        }
    }

    /// Runs [`Agent::from_config`], resolving references against a lone model
    /// named `gpt_4o_mini`. This is where legacy and explicit call options merge.
    fn build(config: &str) -> Agent {
        let mut models = BTreeMap::new();
        models.insert("gpt_4o_mini".to_owned(), crate::test_utils::model_generic());
        Agent::from_config("my_agent", parse(config), &models, &BTreeMap::new())
            .expect("valid agent")
    }

    // --- Deserialization: `model` splits into an identity and call options,
    // while `[call_options]` stays separate until `from_config` merges them. ---

    #[test]
    fn model_reference() {
        let config = parse("model = \"gpt_4o_mini\"\n");
        let field = model_field(&config);
        assert_eq!(reference(field), "gpt_4o_mini");
        assert!(field.call_options.is_none());
        assert!(config.call_options.is_none());
    }

    #[test]
    fn inline_model() {
        let config = parse(
            "[model]\napi_base = \"https://api.openai.com/v1\"\nmodel = \"gpt-4o-mini\"\ntemperature = 0.5\n",
        );
        let field = model_field(&config);
        let AgentModel::Inline(model) = field.model.as_ref().expect("a model") else {
            panic!("expected an inline model");
        };
        // An inline model's call options belong to that model, not to the agent.
        assert_eq!(model.call_options.temperature, Some(0.5));
        assert!(field.call_options.is_none());
        assert!(config.call_options.is_none());
    }

    #[test]
    fn incomplete_inline_model_is_rejected() {
        let result: Result<AgentConfig, _> = toml::from_str(&format!(
            "{PROMPT}[model]\napi_base = \"https://example.com\"\n"
        ));
        result.unwrap_err();
    }

    #[test]
    fn explicit_call_options_stay_separate_from_the_model() {
        let config = parse("model = \"gpt_4o_mini\"\n\n[call_options]\ntemperature = 0\n");
        let field = model_field(&config);
        assert_eq!(reference(field), "gpt_4o_mini");
        assert!(field.call_options.is_none());
        assert_eq!(
            config.call_options.expect("call options").temperature,
            Some(0.0)
        );
    }

    #[test]
    fn legacy_model_id_with_call_options_is_split() {
        // Deprecated syntax mixing identity and options into `model`. The options
        // ride along on the model field until `from_config` merges them.
        let config = parse("[model]\nid = \"gpt_4o_mini\"\ntemperature = 0\ntop_p = 0.5\n");
        let field = model_field(&config);
        assert_eq!(reference(field), "gpt_4o_mini");
        let call_options = field.call_options.as_ref().expect("call options");
        assert_eq!(call_options.temperature, Some(0.0));
        assert_eq!(call_options.top_p, Some(0.5));
        assert!(config.call_options.is_none());
    }

    #[test]
    fn merge_keeps_parent_call_options_when_child_sets_only_model() {
        let parent = parse("[model]\nreasoning_effort = \"high\"\n");
        let child = parse("model = \"gpt_4o_mini\"\n");
        let merged = child.merge(parent);
        let field = model_field(&merged);
        assert_eq!(reference(field), "gpt_4o_mini");
        assert!(
            field
                .call_options
                .as_ref()
                .expect("call options")
                .reasoning_effort
                .is_some()
        );
    }

    #[test]
    fn legacy_call_options_only_is_split() {
        // Used by an inheriting agent to tune the model it inherits.
        let config = parse("[model]\ntemperature = 0\n");
        let field = model_field(&config);
        assert!(field.model.is_none());
        assert_eq!(
            field
                .call_options
                .as_ref()
                .expect("call options")
                .temperature,
            Some(0.0)
        );
        assert!(config.call_options.is_none());
    }

    #[test]
    fn legacy_override_model_is_call_options() {
        let config = parse(
            "model = \"gpt_4o_mini\"\n\n[if.model_type.claude_messages.model]\ntemperature = 0\n",
        );
        let override_ = config
            .overrides
            .model_type
            .get(&ModelType::ClaudeMessages)
            .expect("an anthropic override");
        assert_eq!(
            override_
                .call_options
                .as_ref()
                .expect("call options")
                .temperature,
            Some(0.0)
        );
    }

    // --- from_config: legacy and explicit call options land on the agent. ---

    #[test]
    fn legacy_call_options_reach_the_agent() {
        let agent = build("[model]\nid = \"gpt_4o_mini\"\ntemperature = 0\ntop_p = 0.5\n");
        assert_eq!(agent.call_options.temperature, Some(0.0));
        assert_eq!(agent.call_options.top_p, Some(0.5));
    }

    #[test]
    fn explicit_call_options_win_over_legacy_ones() {
        let agent = build(
            "[call_options]\ntemperature = 1\n\n[model]\nid = \"gpt_4o_mini\"\ntemperature = 0\ntop_p = 0.5\n",
        );
        // Explicit `temperature` overrides the legacy one; `top_p` falls through.
        assert_eq!(agent.call_options.temperature, Some(1.0));
        assert_eq!(agent.call_options.top_p, Some(0.5));
    }

    // --- Skills ---

    #[rstest]
    #[case("skills = [\"pdf\", \"xlsx\"]\n", vec!["pdf", "xlsx"])]
    #[case::empty("skills = []\n", vec![])]
    fn skills_parsed(#[case] toml: &str, #[case] expected: Vec<&str>) {
        let agent = build(toml);
        assert_eq!(agent.skills, expected);
    }

    #[test]
    fn skills_inherited() {
        let parent = parse("skills = [\"pdf\", \"xlsx\"]\n");
        let child = parse("skills = [\"docx\"]\n");
        let agent = child.merge(parent);
        assert_eq!(agent.skills, ["pdf", "xlsx", "docx"]);
    }

    #[test]
    fn skills_deduplicated() {
        let config = parse("skills = [\"pdf\", \"pdf\"]\n");
        let agent = Agent::from_config("id", config, &BTreeMap::new(), &BTreeMap::new()).unwrap();
        assert_eq!(agent.skills, ["pdf"]);
    }
}
