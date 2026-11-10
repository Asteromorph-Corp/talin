use schemars::Schema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Structured representation of a function declaration.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FunctionDeclaration {
    /// The name of the function.
    pub name: String,

    /// A brief description of what the function does.
    pub description: String,

    /// Optional. Specifies the function Behavior. Currently only supported by the
    /// BidiGenerateContent method.
    pub behavior: Option<Behavior>,

    // /// Optional, Describes the parameters to this function in OpenAPI v3.0.3 Schema format.
    // pub parameters: serde_json::Value,
    /// Optional. Describes the parameters to this function in JSON Schema format.
    pub parameters_json_schema: Option<Schema>,

    // /// Optional. Describes the output of this function in OpenAPI v3.0.3 Schema format.
    // pub response: Option<serde_json::Value>,
    /// Optional. Describes the output of this function in JSON Schema format.
    pub response_json_schema: Option<Schema>,
}

/// Defines the function behavior. Defaults to BLOCKING.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Behavior {
    /// Default value. This value is unused.
    Unspecified,
    /// If set, the system will wait to receive the function response before continuing the
    /// conversation.
    Blocking,
    /// If set, the system will not wait to receive the function response. Instead, it will attempt
    /// to handle function responses as they become available while maintaining the conversation
    /// between the user and the model.
    NonBlocking,
}

impl FunctionDeclaration {
    /// Creates a new FunctionDeclaration with the given name and description.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            behavior: None,
            parameters_json_schema: None,
            response_json_schema: None,
        }
    }

    /// Sets the behavior for this function.
    pub fn with_behavior(mut self, behavior: Behavior) -> Self {
        self.behavior = Some(behavior);
        self
    }

    /// Sets the parameters schema for this function.
    pub fn with_parameters(mut self, schema: Schema) -> Self {
        self.parameters_json_schema = Some(schema);
        self
    }

    /// Sets the response schema for this function.
    pub fn with_response(mut self, schema: Schema) -> Self {
        self.response_json_schema = Some(schema);
        self
    }
}
