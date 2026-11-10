use indoc::formatdoc;
use schemars::Schema;
use serde_json::json;

use crate::text::render::{add_indent, render_schema};
use crate::tools::normalize_tool_name;

/// The specification of a function tool available to the LLM.
///
/// # Fields
/// - [`name`](Self::name): The name of the function.
/// - [`description`](Self::description): A description of the function.
/// - [`parameters`](Self::parameters): The parameters of the function as a JSON schema.
/// - [`strict`](Self::strict): Whether the function should be called with strict parameter
///   validation.
/// - [`custom_description`](Self::custom_description): Optional fully custom description used
///   instaed of the auto-generated tool description.
#[derive(Debug, Clone)]
pub struct FunctionSpec {
    /// The name of the function.
    pub name: String,
    /// A description of the function.
    pub description: Option<String>,
    /// The parameters of the function as a JSON schema.
    pub parameters: Schema,
    /// Whether the function should be called with strict parameter validation.
    pub strict: bool,
    /// Optional fully custom description used instaed of the auto-generated tool description.
    pub custom_description: Option<String>,
}

impl FunctionSpec {
    /// Constructs a new [`FunctionSpec`].
    pub fn new(
        name: String,
        description: Option<String>,
        parameters: Schema,
        strict: bool,
        custom_description: Option<String>,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            strict,
            custom_description,
        }
    }

    /// Returns the JSON representation of the function specification.
    pub fn as_json(&self) -> String {
        let json = json!({
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters,
            "strict": self.strict,
        });
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())
    }

    /// Returns a human-readable description of the function specification.
    pub fn describe(&self) -> String {
        if let Some(custom) = &self.custom_description {
            return custom.clone();
        }

        let name = normalize_tool_name(&self.name);
        let desc = self.description.as_deref().unwrap_or("");
        let parameters = render_schema(&self.parameters).map(|p| add_indent(&p, 4, false));

        match parameters {
            Ok(parameters) => formatdoc! {"
                <{name}>
                {desc}

                Schema:
                ```json
                {{
                    \"name\": \"{name}\",
                    \"arguments\": {parameters}
                }}
                ```
                </{name}>"
            },
            Err(e) => {
                log::warn!("Failed to describe parameters for tool {}: {e}", self.name);
                formatdoc! {"
                    <{name}>
                    {desc}
                    </{name}>"
                }
            }
        }
    }
}
