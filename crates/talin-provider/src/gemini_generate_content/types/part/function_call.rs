use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A predicted function call from the model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FunctionCall {
    /// The id of the function to call.
    pub id: Option<String>,

    /// The name of the function to call.
    pub name: String,

    /// The function parameters as a JSON object.
    pub args: Option<serde_json::Value>,
}

impl FunctionCall {
    /// Creates a new [`FunctionCall`].
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            args: None,
        }
    }

    /// Sets the `id` field.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the `args` field.
    pub fn with_args(mut self, args: serde_json::Value) -> Self {
        self.args = Some(args);
        self
    }
}
