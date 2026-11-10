use schemars::Schema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{AllowedCaller, CacheControlEphemeral};

/// A custom tool definition.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Tool {
    /// The name of the tool.
    ///
    /// See [`name`](https://docs.claude.com/en/api/messages#body-tools-name)
    pub name: String,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// A description of what this tool does.
    ///
    /// See [`description`](https://docs.claude.com/en/api/messages#body-tools-description)
    #[cfg_attr(test, schema(nullable = false))]
    pub description: Option<String>,

    /// Whether this tool's input is streamed incrementally as it is generated, rather than
    /// buffered until the full JSON is available.
    ///
    /// `None` uses the default behavior based on the beta headers of the request.
    pub eager_input_streaming: Option<bool>,

    /// Examples of valid inputs for this tool.
    #[cfg_attr(
        test,
        schema(schema_with = crate::claude_messages::spec::schema::object_array_schema)
    )]
    pub input_examples: Option<Vec<serde_json::Value>>,

    /// The JSON schema for the tool's input.
    ///
    /// See [`input_schema`](https://docs.claude.com/en/api/messages#body-tools-input-schema)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::input_schema_schema))]
    pub input_schema: Schema,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

impl Tool {
    /// Creates a new [`Tool`] from the given name and input schema.
    pub fn new(name: impl Into<String>, input_schema: Schema) -> Self {
        Self {
            name: name.into(),
            allowed_callers: None,
            defer_loading: None,
            description: None,
            eager_input_streaming: None,
            input_examples: None,
            input_schema,
            strict: None,
            cache_control: None,
        }
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets the [`description`](Self::description) of what this tool does.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets whether this tool's input is streamed incrementally
    /// ([`eager_input_streaming`](Self::eager_input_streaming)).
    pub fn with_eager_input_streaming(mut self, eager_input_streaming: bool) -> Self {
        self.eager_input_streaming = Some(eager_input_streaming);
        self
    }

    /// Sets the [`input_examples`](Self::input_examples) of valid inputs for this tool.
    pub fn with_input_examples(mut self, input_examples: Vec<serde_json::Value>) -> Self {
        self.input_examples = Some(input_examples);
        self
    }

    /// Sets whether to enforce [`strict`](Self::strict) input validation for this tool.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}
