use std::collections::HashSet;
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::config::FromConfig;
use crate::error::PromptError;

mod base_variable;
mod input_variable;
mod preset_variable;
mod prompt_variable;
mod runtime_variable;

pub(crate) use base_variable::BaseVariable;
pub(crate) use input_variable::InputVariable;
pub(crate) use preset_variable::PresetVariable;
pub(crate) use prompt_variable::PromptVariable;
pub(crate) use runtime_variable::RuntimeVariable;

pub(crate) static VARIABLE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{\s*(?P<legacy_prefix>[!#])?(?:(?P<namespace>input|config|preset|env|base)\.)?(?P<name>[A-Za-z0-9_\-]+)(?:\s*:\s*(?P<type>[^}]+))?\s*\}\}")
        .unwrap_or_else(|e| unreachable!("Is a valid regex: {e}"))
});
pub(crate) static COMMENT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[^\S\r\n]*//[^\r\n]*").unwrap_or_else(|e| unreachable!("Is a valid regex: {e}"))
});

#[derive(Debug, Clone)]
pub(crate) struct Prompt {
    pub content: String,
    pub variables: Vec<PromptVariable>,
}

impl Prompt {
    pub(crate) fn default_user_prompt() -> Prompt {
        Prompt::from_str("{{input}}").expect("Static prompt is valid")
    }

    /// Partitions this prompt's variables by how they are resolved.
    ///
    /// Returns the [`RuntimeVariable`]s (substituted at runtime), the config
    /// variable keys (substituted from other prompts at build time), and the
    /// [`BaseVariable`]s (expanded from the base prompts at build time, only
    /// valid inside overrides).
    pub(crate) fn split_variables(&self) -> (Vec<RuntimeVariable>, Vec<&str>, Vec<BaseVariable>) {
        let mut runtime = Vec::new();
        let mut config = Vec::new();
        let mut base = Vec::new();
        for var in &self.variables {
            match var {
                PromptVariable::Input(input_var) => {
                    runtime.push(RuntimeVariable::Input(input_var.clone()))
                }
                PromptVariable::Env(env_var) => runtime.push(RuntimeVariable::Env(env_var.clone())),
                PromptVariable::Preset(preset_var) => {
                    runtime.push(RuntimeVariable::Preset(preset_var.clone()))
                }
                PromptVariable::Config(config_var) => config.push(config_var.as_str()),
                PromptVariable::Base(base_var) => base.push(*base_var),
            }
        }
        (runtime, config, base)
    }
}

impl FromStr for Prompt {
    type Err = PromptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut content = String::with_capacity(s.len());
        let mut variables = Vec::new();

        let mut last_match = 0;

        for m in VARIABLE_RE.find_iter(s) {
            let inner = s[m.start() + 2..m.end() - 2].trim();
            let variable: PromptVariable = inner.parse()?;

            content.push_str(&s[last_match..m.start()]);
            content.push_str("{{");
            content.push_str(&variable.name());
            content.push_str("}}");
            variables.push(variable);

            last_match = m.end();
        }
        content.push_str(&s[last_match..]);

        let content = COMMENT_RE.replace_all(&content, "").into_owned();

        // Deduplicate using the canonical name, keeping the first appearance
        let mut seen = HashSet::new();
        variables.retain(|var| seen.insert(var.name()));

        let prompt = Prompt { content, variables };
        Ok(prompt)
    }
}

impl FromConfig for Prompt {
    type Config = String;
    type Error = PromptError;

    fn from_config(config: String) -> Result<Self, Self::Error> {
        let prompt = config.parse()?;
        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_variables_keep_first_appearance_order() {
        let prompt: Prompt = "{{b}} {{a}} {{b}}".parse().unwrap();
        let names: Vec<_> = prompt.variables.iter().map(|v| v.name()).collect();
        assert_eq!(names, ["b", "a"]);
    }

    #[test]
    fn test_variable_re_legacy_input() {
        let input = "{{field_name}}";
        let caps = VARIABLE_RE.captures(input).unwrap();
        assert_eq!(caps.name("name").unwrap().as_str(), "field_name");

        let input = "{{field_name: i32}}";
        let caps = VARIABLE_RE.captures(input).unwrap();
        assert_eq!(caps.name("name").unwrap().as_str(), "field_name");
        assert_eq!(caps.name("type").unwrap().as_str(), "i32");

        let input = "{{foo_bar123}}";
        let caps = VARIABLE_RE.captures(input).unwrap();
        assert_eq!(caps.name("name").unwrap().as_str(), "foo_bar123");

        let input = "{{foo: Option<String>}}";
        let caps = VARIABLE_RE.captures(input).unwrap();
        assert_eq!(caps.name("name").unwrap().as_str(), "foo");
        assert_eq!(caps.name("type").unwrap().as_str(), "Option<String>");
    }

    #[test]
    fn test_variable_re_legacy_config() {
        let a = "{{!model_name}}";
        let caps = VARIABLE_RE.captures(a).unwrap();

        assert_eq!(caps.name("legacy_prefix").unwrap().as_str(), "!");
        assert_eq!(caps.name("name").unwrap().as_str(), "model_name");
    }

    #[test]
    fn test_variable_re_legacy_preset() {
        let a = "{{#current_date}}";
        let caps = VARIABLE_RE.captures(a).unwrap();

        assert_eq!(caps.name("legacy_prefix").unwrap().as_str(), "#");
        assert_eq!(caps.name("name").unwrap().as_str(), "current_date");
    }

    #[test]
    fn test_variable_re_namespaced_input() {
        let a = "{{input.field_name}}";
        let caps = VARIABLE_RE.captures(a).unwrap();

        assert_eq!(caps.name("namespace").unwrap().as_str(), "input");
        assert_eq!(caps.name("name").unwrap().as_str(), "field_name");
    }

    #[test]
    fn test_variable_re_namespaced_config() {
        let a = "{{config.model_name}}";
        let caps = VARIABLE_RE.captures(a).unwrap();

        assert_eq!(caps.name("namespace").unwrap().as_str(), "config");
        assert_eq!(caps.name("name").unwrap().as_str(), "model_name");
    }

    #[test]
    fn test_variable_re_namespaced_preset() {
        let a = "{{preset.current_date}}";
        let caps = VARIABLE_RE.captures(a).unwrap();

        assert_eq!(caps.name("namespace").unwrap().as_str(), "preset");
        assert_eq!(caps.name("name").unwrap().as_str(), "current_date");
    }

    #[test]
    fn test_variable_re_namespaced_base() {
        let a = "{{base.system_prompt}}";
        let caps = VARIABLE_RE.captures(a).unwrap();

        assert_eq!(caps.name("namespace").unwrap().as_str(), "base");
        assert_eq!(caps.name("name").unwrap().as_str(), "system_prompt");
    }

    #[test]
    fn test_parse_base_variable() {
        let prompt: Prompt = "{{base.system_prompt}}".parse().unwrap();
        assert_eq!(prompt.content, "{{_base_system_prompt}}");
        assert_eq!(prompt.variables.len(), 1);
        assert!(matches!(
            prompt.variables[0],
            PromptVariable::Base(BaseVariable::SystemPrompt)
        ));
    }

    #[test]
    fn test_parse_unknown_base_variable() {
        let result: Result<Prompt, _> = "{{base.nonsense}}".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_re() {
        let a = "{{field_name}}";
        let captured = VARIABLE_RE.captures(a).unwrap();
        assert_eq!(captured.get_match().as_str(), "{{field_name}}");

        let b = "Hi {{field_name: &HashMap<String, String>}}";
        let captured = VARIABLE_RE.captures(b).unwrap();
        assert_eq!(
            captured.get(0).unwrap().as_str(),
            "{{field_name: &HashMap<String, String>}}"
        );
    }

    #[test]
    fn comment_re_matches_comment() {
        let input = "This is a prompt // This is a comment";
        let result = COMMENT_RE.replace_all(input, "");
        assert_eq!(result, "This is a prompt");
    }

    #[test]
    fn comment_re_matches_multiline_comment() {
        let input = "This is a prompt // This is a comment\n\
            This is another line // Another comment";
        let result = COMMENT_RE.replace_all(input, "");
        assert_eq!(result, "This is a prompt\nThis is another line");
    }
}
