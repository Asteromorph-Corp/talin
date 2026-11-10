use schemars::JsonSchema;
use serde::Deserialize;

/// A simple string input for tools.
///
/// Use this for tools that require a single string argument.
#[derive(Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(description = "The input for the tool.")]
pub struct DefaultFunctionInput {
    /// The input string.
    pub input: String,
}

/// An empty input type for tools that require no arguments.
///
/// Use this for tools that don't require any input parameters
#[derive(Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EmptyFunctionInput {}

impl DefaultFunctionInput {
    /// Creates a new `DefaultFunctionInput` from a string-like value.
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
        }
    }
}

impl From<String> for DefaultFunctionInput {
    fn from(input: String) -> Self {
        Self::new(input)
    }
}

impl From<&str> for DefaultFunctionInput {
    fn from(input: &str) -> Self {
        Self::new(input)
    }
}

#[cfg(test)]
mod tests {
    use schemars::schema_for;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_default_tool_input_schema() {
        let schema = schema_for!(DefaultFunctionInput);
        let schema = serde_json::to_value(schema).unwrap();

        assert_eq!(
            schema,
            json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "DefaultFunctionInput",
                "description": "The input for the tool.",
                "type": "object",
                "properties": {
                    "input": {
                        "description": "The input string.",
                        "type": "string"
                    }
                },
                "additionalProperties": false,
                "required": ["input"]
            })
        )
    }

    #[test]
    fn test_empty_tool_input_schema() {
        let schema = schema_for!(());
        let schema = serde_json::to_value(schema).unwrap();

        assert_eq!(
            schema,
            json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "null",
                "type": "null",
            })
        )
    }
}
