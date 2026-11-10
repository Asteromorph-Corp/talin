use std::path::Path;

use thiserror::Error;

use crate::config::ModelType;

#[derive(Debug, Error)]
pub enum TalinError {
    #[error("Invalid configuration for agent {0}: {1}")]
    InvalidAgentConfig(String, AgentError),
    #[error("Invalid configuration for model {0}: {1}")]
    InvalidModelConfig(String, ModelError),
    #[error("Invalid prompt {0}: {1}")]
    InvalidPrompt(String, PromptError),

    #[error("Failed to deserialize configuration file at {0}: {1}")]
    ConfigDeserializeFailure(String, toml::de::Error),
    #[error("Found both 'configs/{0}.toml' and 'configs/{0}/'")]
    ConflictingConfigPath(&'static str),
    #[error("{0}")]
    ConflictingConfigKey(#[from] DuplicateConfig),

    #[error("File I/O error: {0}")]
    Io(#[from] IoError),
    #[error("Missing env var: {0}")]
    MissingEnvVar(&'static str),
}

impl TalinError {
    pub fn invalid_agent_config(agent_id: impl Into<String>) -> impl FnOnce(AgentError) -> Self {
        move |e| TalinError::InvalidAgentConfig(agent_id.into(), e)
    }
}

#[derive(Debug, Error)]
#[error("Found duplicate {config_type} '{key}' from both '{source1}' and '{source2}'")]
pub struct DuplicateConfig {
    pub config_type: &'static str,
    pub key: String,
    pub source1: String,
    pub source2: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AgentSkillError {
    #[error("skill entries must not be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum ModelError {}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Invalid prompt at '{0}': {1}")]
    InvalidPrompt(String, PromptError),
    #[error("Invalid tool configuration: {0}")]
    InvalidTool(#[from] AgentToolError),
    #[error("Invalid skill: {0}")]
    InvalidSkill(#[from] AgentSkillError),
    #[error("Invalid input variable: {0}")]
    InvalidInputVariable(#[from] InputVariableError),
    #[error("Invalid override ({0}): {1}")]
    InvalidOverride(ModelType, OverrideError),
    #[error("Missing required field: {0}")]
    MissingRequiredField(&'static str),
    #[error("{0}")]
    ConflictingConfigKey(#[from] DuplicateConfig),
    #[error("Error resolving {0} prompt: {1}")]
    PromptResolutionFailed(&'static str, PromptResolutionError),
    #[error("No configuration found for {0} '{1}'")]
    MissingConfig(&'static str, String),
    #[error("Cyclic inheritance detected")]
    CyclicInheritance,
}

impl AgentError {
    pub fn invalid_system_prompt(e: PromptError) -> Self {
        AgentError::InvalidPrompt(String::from("system_prompt"), e)
    }

    pub fn system_prompt_resolution_failed(e: PromptResolutionError) -> Self {
        AgentError::PromptResolutionFailed("system", e)
    }

    pub fn invalid_user_prompt(e: PromptError) -> Self {
        AgentError::InvalidPrompt(String::from("user_prompt"), e)
    }

    pub fn user_prompt_resolution_failed(e: PromptResolutionError) -> Self {
        AgentError::PromptResolutionFailed("user", e)
    }

    pub fn invalid_override(override_id: ModelType, e: OverrideError) -> Self {
        AgentError::InvalidOverride(override_id, e)
    }

    pub fn missing_parent(parent_id: impl Into<String>) -> Self {
        AgentError::MissingConfig("parent agent", parent_id.into())
    }

    pub fn missing_model(model_id: impl Into<String>) -> Self {
        AgentError::MissingConfig("model", model_id.into())
    }

    pub fn missing_mcp_server(mcp_server_id: impl Into<String>) -> Self {
        AgentError::MissingConfig("mcp server", mcp_server_id.into())
    }
}

#[derive(Debug, Error)]
pub enum OverrideError {
    #[error(
        "Prompt override cannot introduce new input variable; {0} prompt introduced new input variable `{1}`"
    )]
    IntroduceVariable(&'static str, String),
    #[error("Includes built-in tool configuration for {0}")]
    BuiltinToolMismatch(ModelType),
}

#[derive(Debug, Error)]
pub enum AgentToolError {
    #[error("Mcp tool {1} for server {0} not specified in server config")]
    McpUnspecifiedTool(String, String),
    #[error("Found both wildcard 'mcp.{0}/*' and specific tool 'mcp.{0}/{1}'")]
    McpWildcardConflict(String, String),
    #[error("Found multiple wildcard entries 'mcp.{0}/*'")]
    McpMultipleWildcard(String),
    #[error("Invalid init params for tool '{0}': {1}")]
    InvalidInitParams(String, ToolInitParamsError),
}

#[derive(Debug, Error)]
pub enum ToolInitParamsError {
    #[error("Must set both `\"$enum\"` and \"$variant\" for enum field.")]
    MissingEnumOrVariant,
    #[error("Cannot mix tuple indices (`$N`) with regular fields.")]
    MixedFields,
    #[error("`{0}` must be a string.")]
    NonStringMetadataValue(&'static str),
}

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Unknown variable namespace: {0}")]
    UnknownVariableNamespace(String),

    #[error("Invalid input variable: {0}")]
    InvalidInputVariable(#[from] InputVariableError),
    #[error("Unknown preset variable: {0}")]
    UnknownPresetVariable(String),
    #[error("Unknown base variable: {0} (expected `system_prompt` or `user_prompt`)")]
    UnknownBaseVariable(String),
}

#[derive(Debug, Error)]
pub enum PromptResolutionError {
    #[error("Missing config variable: {0}")]
    MissingConfigVariable(String),
    #[error("Detected cyclic dependency")]
    CyclicDependency,
    #[error(
        "`base.{0}` can only be used inside a prompt override; the base prompts cannot reference themselves"
    )]
    BaseVariableNotAllowed(&'static str),
}

#[derive(Debug, Error)]
pub enum InputVariableError {
    #[error("Invalid identifier for input variable: '{0}' ({1})")]
    InvalidIdent(String, syn::Error),
    #[error("Invalid type for input variable '{0}': '{1}' ({2})")]
    InvalidType(String, String, syn::Error),
}

#[derive(Debug, Error)]
pub enum IoError {
    #[error("Failed to create directory {0}: {1}")]
    CreateDirectory(String, std::io::Error),
    #[error("Failed to read file at {0}: {1}")]
    ReadFile(String, std::io::Error),
    #[error("Failed to write file at {0}: {1}")]
    WriteFile(String, std::io::Error),
}

impl IoError {
    pub fn create_dir_failure(path: &Path, e: std::io::Error) -> Self {
        IoError::CreateDirectory(path.display().to_string(), e)
    }

    pub fn read_file_failure(path: &Path, e: std::io::Error) -> Self {
        IoError::ReadFile(path.display().to_string(), e)
    }

    pub fn write_file_failure(path: &Path, e: std::io::Error) -> Self {
        IoError::WriteFile(path.display().to_string(), e)
    }
}
