use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexSet;

use crate::config::{
    AgentPromptsOverride, BaseVariable, InputVariable, ModelType, Prompt, RuntimeVariable,
};
use crate::error::{AgentError, OverrideError, PromptResolutionError};
use crate::resolution::collect_prompts;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPrompts {
    pub system_prompt: Option<String>,
    pub system_overrides: BTreeMap<ModelType, String>,
    pub user_prompt: String,
    pub user_overrides: BTreeMap<ModelType, String>,
    pub variables: IndexSet<RuntimeVariable>,
}
/// A single resolved prompt together with the variables it references.
#[derive(Debug, Clone, Default)]
struct ResolvedPrompt {
    pub content: String,
    pub variables: IndexSet<RuntimeVariable>,
}

/// The fully-resolved base (non-override) prompts, used to expand
/// `base.system_prompt` / `base.user_prompt` references inside overrides.
#[derive(Debug, Clone, Default)]
struct BasePrompts {
    system: ResolvedPrompt,
    user: ResolvedPrompt,
}

impl BasePrompts {
    fn get(&self, var: BaseVariable) -> &ResolvedPrompt {
        match var {
            BaseVariable::SystemPrompt => &self.system,
            BaseVariable::UserPrompt => &self.user,
        }
    }
}

/// Prompt overrides for every model type, keyed by [`ModelType`].
#[derive(Debug, Default)]
struct PromptOverrides {
    system: BTreeMap<ModelType, String>,
    user: BTreeMap<ModelType, String>,
    /// The union of all variables introduced by any of the overrides.
    ///
    /// This may include new environment variables or other non-input variables, but will never
    /// include new input variables.
    variables: IndexSet<RuntimeVariable>,
}

/// Prompt overrides for a single model type.
#[derive(Debug, Default)]
struct PromptOverride {
    system: Option<String>,
    user: Option<String>,
    /// Additiona variables introduced by the override.
    ///
    /// This may include new environment variables or other non-input variables, but will never
    /// include new input variables.
    variables: IndexSet<RuntimeVariable>,
}

pub(crate) fn resolve_prompts(
    id: &str,
    system_prompt: Option<&Prompt>,
    user_prompt: &Prompt,
    local_prompts: &BTreeMap<String, Prompt>,
    shared_prompts: &BTreeMap<String, Prompt>,
    extra_variables: &[InputVariable],
    prompt_overrides: BTreeMap<ModelType, AgentPromptsOverride>,
) -> Result<ResolvedPrompts, AgentError> {
    let prompts = collect_prompts(id, shared_prompts, local_prompts)?;

    // Base prompts are resolved without a base context: `base.*` references are
    // rejected here so the base prompts cannot reference themselves.
    let system_prompt = system_prompt
        .map(|prompt| resolve_prompt(prompt, &prompts, None))
        .transpose()
        .map_err(AgentError::system_prompt_resolution_failed)?;
    let user_prompt = resolve_prompt(user_prompt, &prompts, None)
        .map_err(AgentError::user_prompt_resolution_failed)?;

    // Without a system prompt, `base.system_prompt` expands to nothing inside overrides.
    let base = BasePrompts {
        system: system_prompt.clone().unwrap_or_default(),
        user: user_prompt.clone(),
    };

    let variables = collect_variables(
        [
            system_prompt
                .as_ref()
                .map(|p| p.variables.clone())
                .unwrap_or_default(),
            user_prompt.variables,
        ],
        extra_variables,
    );

    let overrides =
        resolve_prompt_overrides(id, &prompt_overrides, shared_prompts, &variables, &base)?;

    let variables = variables
        .into_iter()
        .chain(overrides.variables)
        .collect::<IndexSet<_>>();

    Ok(ResolvedPrompts {
        system_prompt: system_prompt.map(|p| p.content),
        system_overrides: overrides.system,
        user_prompt: user_prompt.content,
        user_overrides: overrides.user,
        variables,
    })
}

/// Resolves prompt overrides for all model types.
fn resolve_prompt_overrides(
    id: &str,
    prompt_overrides: &BTreeMap<ModelType, AgentPromptsOverride>,
    shared_prompts: &BTreeMap<String, Prompt>,
    existing_variables: &IndexSet<RuntimeVariable>,
    base: &BasePrompts,
) -> Result<PromptOverrides, AgentError> {
    let mut resolved = PromptOverrides::default();

    for model_type in ModelType::ALL {
        let Some(prompt_override) = prompt_overrides.get(&model_type) else {
            continue;
        };

        let prompt_override = resolve_prompt_override(
            id,
            model_type,
            prompt_override,
            shared_prompts,
            existing_variables,
            base,
        )?;

        if let Some(system) = prompt_override.system {
            resolved.system.insert(model_type, system);
        }
        if let Some(user) = prompt_override.user {
            resolved.user.insert(model_type, user);
        }
        resolved.variables.extend(prompt_override.variables);
    }

    Ok(resolved)
}

/// Resolves the system and user prompt overrides for a single variant.
fn resolve_prompt_override(
    id: &str,
    model_type: ModelType,
    prompt_override: &AgentPromptsOverride,
    shared_prompts: &BTreeMap<String, Prompt>,
    existing_variables: &IndexSet<RuntimeVariable>,
    base: &BasePrompts,
) -> Result<PromptOverride, AgentError> {
    let prompts = collect_prompts(id, shared_prompts, &prompt_override.prompts)?;

    let mut resolved = PromptOverride::default();

    if let Some(sp) = &prompt_override.system_prompt {
        let prompt = resolve_override_prompt(
            model_type,
            "system",
            sp,
            &prompts,
            existing_variables,
            base,
            AgentError::system_prompt_resolution_failed,
        )?;
        resolved.variables.extend(prompt.variables);
        resolved.system = Some(prompt.content);
    }

    if let Some(up) = &prompt_override.user_prompt {
        let prompt = resolve_override_prompt(
            model_type,
            "user",
            up,
            &prompts,
            existing_variables,
            base,
            AgentError::user_prompt_resolution_failed,
        )?;
        resolved.variables.extend(prompt.variables);
        resolved.user = Some(prompt.content);
    }

    Ok(resolved)
}

fn resolve_override_prompt<'a>(
    model_type: ModelType,
    label: &'static str,
    prompt_source: &'a Prompt,
    prompts: &BTreeMap<&str, &'a Prompt>,
    existing_variables: &IndexSet<RuntimeVariable>,
    base: &BasePrompts,
    map_err: fn(PromptResolutionError) -> AgentError,
) -> Result<ResolvedPrompt, AgentError> {
    let prompt = resolve_prompt(prompt_source, prompts, Some(base)).map_err(map_err)?;
    let new_vars = get_new_variables(existing_variables, &prompt.variables);
    if let Some(new_input_var) = new_vars.iter().find_map(|var| match var {
        RuntimeVariable::Input(input_var) => Some(input_var),
        _ => None,
    }) {
        return Err(AgentError::invalid_override(
            model_type,
            OverrideError::IntroduceVariable(label, new_input_var.ident.to_string()),
        ));
    }

    Ok(ResolvedPrompt {
        content: prompt.content,
        variables: new_vars,
    })
}

fn resolve_prompt<'a>(
    prompt: &'a Prompt,
    prompts: &BTreeMap<&str, &'a Prompt>,
    base: Option<&BasePrompts>,
) -> Result<ResolvedPrompt, PromptResolutionError> {
    let content = prompt.content.clone();
    let (variables, config_variables, base_variables) = prompt.split_variables();
    let variables = variables.into_iter().collect();

    let prompt = ResolvedPrompt { content, variables };
    let prompt = resolve_base_variables(prompt, &base_variables, base)?;

    let mut visiting = BTreeSet::new();
    let prompt = resolve_keys(prompt, prompts, base, &config_variables, &mut visiting)?;

    Ok(prompt)
}

fn resolve_keys<'a>(
    mut prompt: ResolvedPrompt,
    prompts: &BTreeMap<&str, &'a Prompt>,
    base: Option<&BasePrompts>,
    keys: &[&'a str],
    visiting: &mut BTreeSet<&'a str>,
) -> Result<ResolvedPrompt, PromptResolutionError> {
    for key in keys {
        prompt = resolve_key(prompt, prompts, base, key, visiting)?;
    }
    Ok(prompt)
}

fn resolve_key<'a>(
    mut prompt: ResolvedPrompt,
    prompts: &BTreeMap<&str, &'a Prompt>,
    base: Option<&BasePrompts>,
    key: &'a str,
    visiting: &mut BTreeSet<&'a str>,
) -> Result<ResolvedPrompt, PromptResolutionError> {
    if visiting.contains(&key) {
        return Err(PromptResolutionError::CyclicDependency);
    }

    let Some(value) = prompts.get(key) else {
        return Err(PromptResolutionError::MissingConfigVariable(key.to_owned()));
    };

    let (variables, config_variables, base_variables) = value.split_variables();
    prompt.content = prompt
        .content
        .replace(&format!("{{{{_config_{key}}}}}"), &value.content);
    prompt.variables.extend(variables);
    let prompt = resolve_base_variables(prompt, &base_variables, base)?;

    visiting.insert(key);
    let out = resolve_keys(prompt, prompts, base, &config_variables, visiting)?;
    visiting.remove(key);

    Ok(out)
}

/// Expands `base.*` references into the corresponding resolved base prompt.
///
/// Base prompts are already fully resolved, so their content (which may still
/// contain runtime `{{name}}` placeholders) is substituted verbatim and the
/// variables it references are pulled in. Outside an override `base` is `None`,
/// so any `base.*` reference is rejected.
fn resolve_base_variables(
    mut prompt: ResolvedPrompt,
    base_variables: &[BaseVariable],
    base: Option<&BasePrompts>,
) -> Result<ResolvedPrompt, PromptResolutionError> {
    for base_var in base_variables {
        let Some(base) = base else {
            return Err(PromptResolutionError::BaseVariableNotAllowed(
                base_var.name(),
            ));
        };
        let resolved = base.get(*base_var);
        prompt.content = prompt.content.replace(
            &format!("{{{{_base_{}}}}}", base_var.name()),
            &resolved.content,
        );
        prompt.variables.extend(resolved.variables.iter().cloned());
    }
    Ok(prompt)
}

fn collect_variables(
    variables: [IndexSet<RuntimeVariable>; 2],
    extra_variables: &[InputVariable],
) -> IndexSet<RuntimeVariable> {
    let extra_variables = extra_variables
        .iter()
        .map(|var| RuntimeVariable::Input(Box::new(var.clone())));
    variables
        .into_iter()
        .flatten()
        .chain(extra_variables)
        .collect::<IndexSet<_>>()
}

/// Returns the variables in `variables` that are not already present in `existing`.
///
/// The input is a single prompt's (already name-deduplicated) variable list, so the
/// result is a plain [`Vec`]; cross-prompt unions are handled by callers via [`IndexSet`].
fn get_new_variables(
    existing: &IndexSet<RuntimeVariable>,
    variables: &IndexSet<RuntimeVariable>,
) -> IndexSet<RuntimeVariable> {
    variables.difference(existing).cloned().collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use indexmap::IndexSet;

    use super::*;

    macro_rules! overrides {
        (@entry $map:ident, $model_type:ident, ? $value:expr) => {
            if let Some(__val) = $value {
                $map.insert($crate::config::ModelType::$model_type, __val);
            }
        };
        (@entry $map:ident, $model_type:ident, $value:expr) => {
            $map.insert($crate::config::ModelType::$model_type, $value);
        };
        (@munch $map:ident, $(,)*) => {};
        (@munch $map:ident, $model_type:ident => ? $value:expr, $($rest:tt)*) => {
            overrides!(@entry $map, $model_type, ? $value);
            overrides!(@munch $map, $($rest)*);
        };
        (@munch $map:ident, $model_type:ident => $value:expr, $($rest:tt)*) => {
            overrides!(@entry $map, $model_type, $value);
            overrides!(@munch $map, $($rest)*);
        };
        ($($args:tt)*) => {{
            let mut __map = std::collections::BTreeMap::new();
            overrides!(@munch __map, $($args)*,);
            __map
        }};
    }

    fn parse_prompt(s: &str) -> Prompt {
        s.parse().expect("valid prompt")
    }

    fn make_override(
        system_prompt: Option<Prompt>,
        user_prompt: Option<Prompt>,
    ) -> AgentPromptsOverride {
        AgentPromptsOverride {
            system_prompt,
            user_prompt,
            prompts: BTreeMap::new(),
        }
    }

    // ── resolve_prompt ──────────────────────────────────────────

    #[test]
    fn resolve_prompt_plain_text() {
        let prompt = parse_prompt("Hello world");
        let resolved = resolve_prompt(&prompt, &BTreeMap::new(), None).unwrap();
        assert_eq!(resolved.content, "Hello world");
        assert!(resolved.variables.is_empty());
    }

    #[test]
    fn resolve_prompt_with_input_variable() {
        let prompt = parse_prompt("Hello {{input.name}}");
        let resolved = resolve_prompt(&prompt, &BTreeMap::new(), None).unwrap();
        assert_eq!(resolved.content, "Hello {{name}}");
        assert_eq!(resolved.variables.len(), 1);
        assert!(matches!(resolved.variables[0], RuntimeVariable::Input(_)));
    }

    #[test]
    fn resolve_prompt_substitutes_config_variable() {
        let prompt = parse_prompt("Hello {{config.greeting}}");
        let greeting = parse_prompt("world");
        let prompts = BTreeMap::from([("greeting", &greeting)]);

        let resolved = resolve_prompt(&prompt, &prompts, None).unwrap();
        assert_eq!(resolved.content, "Hello world");
    }

    #[test]
    fn resolve_prompt_nested_config_variables() {
        let prompt = parse_prompt("{{config.outer}}");
        let outer = parse_prompt("A {{config.inner}} B");
        let inner = parse_prompt("deep");
        let prompts = BTreeMap::from([("outer", &outer), ("inner", &inner)]);

        let resolved = resolve_prompt(&prompt, &prompts, None).unwrap();
        assert_eq!(resolved.content, "A deep B");
    }

    #[test]
    fn resolve_prompt_missing_config_variable() {
        let prompt = parse_prompt("Hello {{config.missing}}");
        let result = resolve_prompt(&prompt, &BTreeMap::new(), None);
        assert!(matches!(
            result,
            Err(PromptResolutionError::MissingConfigVariable(_))
        ));
    }

    #[test]
    fn resolve_prompt_cyclic_config_variable() {
        let prompt = parse_prompt("{{config.a}}");
        let a = parse_prompt("{{config.b}}");
        let b = parse_prompt("{{config.a}}");
        let prompts = BTreeMap::from([("a", &a), ("b", &b)]);

        let result = resolve_prompt(&prompt, &prompts, None);
        assert!(matches!(
            result,
            Err(PromptResolutionError::CyclicDependency)
        ));
    }

    // ── collect_variables ───────────────────────────────────────

    #[test]
    fn collect_variables_deduplicates() {
        let var = InputVariable::new("name", None).unwrap();
        let vars1 = IndexSet::from_iter([RuntimeVariable::Input(Box::new(var.clone()))]);
        let vars2 = IndexSet::from_iter([RuntimeVariable::Input(Box::new(var))]);

        let collected = collect_variables([vars1, vars2], &[]);
        assert_eq!(collected.len(), 1);
    }

    #[test]
    fn collect_variables_includes_extra() {
        let extras = [InputVariable::new("extra", None).unwrap()];
        let collected = collect_variables([IndexSet::new(), IndexSet::new()], &extras);
        assert_eq!(collected.len(), 1);
        assert_eq!(collected.first().unwrap().name(), "extra");
    }

    // ── get_new_input_variable ──────────────────────────────────

    #[test]
    fn get_new_input_variable_returns_none_for_existing() {
        let var = InputVariable::new("name", None).unwrap();
        let existing = IndexSet::from_iter([RuntimeVariable::Input(Box::new(var.clone()))]);
        let new = IndexSet::from_iter([RuntimeVariable::Input(Box::new(var))]);

        assert!(get_new_variables(&existing, &new).is_empty());
    }

    #[test]
    fn get_new_input_variable_detects_new() {
        let existing_var = InputVariable::new("name", None).unwrap();
        let new_var = InputVariable::new("age", Some("i32")).unwrap();
        let existing = IndexSet::from_iter([RuntimeVariable::Input(Box::new(existing_var))]);
        let new = IndexSet::from_iter([RuntimeVariable::Input(Box::new(new_var))]);

        let result = get_new_variables(&existing, &new);
        assert_eq!(result.len(), 1);
        let var = result.first().unwrap();
        assert_eq!(var.name(), "age");
    }

    #[test]
    fn get_new_input_variable_ignores_non_input() {
        let env_var = IndexSet::from_iter([RuntimeVariable::Env(String::from("HOME"))]);
        let result = get_new_variables(&IndexSet::new(), &env_var);
        assert_eq!(result.len(), 1); // Env vars are "new" but not input vars
    }

    // ── resolve_prompt_override ───────────────────────────────

    #[test]
    fn resolve_prompt_override_both_none() {
        let ext = make_override(None, None);
        let shared = BTreeMap::new();
        let existing = IndexSet::new();
        let resolved = resolve_prompt_override(
            "test_agent",
            ModelType::ClaudeMessages,
            &ext,
            &shared,
            &existing,
            &BasePrompts::default(),
        )
        .unwrap();
        assert!(resolved.system.is_none());
        assert!(resolved.user.is_none());
    }

    #[test]
    fn resolve_prompt_override_overrides_system() {
        let sp = parse_prompt("Override system");
        let ext = make_override(Some(sp), None);
        let shared = BTreeMap::new();
        let existing = IndexSet::new();
        let resolved = resolve_prompt_override(
            "test_agent",
            ModelType::ClaudeMessages,
            &ext,
            &shared,
            &existing,
            &BasePrompts::default(),
        )
        .unwrap();
        assert_eq!(resolved.system.unwrap(), "Override system");
        assert!(resolved.user.is_none());
    }

    #[test]
    fn resolve_prompt_override_overrides_both() {
        let sp = parse_prompt("Sys override");
        let up = parse_prompt("Usr override");
        let ext = make_override(Some(sp), Some(up));
        let shared = BTreeMap::new();
        let existing = IndexSet::new();
        let resolved = resolve_prompt_override(
            "test_agent",
            ModelType::ClaudeMessages,
            &ext,
            &shared,
            &existing,
            &BasePrompts::default(),
        )
        .unwrap();
        assert_eq!(resolved.system.unwrap(), "Sys override");
        assert_eq!(resolved.user.unwrap(), "Usr override");
    }

    #[test]
    fn resolve_prompt_override_rejects_new_input_variable() {
        let sp = parse_prompt("Hello {{input.new_var}}");
        let ext = make_override(Some(sp), None);
        let shared = BTreeMap::new();
        let existing = IndexSet::new();
        let result = resolve_prompt_override(
            "test_agent",
            ModelType::ClaudeMessages,
            &ext,
            &shared,
            &existing,
            &BasePrompts::default(),
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("new input variable"), "got: {err}");
    }

    #[test]
    fn resolve_prompt_override_accepts_existing_input_variable() {
        let sp = parse_prompt("Hello {{input.name}}");
        let ext = make_override(Some(sp), None);
        let shared = BTreeMap::new();
        let var = InputVariable::new("name", None).unwrap();
        let existing = IndexSet::from_iter([RuntimeVariable::Input(Box::new(var))]);
        let resolved = resolve_prompt_override(
            "test_agent",
            ModelType::ClaudeMessages,
            &ext,
            &shared,
            &existing,
            &BasePrompts::default(),
        )
        .unwrap();
        assert_eq!(resolved.system.unwrap(), "Hello {{name}}");
    }

    #[test]
    fn resolve_prompt_override_collects_non_input_variable() {
        let sp = parse_prompt("Value: {{env.HOME}}");
        let ext = make_override(Some(sp), None);
        let shared = BTreeMap::new();
        let existing = IndexSet::new();
        let resolved = resolve_prompt_override(
            "test_agent",
            ModelType::ClaudeMessages,
            &ext,
            &shared,
            &existing,
            &BasePrompts::default(),
        )
        .unwrap();
        assert_eq!(resolved.system.unwrap(), "Value: {{_env_home}}");
        assert_eq!(resolved.variables.len(), 1);
        assert!(matches!(
            resolved.variables.first().unwrap(),
            RuntimeVariable::Env(env) if env == "HOME"
        ));
    }

    // ── resolve_prompt_overrides ──────────────────────────────────

    #[test]
    fn resolve_prompt_overrides_all_empty() {
        let overrides = BTreeMap::new();
        let shared = BTreeMap::new();
        let existing = IndexSet::new();
        let resolved = resolve_prompt_overrides(
            "test_agent",
            &overrides,
            &shared,
            &existing,
            &BasePrompts::default(),
        )
        .unwrap();
        assert!(!resolved.system.contains_key(&ModelType::ClaudeMessages));
        assert!(!resolved.user.contains_key(&ModelType::ClaudeMessages));
        assert!(!resolved.system.contains_key(&ModelType::GenericChat));
        assert!(!resolved.user.contains_key(&ModelType::OpenAiChat));
    }

    #[test]
    fn resolve_prompt_overrides_single_variant() {
        let sp = parse_prompt("ClaudeMessages sys");
        let prompt_overrides = overrides!(
            ClaudeMessages => make_override(Some(sp), None)
        );

        let shared = BTreeMap::new();
        let existing = IndexSet::new();
        let resolved = resolve_prompt_overrides(
            "test_agent",
            &prompt_overrides,
            &shared,
            &existing,
            &BasePrompts::default(),
        )
        .unwrap();
        assert_eq!(
            resolved.system.get(&ModelType::ClaudeMessages).unwrap(),
            "ClaudeMessages sys"
        );
        assert!(!resolved.user.contains_key(&ModelType::ClaudeMessages));
        assert!(!resolved.system.contains_key(&ModelType::OpenAiChat));
    }

    // ── resolve_prompts (integration) ───────────────────────────

    #[test]
    fn resolve_prompts_basic() {
        let system = parse_prompt("You are helpful.");
        let user = Prompt::default_user_prompt();
        let local = BTreeMap::new();
        let shared = BTreeMap::new();

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            BTreeMap::default(),
        )
        .unwrap();
        assert_eq!(result.system_prompt.as_deref(), Some("You are helpful."));
        assert_eq!(result.user_prompt, "{{input}}");
    }

    #[test]
    fn resolve_prompts_custom_user_prompt() {
        let system = parse_prompt("System");
        let user = parse_prompt("Custom: {{input.query}}");
        let local = BTreeMap::new();
        let shared = BTreeMap::new();

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            BTreeMap::default(),
        )
        .unwrap();
        assert_eq!(result.user_prompt, "Custom: {{query}}");
        assert!(result.variables.iter().any(|v| v.name() == "query"));
    }

    #[test]
    fn resolve_prompts_with_config_variable() {
        let system = parse_prompt("Hello {{config.tone}}");
        let user = Prompt::default_user_prompt();
        let tone: Prompt = "friendly".parse().unwrap();
        let local = BTreeMap::new();
        let shared = BTreeMap::from([("tone".to_string(), tone)]);

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            BTreeMap::default(),
        )
        .unwrap();
        assert_eq!(result.system_prompt.as_deref(), Some("Hello friendly"));
    }

    #[test]
    fn resolve_prompts_with_overrides() {
        let system = parse_prompt("Default system");
        let user = Prompt::default_user_prompt();
        let claude_system = parse_prompt("Claude system");
        let openai_user = parse_prompt("OpenAI user");
        let local = BTreeMap::new();
        let shared = BTreeMap::new();

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            overrides!(
                ClaudeMessages => make_override(Some(claude_system), None),
                OpenAiChat => make_override(None, Some(openai_user)),
            ),
        )
        .unwrap();
        assert_eq!(result.system_prompt.as_deref(), Some("Default system"));
        assert_eq!(
            result.system_overrides[&ModelType::ClaudeMessages],
            "Claude system"
        );
        assert!(!result.system_overrides.contains_key(&ModelType::OpenAiChat));
        assert_eq!(result.user_overrides[&ModelType::OpenAiChat], "OpenAI user");
        assert!(
            !result
                .user_overrides
                .contains_key(&ModelType::ClaudeMessages)
        );
    }

    #[test]
    fn resolve_prompts_override_rejects_new_variable() {
        let system = parse_prompt("Default system");
        let user = Prompt::default_user_prompt();
        let gemini_system = parse_prompt("Hi {{input.sneaky}}");
        let local = BTreeMap::new();
        let shared = BTreeMap::new();

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            overrides!(
                GeminiInteractions => make_override(Some(gemini_system), None)
            ),
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("sneaky"), "got: {err}");
    }

    #[test]
    fn resolve_prompts_appends_env_variable_from_override() {
        let system = parse_prompt("Default system");
        let user = Prompt::default_user_prompt();
        let claude_system = parse_prompt("System {{env.API_KEY}}");
        let local = BTreeMap::new();
        let shared = BTreeMap::new();

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            overrides!(
                ClaudeMessages => make_override(Some(claude_system), None)
            ),
        )
        .unwrap();
        let var_names: Vec<_> = result.variables.iter().map(|v| v.name()).collect();
        assert!(
            var_names.contains(&"_env_api_key".to_string()),
            "expected env variable from override in final variables, got: {var_names:?}"
        );
    }

    #[test]
    fn resolve_prompts_deduplicates_env_variable_from_override() {
        let system = parse_prompt("Default system {{env.HOME}}");
        let user = Prompt::default_user_prompt();
        let claude_system = parse_prompt("{{env.HOME}}");
        let local = BTreeMap::new();
        let shared = BTreeMap::new();

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            overrides!(
                ClaudeMessages => make_override(Some(claude_system), None)
            ),
        )
        .unwrap();
        let env_home_count = result
            .variables
            .iter()
            .filter(|v| matches!(v, RuntimeVariable::Env(env) if env == "HOME"))
            .count();
        assert_eq!(
            env_home_count, 1,
            "env variable should be deduplicated, got {env_home_count}"
        );
    }

    // ── base variables ──────────────────────────────────────────

    #[test]
    fn resolve_prompt_rejects_base_variable_at_top_level() {
        let prompt = parse_prompt("Wrap {{base.system_prompt}}");
        let result = resolve_prompt(&prompt, &BTreeMap::new(), None);
        assert!(matches!(
            result,
            Err(PromptResolutionError::BaseVariableNotAllowed(
                "system_prompt"
            ))
        ));
    }

    #[test]
    fn resolve_prompt_substitutes_base_variable_in_override() {
        let base = BasePrompts {
            system: ResolvedPrompt {
                content: "Base system".to_string(),
                variables: IndexSet::new(),
            },
            user: ResolvedPrompt::default(),
        };
        let prompt = parse_prompt("{{base.system_prompt}}\n\nAlso be terse.");
        let resolved = resolve_prompt(&prompt, &BTreeMap::new(), Some(&base)).unwrap();
        assert_eq!(resolved.content, "Base system\n\nAlso be terse.");
    }

    #[test]
    fn resolve_prompt_base_variable_carries_base_variables() {
        let name_var = InputVariable::new("name", None).unwrap();
        let base = BasePrompts {
            system: ResolvedPrompt {
                content: "Hello {{name}}".to_string(),
                variables: IndexSet::from_iter([RuntimeVariable::Input(Box::new(name_var))]),
            },
            user: ResolvedPrompt::default(),
        };
        let prompt = parse_prompt("{{base.system_prompt}} — extended");
        let resolved = resolve_prompt(&prompt, &BTreeMap::new(), Some(&base)).unwrap();
        assert_eq!(resolved.content, "Hello {{name}} — extended");
        assert!(resolved.variables.iter().any(|v| v.name() == "name"));
    }

    #[test]
    fn resolve_prompt_base_variable_through_config() {
        let base = BasePrompts {
            system: ResolvedPrompt {
                content: "Base".to_string(),
                variables: IndexSet::new(),
            },
            user: ResolvedPrompt::default(),
        };
        let prompt = parse_prompt("{{config.wrapper}}");
        let wrapper = parse_prompt("[ {{base.system_prompt}} ]");
        let prompts = BTreeMap::from([("wrapper", &wrapper)]);
        let resolved = resolve_prompt(&prompt, &prompts, Some(&base)).unwrap();
        assert_eq!(resolved.content, "[ Base ]");
    }

    #[test]
    fn resolve_prompts_override_extends_base_system_prompt() {
        let system = parse_prompt("You are helpful.");
        let user = Prompt::default_user_prompt();
        let claude_system = parse_prompt("{{base.system_prompt}} Be concise.");
        let local = BTreeMap::new();
        let shared = BTreeMap::new();

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            overrides!(
                ClaudeMessages => make_override(Some(claude_system), None)
            ),
        )
        .unwrap();
        assert_eq!(result.system_prompt.as_deref(), Some("You are helpful."));
        assert_eq!(
            result.system_overrides[&ModelType::ClaudeMessages],
            "You are helpful. Be concise."
        );
    }

    #[test]
    fn resolve_prompts_override_base_reuses_base_input_variable() {
        // An override that wraps the base prompt inherits the base's input
        // variable without it being flagged as a newly-introduced variable.
        let system = parse_prompt("Answer {{input.question}}.");
        let user = Prompt::default_user_prompt();
        let claude_system = parse_prompt("{{base.system_prompt}} Cite sources.");
        let local = BTreeMap::new();
        let shared = BTreeMap::new();

        let result = resolve_prompts(
            "test",
            Some(&system),
            &user,
            &local,
            &shared,
            &[],
            overrides!(
                ClaudeMessages => make_override(Some(claude_system), None)
            ),
        )
        .unwrap();
        assert_eq!(
            result.system_overrides[&ModelType::ClaudeMessages],
            "Answer {{question}}. Cite sources."
        );
    }

    #[test]
    fn resolve_prompts_without_system_prompt() {
        let user = Prompt::default_user_prompt();
        let claude_system = parse_prompt("[{{base.system_prompt}}] Be concise.");

        let result = resolve_prompts(
            "test",
            None,
            &user,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            overrides!(
                ClaudeMessages => make_override(Some(claude_system), None)
            ),
        )
        .unwrap();
        assert_eq!(result.system_prompt, None);
        // `base.system_prompt` expands to nothing when the base has no system prompt.
        assert_eq!(
            result.system_overrides[&ModelType::ClaudeMessages],
            "[] Be concise."
        );
    }
}
