use std::convert::Infallible;

use serde::de::DeserializeOwned;

use crate::error::{AgentError, ModelError, PromptError, TalinError};

pub(crate) trait FromConfig: Sized {
    type Config: DeserializeOwned;
    type Error: FromConfigError;

    fn from_config(config: Self::Config) -> Result<Self, Self::Error>;
}

pub(crate) trait FromConfigError: Sized {
    fn into_error(self, key: impl Into<String>) -> TalinError;
}

impl FromConfigError for ModelError {
    fn into_error(self, key: impl Into<String>) -> TalinError {
        TalinError::InvalidModelConfig(key.into(), self)
    }
}

impl FromConfigError for AgentError {
    fn into_error(self, key: impl Into<String>) -> TalinError {
        TalinError::InvalidAgentConfig(key.into(), self)
    }
}

impl FromConfigError for PromptError {
    fn into_error(self, key: impl Into<String>) -> TalinError {
        TalinError::InvalidPrompt(key.into(), self)
    }
}

impl FromConfigError for Infallible {
    fn into_error(self, _key: impl Into<String>) -> TalinError {
        unreachable!("Infallible error should never occur")
    }
}
