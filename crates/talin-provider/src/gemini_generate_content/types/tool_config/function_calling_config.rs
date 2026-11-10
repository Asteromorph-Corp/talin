use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Configuration for function calling.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FunctionCallingConfig {
    /// Specifies the mode in which function calling should execute.
    pub mode: Option<FunctionCallingMode>,

    /// Optional. A set of function names that, when provided, limits the functions
    /// the model will call.
    pub allowed_function_names: Option<Vec<String>>,
}

/// Defines the execution behavior for function calling by defining the execution mode.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum FunctionCallingMode {
    /// Unspecified function calling mode. This value should not be used.
    ModeUnspecified,
    /// Default model behavior, model decides to predict either a function call or a natural
    /// language response.
    Auto,
    /// Model is constrained to always predicting a function call only. If "allowedFunctionNames"
    /// are set, the predicted function call will be limited to any one of "allowedFunctionNames",
    /// else the predicted function call will be any one of the provided "functionDeclarations".
    Any,
    /// Model will not predict any function call. Model behavior is same as when not passing any
    /// function declarations.
    None,
    /// Model decides to predict either a function call or a natural language response, but will
    /// validate function calls with constrained decoding. If "allowedFunctionNames" are set, the
    /// predicted function call will be limited to any one of "allowedFunctionNames", else the
    /// predicted function call will be any one of the provided "functionDeclarations".
    Validated,
}

impl FunctionCallingConfig {
    /// Creates a new empty FunctionCallingConfig.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a FunctionCallingConfig with AUTO mode.
    pub fn auto() -> Self {
        Self {
            mode: Some(FunctionCallingMode::Auto),
            allowed_function_names: None,
        }
    }

    /// Creates a FunctionCallingConfig with ANY mode.
    pub fn any() -> Self {
        Self {
            mode: Some(FunctionCallingMode::Any),
            allowed_function_names: None,
        }
    }

    /// Creates a FunctionCallingConfig with NONE mode.
    pub fn none() -> Self {
        Self {
            mode: Some(FunctionCallingMode::None),
            allowed_function_names: None,
        }
    }

    /// Creates a FunctionCallingConfig with VALIDATED mode.
    pub fn validated() -> Self {
        Self {
            mode: Some(FunctionCallingMode::Validated),
            allowed_function_names: None,
        }
    }

    /// Sets the function calling mode.
    pub fn with_mode(mut self, mode: FunctionCallingMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Sets the allowed function names.
    pub fn with_allowed_function_names(mut self, names: Vec<String>) -> Self {
        self.allowed_function_names = Some(names);
        self
    }

    /// Adds an allowed function name.
    pub fn with_allowed_function(mut self, name: impl Into<String>) -> Self {
        self.allowed_function_names
            .get_or_insert_with(Vec::new)
            .push(name.into());
        self
    }
}
