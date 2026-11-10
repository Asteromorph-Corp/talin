use std::collections::btree_map::OccupiedEntry;
use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;

use crate::config::{AgentConfig, FromConfigError, Merge};
use crate::error::{AgentError, TalinError};

/// Resolves agent inheritance by merging child agents with their parent agents.
/// This function processes all agents and applies inheritance chains, detecting
/// circular dependencies in the process.
pub(crate) fn resolve_agent_inheritance(
    agents: BTreeMap<String, AgentConfig>,
) -> Result<BTreeMap<String, AgentConfig>, TalinError> {
    let mut resolver = AgentResolver::new(agents);
    resolver.resolve_all()?;
    Ok(resolver.resolved)
}

struct AgentResolver {
    input: IndexMap<String, AgentConfig>,
    resolved: BTreeMap<String, AgentConfig>,
    resolving: BTreeSet<String>,
}

impl AgentResolver {
    fn new(input: BTreeMap<String, AgentConfig>) -> Self {
        let input = input.into_iter().collect();
        Self {
            input,
            resolved: BTreeMap::new(),
            resolving: BTreeSet::new(),
        }
    }

    fn resolve_all(&mut self) -> Result<(), TalinError> {
        while self.resolve_one()? {}
        Ok(())
    }

    fn resolve_one(&mut self) -> Result<bool, TalinError> {
        let Some((name, config)) = self.input.pop() else {
            return Ok(false);
        };
        self.resolve_agent(name, config)?;
        Ok(true)
    }

    fn resolve_agent(
        &mut self,
        id: String,
        config: AgentConfig,
    ) -> Result<OccupiedEntry<'_, String, AgentConfig>, TalinError> {
        // If the agent does not extend another agent, mark it as resolved directly.
        let Some(parent_id) = &config.extends else {
            return Ok(self.resolved.entry(id).insert_entry(config));
        };

        self.resolving.insert(id.clone());

        // Fetch the parent config
        // - If the parent config is already resolved, use it directly.
        // - If the parent config is unresolved, resolve it recursively.
        // - If the parent config is missing, return an error.
        let parent_config = if let Some(parent_config_resolved) = self.resolved.get(parent_id) {
            parent_config_resolved.clone()
        } else if let Some(parent_config_unresolved) = self.input.swap_remove(parent_id) {
            let entry = self.resolve_agent(parent_id.clone(), parent_config_unresolved)?;
            entry.get().clone()
        } else if self.resolving.contains(parent_id) {
            return Err(AgentError::CyclicInheritance.into_error(id));
        } else {
            return Err(AgentError::missing_parent(parent_id).into_error(id));
        };
        let merged = config.merge(parent_config);

        self.resolving.remove(&id);
        Ok(self.resolved.entry(id).insert_entry(merged))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::config::{
        AgentModel, AgentOverrides, Attachments, CallOptions, InlineOrEnv, Model, ModelField,
        ProviderConfig,
    };

    fn create_test_model() -> ModelField {
        let model = AgentModel::Inline(Box::new(Model {
            api_base: InlineOrEnv::Inline("https://api.example.com".to_owned()),
            model: InlineOrEnv::Inline("example".to_owned()),
            api_key: None,
            headers: Default::default(),
            provider_config: ProviderConfig::default(),
            call_options: CallOptions::default(),
        }));
        ModelField {
            model: Some(model),
            call_options: None,
        }
    }

    #[allow(deprecated)]
    fn create_test_agent(
        extends: Option<String>,
        system_prompt: Option<String>,
        model: Option<ModelField>,
    ) -> AgentConfig {
        AgentConfig {
            abstract_: false,
            extends,
            role: String::new(),
            system_prompt,
            user_prompt: None,
            attachments: Attachments::default(),
            extra_variables: BTreeMap::new(),
            max_iterations: None,
            max_consecutive_fails: None,
            max_continuations: None,
            coworkers: Vec::new(),
            tools: Vec::new(),
            toolboxes: Vec::new(),
            builtin_tools: None,
            skills: Vec::new(),
            response_format: None,
            schema_mode: None,
            custom_output: false,
            model,
            call_options: None,
            prompts: BTreeMap::new(),
            overrides: AgentOverrides::default(),
        }
    }

    #[test]
    fn test_no_inheritance() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "agent1".to_string(),
            create_test_agent(None, Some("prompt1".to_string()), Some(create_test_model())),
        );

        let result = resolve_agent_inheritance(agents).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("agent1"));
    }

    #[test]
    fn test_simple_inheritance() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "base".to_string(),
            create_test_agent(
                None,
                Some("base prompt".to_string()),
                Some(create_test_model()),
            ),
        );
        agents.insert(
            "child".to_string(),
            create_test_agent(
                Some("base".to_string()),
                Some("child prompt".to_string()),
                None,
            ),
        );

        let result = resolve_agent_inheritance(agents).unwrap();
        assert_eq!(result.len(), 2);

        let child = result.get("child").unwrap();
        assert_eq!(child.system_prompt, Some("child prompt".to_string()));
        // assert_eq!(child.model, Some(create_test_model()));
    }

    #[test]
    fn test_missing_parent() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "child".to_string(),
            create_test_agent(
                Some("nonexistent".to_string()),
                Some("prompt".to_string()),
                None,
            ),
        );

        let result = resolve_agent_inheritance(agents);
        assert!(result.is_err());
        match result {
            Err(TalinError::InvalidAgentConfig(child, AgentError::MissingConfig(_, parent))) => {
                assert_eq!(child, "child");
                assert_eq!(parent, "nonexistent");
            }
            _ => panic!("Expected MissingParentAgent error"),
        }
    }

    #[test]
    fn test_circular_dependency() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "agent1".to_string(),
            create_test_agent(
                Some("agent2".to_string()),
                Some("prompt1".to_string()),
                None,
            ),
        );
        agents.insert(
            "agent2".to_string(),
            create_test_agent(
                Some("agent1".to_string()),
                Some("prompt2".to_string()),
                None,
            ),
        );

        let result = resolve_agent_inheritance(agents);
        match result {
            Err(TalinError::InvalidAgentConfig(_, AgentError::CyclicInheritance)) => {}
            _ => panic!("Expected CircularAgentInheritance error, got {result:?}",),
        }
    }

    #[test]
    fn test_chain_inheritance() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "grandparent".to_string(),
            create_test_agent(
                None,
                Some("gp prompt".to_string()),
                Some(create_test_model()),
            ),
        );
        agents.insert(
            "parent".to_string(),
            create_test_agent(
                Some("grandparent".to_string()),
                Some("p prompt".to_string()),
                None,
            ),
        );
        agents.insert(
            "child".to_string(),
            create_test_agent(
                Some("parent".to_string()),
                Some("c prompt".to_string()),
                None,
            ),
        );

        let result = resolve_agent_inheritance(agents).unwrap();
        assert_eq!(result.len(), 3);

        let child = result.get("child").unwrap();
        assert_eq!(child.system_prompt, Some("c prompt".to_string()));
        // assert_eq!(child.model, Some(create_test_model()));
    }
}
