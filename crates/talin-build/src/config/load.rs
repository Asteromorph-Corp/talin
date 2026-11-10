use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use itertools::Itertools;
use serde::de::DeserializeOwned;
use walkdir::WalkDir;

use crate::config::{
    Agent, AgentConfig, Configs, FromConfig, FromConfigError, McpServer, Model, Prompt, Tool,
};
use crate::error::{IoError, TalinError};
use crate::resolution::{collect_configs, resolve_agent_inheritance};

pub(crate) fn load_configs(config_dir: &Path) -> Result<Configs, TalinError> {
    let prompts = load_typed_configs::<Prompt>(config_dir, "prompts")?;
    let models = load_typed_configs::<Model>(config_dir, "models")?;
    let agents = load_agent_configs(config_dir, &models, &prompts)?;
    let tools = load_typed_configs::<Tool>(config_dir, "tools")?;
    let servers = load_typed_configs::<McpServer>(config_dir, "servers")?;

    Ok(Configs {
        agents,
        models,
        tools,
        servers,
    })
}

fn load_typed_configs<C>(
    config_dir: &Path,
    config_type: &'static str,
) -> Result<BTreeMap<String, C>, TalinError>
where
    C: FromConfig,
{
    let configs_unresolved = load_raw_configs::<C::Config>(config_dir, config_type)?;
    configs_unresolved
        .into_iter()
        .map(|(k, v)| match C::from_config(v) {
            Ok(config) => Ok((k, config)),
            Err(e) => Err(e.into_error(k)),
        })
        .try_collect()
}

fn load_agent_configs(
    config_dir: &Path,
    models: &BTreeMap<String, Model>,
    prompts: &BTreeMap<String, Prompt>,
) -> Result<BTreeMap<String, Agent>, TalinError> {
    let configs_unresolved = load_raw_configs::<AgentConfig>(config_dir, "agents")?;

    let configs_resolved = resolve_agent_inheritance(configs_unresolved)?;
    configs_resolved
        .into_iter()
        .filter(|(_, config)| !config.abstract_)
        .map(|(key, config)| {
            Agent::from_config(&key, config, models, prompts)
                .map(|agent| (key.clone(), agent))
                .map_err(|e| TalinError::InvalidAgentConfig(key, e))
        })
        .collect()
}

/// Internal function to load configs without converting them to their final types.
fn load_raw_configs<T>(
    config_dir: &Path,
    config_type: &'static str,
) -> Result<BTreeMap<String, T>, TalinError>
where
    T: DeserializeOwned,
{
    let config_paths = read_config_paths(config_dir, config_type)?;
    let configs = config_paths
        .iter()
        .map(|path| -> Result<_, TalinError> {
            let config_str =
                fs::read_to_string(path).map_err(|e| IoError::read_file_failure(path, e))?;
            let config = toml::from_str::<BTreeMap<String, T>>(&config_str)
                .map_err(|e| TalinError::ConfigDeserializeFailure(path.display().to_string(), e))?;
            Ok((path.display().to_string(), config))
        })
        .try_collect()?;
    let configs = collect_configs(configs, config_type)?;
    Ok(configs)
}

fn read_config_paths(
    config_dir: &Path,
    config_type: &'static str,
) -> Result<Vec<PathBuf>, TalinError> {
    let single_file = config_dir.join(format!("{config_type}.toml"));
    let directory = config_dir.join(config_type);

    match (
        single_file.exists(),
        directory.exists() && directory.is_dir(),
    ) {
        (true, true) => Err(TalinError::ConflictingConfigPath(config_type)),
        (false, false) => Ok(vec![]),
        (true, false) => Ok(vec![single_file]),
        (false, true) => {
            let mut configs = WalkDir::new(&directory)
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    path.extension()
                        .is_some_and(|ext| ext == "toml")
                        .then(|| path.to_path_buf())
                })
                .collect::<Vec<_>>();
            configs.sort();
            Ok(configs)
        }
    }
}
