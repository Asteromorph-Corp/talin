use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Computer Use tool type.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ComputerUse {
    /// Required. The environment being operated.
    pub environment: Environment,

    /// Optional. By default, predefined functions are included in the final model call. Some of
    /// them can be explicitly excluded from being automatically included. This can serve two
    /// purposes: 1. Using a more restricted / different action space. 2. Improving the definitions
    /// / instructions of predefined functions.
    pub excluded_predefined_functions: Option<Vec<String>>,
}

/// Represents the environment being operated, such as a web browser.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Environment {
    /// Defaults to browser.
    #[default]
    EnvironmentUnspecified,
    /// Operates in a web browser.
    EnvironmentBrowser,
}

impl ComputerUse {
    /// Creates a new empty ComputerUse.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the environment for this tool.
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Sets the excluded predefined functions for this tool.
    pub fn with_excluded_predefined_functions(
        mut self,
        excluded_predefined_functions: Vec<String>,
    ) -> Self {
        self.excluded_predefined_functions = Some(excluded_predefined_functions);
        self
    }
}
