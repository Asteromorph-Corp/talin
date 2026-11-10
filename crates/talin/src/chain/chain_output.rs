pub use talin_macros::ChainOutput;

use crate::schemas::ContentPart;
use crate::text::parse::ParseError;

/// A type alias for accessing the partial type of a [`ChainOutput`].
///
/// This is a convenience alias for `<O as ChainOutput>::Partial`.
pub type Partial<O> = <O as ChainOutput>::Partial;

/// A trait for types that can be produced as output from a chain.
///
/// Chain outputs are constructed in two phases:
/// 1. First, a [`PartialOutput`] is parsed from the LLM response.
/// 2. Then, the partial is combined with the input to produce the final output.
///
/// # Deriving
///
/// Use the `#[derive(ChainOutput)]` macro to automatically implement this trait
/// for custom structs. The macro supports JSON parsing and schema generation.
pub trait ChainOutput: Sized + Send + Sync {
    /// The partial output type that can be parsed from LLM responses.
    type Partial: PartialOutput;
}

/// A trait for types that can be parsed from LLM responses.
///
/// Partial outputs represent the intermediate state between receiving an LLM response
/// and constructing the final chain output. They can be parsed from either text
/// responses or tool calls.
pub trait PartialOutput: Sized + Send + Sync + 'static {
    /// Parses the partial output from a text response.
    ///
    /// For structured outputs, this typically involves JSON parsing.
    fn from_text(text: &str) -> Result<Self, ParseError>;

    /// Parses the partial output from the full content of a response.
    ///
    /// Defautls to parsing the concatenated texgt parts, ignoring any media.
    fn from_parts(parts: Vec<ContentPart>) -> Result<Self, ParseError> {
        let text: String = parts
            .into_iter()
            .filter_map(ContentPart::into_text)
            .collect();
        Self::from_text(&text)
    }
}

/// A trait for combining a partial output with input to produce the final output.
///
/// This allows chain outputs to incorporate data from both the LLM response
/// and the original input. For example, an output might include a response
/// field from the LLM and an ID field propagated from the input.
pub trait PartialWithInput<I, O>: PartialOutput
where
    I: Send + Sync,
    O: Send + Sync,
{
    /// Combines this partial output with the given input to produce the final output.
    fn with_input(self, input: I) -> O;
}

/// Default implementation for outputs that don't need input data.
impl<I, O> PartialWithInput<I, O> for O
where
    I: Send + Sync,
    O: PartialOutput,
{
    fn with_input(self, _input: I) -> O {
        self
    }
}

impl ChainOutput for String {
    type Partial = Self;
}
impl PartialOutput for String {
    fn from_text(text: &str) -> Result<Self, ParseError> {
        Ok(text.to_owned())
    }
}

impl ChainOutput for () {
    type Partial = ();
}
impl PartialOutput for () {
    fn from_text(_text: &str) -> Result<Self, ParseError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use schemars::{JsonSchema, schema_for};
    use serde::Deserialize;

    use super::*;

    #[rstest]
    #[case(
        r#"
        {
            "message": "Hello, world!",
            "value": 42.0
        }"#,
        "Hello, world!",
        42.0
    )]
    #[case::messy(
        r#"
        Hello! This is a test message. [[1]]("Hi")
        {
            "message": "Test message",
            "value": 3.15
        }
        [[2]]("Something")"#,
        "Test message",
        3.15
    )]
    fn test_chain_output_from_text(
        #[case] response: &str,
        #[case] message: &str,
        #[case] value: f64,
    ) {
        #[derive(ChainOutput)]
        struct MyOutput {
            message: String,
            value: f64,
            #[chain(from = response)]
            raw: String,
        }

        let output: MyOutput = MyOutput::from_text(response).unwrap();
        assert_eq!(output.message, message);
        assert_eq!(output.value, value);
        assert_eq!(output.raw, response);
    }

    #[test]
    fn test_chain_output_from_text_and_input() {
        #[derive(Debug)]
        struct MyInput {
            prefix: String,
        }

        #[derive(ChainOutput)]
        #[chain(from_input = MyInput)]
        struct MyOutput {
            message: String,
            value: f64,
            #[chain(from = input)]
            prefix: String,
        }

        let content = r#"
        {
            "message": "Hello, world!",
            "value": 42.0
        }"#;

        let input = MyInput {
            prefix: "Greeting:".to_string(),
        };

        let partial: Partial<MyOutput> = Partial::<MyOutput>::from_text(content).unwrap();
        let output: MyOutput = partial.with_input(input);
        assert_eq!(output.message, "Hello, world!");
        assert_eq!(output.value, 42.0);
        assert_eq!(output.prefix, "Greeting:");
    }

    #[test]
    fn test_chain_output_derive_schema() {
        struct MyInput {
            count: u32,
        }

        /// An example output struct
        #[derive(ChainOutput)]
        #[chain(derive_schema, from_input = MyInput)]
        #[allow(dead_code)]
        struct MyOutput {
            /// The message from the response
            #[chain(from = response_json)]
            message: String,

            /// A numeric value from the response
            #[chain(from = response_json)]
            value: f64,

            /// The raw response text, should not be included in the schema.
            #[chain(from = response)]
            raw: String,

            /// A value propagted from input, should not be included in the schema.
            #[chain(from = input.count)]
            count: u32,
        }

        assert_eq!(
            schema_for!(MyOutput).to_value(),
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "MyOutput",
                "description": "An example output struct",
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message from the response"
                    },
                    "value": {
                        "type": "number",
                        "format": "double",
                        "description": "A numeric value from the response"
                    }
                },
                "required": ["message", "value"]
            })
        );
    }

    #[test]
    fn test_chain_output_derive_schema_empty() {
        #[derive(ChainOutput)]
        #[chain(derive_schema)]
        #[allow(dead_code)]
        struct MyOutput {
            #[chain(from = response)]
            raw: String,
        }

        assert_eq!(
            schema_for!(MyOutput).to_value(),
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "MyOutput",
                "type": "object",
                "properties": {}
            })
        );
    }

    #[test]
    fn test_chain_output_derive_schema_inner() {
        struct MyInput {
            count: u32,
        }

        #[derive(Deserialize, JsonSchema)]
        #[allow(dead_code)]
        struct MyOutputInner {
            /// The message from the response
            message: String,

            /// A numeric value from the response
            value: f64,
        }

        /// An example output struct
        #[derive(ChainOutput)]
        #[chain(derive_schema, from_input = MyInput)]
        #[allow(dead_code)]
        struct MyOutput {
            /// The raw response text
            #[chain(from = response)]
            inner: MyOutputInner,

            /// A value propagted from input, should not be included in the schema.
            #[chain(from = input.count)]
            count: u32,
        }

        assert_eq!(
            schema_for!(MyOutput).to_value(),
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "MyOutput",
                "description": "An example output struct",
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message from the response"
                    },
                    "value": {
                        "type": "number",
                        "format": "double",
                        "description": "A numeric value from the response"
                    }
                },
                "required": ["message", "value"]
            })
        );
    }
}
