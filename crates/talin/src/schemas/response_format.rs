use indoc::formatdoc;
use schemars::Schema;

use crate::text::render::render_schema;

/// A parameter to control the format that the model must output.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ResponseFormat {
    /// Default response format. Used to generate text responses.
    Text,

    /// JSON object response format. An older method of generating JSON responses.
    /// Using `json_schema` is recommended for models that support it.
    JsonObject,

    /// JSON Schema response format. Used to generate structured JSON responses.
    JsonSchema(ResponseFormatJsonSchema),
}

/// A JSON Schema that defines the structure of the output.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseFormatJsonSchema {
    /// The name of the response format. Must be a-z, A-Z, 0-9, or contain underscores and dashes,
    /// with a maximum length of 64.
    pub name: String,

    /// A description of what the response format is for, used by the model to determine how to
    /// respond in the format.
    pub description: Option<String>,

    /// The schema for the response format, described as a JSON Schema object.
    pub schema: Schema,

    /// Whether to enable strict schema adherence when generating the output.
    pub strict: Option<bool>,
}

impl ResponseFormat {
    /// The model will output a JSON object that conforms to the provided JSON schema.
    ///
    /// This function does not validate whether the provided schema is a valid JSON schema. The
    /// OpenAI API may reject the request if the schema is invalid.
    #[must_use]
    pub fn json_schema(
        name: impl Into<String>,
        description: Option<String>,
        schema: Schema,
        strict: bool,
    ) -> Self {
        ResponseFormat::JsonSchema(ResponseFormatJsonSchema {
            name: name.into(),
            description,
            schema,
            strict: Some(strict),
        })
    }

    /// Returns a human-readable instruction of the response format.
    pub fn describe(&self) -> Option<String> {
        const JSON_INSTRUCTION: &str = "Your final response must be a valid JSON object.";

        match self {
            ResponseFormat::Text => None,
            ResponseFormat::JsonObject => Some(JSON_INSTRUCTION.to_owned()),
            ResponseFormat::JsonSchema(schema) => {
                let instruction = match render_schema(&schema.schema) {
                    Ok(schema) => formatdoc! {"
                        {JSON_INSTRUCTION}

                        Schema:
                        ```json
                        {schema}
                        ```"
                    },
                    Err(e) => {
                        log::warn!("Failed to describe response format: {e}");
                        JSON_INSTRUCTION.to_owned()
                    }
                };
                Some(instruction)
            }
        }
    }
}
