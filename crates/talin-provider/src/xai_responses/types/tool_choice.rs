use serde::{Deserialize, Serialize};

use crate::utils::tagged_enum;

tagged_enum! {
    /// Controls which (if any) tool is called by the model.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ModelToolChoice {
        /// Forces the model to use a specific tool.
        #[serde(rename = "function")]
        @schema(title = "ModelToolChoiceTool")
        Tool(ModelToolChoiceTool),
        /// Controls tool access by the model using a mode.
        #[serde(untagged)]
        @schema(value_type = String, title = "ModelToolChoiceMode")
        Mode(ModelToolChoiceMode),
    }
}

/// Tool choice mode.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ModelToolChoiceMode {
    /// The model will not call any tool and instead generates a message.
    None,

    /// The model can pick between generating a message or calling one or more tools.
    Auto,

    /// The model must call one or more tools.
    Required,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

/// Specific tool choice configuration.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ModelToolChoiceTool {
    /// Name of the function to use.
    pub name: String,
}

impl ModelToolChoiceTool {
    /// Creates a [`ModelToolChoiceTool`] forcing the model to call the named function.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
