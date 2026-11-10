use schemars::Schema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::{string_enum, tagged_enum};

/// Configuration options for the model's output.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct OutputConfig {
    /// The effort level for the output.
    pub effort: Option<OutputEffort>,

    /// The JSON schema output format.
    pub format: Option<JSONOutputFormat>,
}

string_enum! {
    /// The effort level for the model's output.
    #[serde(rename_all = "snake_case")]
    pub enum OutputEffort {
        /// `"low"`
        Low,

        /// `"medium"`
        Medium,

        /// `"high"`
        High,

        /// `"xhigh"`
        #[serde(rename = "xhigh")]
        XHigh,

        /// `"max"`
        Max,
    }
}

tagged_enum! {
    /// The output format of the generated data.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum JSONOutputFormat {
        /// JSON Schema format.
        JsonSchema(OutputFormatJsonSchema),
    }
}

/// JSON Schema output format.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OutputFormatJsonSchema {
    /// The JSON Schema.
    #[cfg_attr(test, schema(value_type = Object))]
    pub schema: Schema,
}

impl OutputConfig {
    /// Creates a new [`OutputConfig`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`effort`](Self::effort) level for the output.
    pub fn with_effort(mut self, effort: OutputEffort) -> Self {
        self.effort = Some(effort);
        self
    }

    /// Sets the [`format`](Self::format) for the output.
    pub fn with_format(mut self, format: JSONOutputFormat) -> Self {
        self.format = Some(format);
        self
    }
}

impl OutputFormatJsonSchema {
    /// Creates a new [`OutputFormatJsonSchema`] from the given schema.
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }
}

impl From<Schema> for OutputConfig {
    fn from(schema: Schema) -> Self {
        OutputConfig {
            effort: None,
            format: Some(JSONOutputFormat::JsonSchema(OutputFormatJsonSchema {
                schema,
            })),
        }
    }
}

impl From<Schema> for JSONOutputFormat {
    fn from(schema: Schema) -> Self {
        JSONOutputFormat::JsonSchema(OutputFormatJsonSchema { schema })
    }
}
