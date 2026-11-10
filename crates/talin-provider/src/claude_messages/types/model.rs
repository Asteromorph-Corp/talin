use serde::{Deserialize, Serialize};

/// The model that will complete the prompt.
///
/// Accepts any of the [`KnownModel`] variants, or an arbitrary string for models this crate does
/// not yet name.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Model {
    /// A model named by this crate.
    Known(#[cfg_attr(test, schema(inline = true))] KnownModel),

    /// A model identified by its raw API identifier.
    Other(String),
}

/// A model named by this crate.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KnownModel {
    /// `"claude-haiku-5-5"`
    #[serde(rename = "claude-haiku-5-5")]
    ClaudeHaiku55,

    /// `"claude-sonnet-5-5"`
    #[serde(rename = "claude-sonnet-5-5")]
    ClaudeSonnet55,

    /// `"claude-fable-5-1"`
    #[serde(rename = "claude-fable-5-1")]
    ClaudeFable51,

    /// `"claude-mythos-5-1"`
    #[serde(rename = "claude-mythos-5-1")]
    ClaudeMythos51,

    /// `"claude-sonnet-5"`
    #[serde(rename = "claude-sonnet-5")]
    ClaudeSonnet5,

    /// `"claude-fable-5"`
    #[serde(rename = "claude-fable-5")]
    ClaudeFable5,

    /// `"claude-mythos-5"`
    #[serde(rename = "claude-mythos-5")]
    ClaudeMythos5,

    /// `"claude-opus-5-5"`
    #[serde(rename = "claude-opus-5-5")]
    ClaudeOpus55,

    /// `"claude-opus-5"`
    #[serde(rename = "claude-opus-5")]
    ClaudeOpus5,

    /// `"claude-opus-4-8"`
    #[serde(rename = "claude-opus-4-8")]
    ClaudeOpus48,

    /// `"claude-opus-4-7"`
    #[serde(rename = "claude-opus-4-7")]
    ClaudeOpus47,

    /// `"claude-opus-4-6"`
    #[serde(rename = "claude-opus-4-6")]
    ClaudeOpus46,

    /// `"claude-sonnet-4-6"`
    #[serde(rename = "claude-sonnet-4-6")]
    ClaudeSonnet46,

    /// `"claude-haiku-4-5"`
    #[serde(rename = "claude-haiku-4-5")]
    ClaudeHaiku45,

    /// `"claude-opus-4-5"`
    #[serde(rename = "claude-opus-4-5")]
    ClaudeOpus45,

    /// `"claude-sonnet-4-5"`
    #[serde(rename = "claude-sonnet-4-5")]
    ClaudeSonnet45,
}

impl Model {
    /// Returns the API identifier of the model.
    pub fn as_str(&self) -> &str {
        match self {
            Model::Known(k) => k.as_str(),
            Model::Other(s) => s.as_str(),
        }
    }
}

impl KnownModel {
    /// Returns the API identifier of the model.
    fn as_str(&self) -> &'static str {
        match self {
            KnownModel::ClaudeHaiku55 => "claude-haiku-5-5",
            KnownModel::ClaudeSonnet55 => "claude-sonnet-5-5",
            KnownModel::ClaudeFable51 => "claude-fable-5-1",
            KnownModel::ClaudeMythos51 => "claude-mythos-5-1",
            KnownModel::ClaudeSonnet5 => "claude-sonnet-5",
            KnownModel::ClaudeFable5 => "claude-fable-5",
            KnownModel::ClaudeMythos5 => "claude-mythos-5",
            KnownModel::ClaudeOpus55 => "claude-opus-5-5",
            KnownModel::ClaudeOpus5 => "claude-opus-5",
            KnownModel::ClaudeOpus48 => "claude-opus-4-8",
            KnownModel::ClaudeOpus47 => "claude-opus-4-7",
            KnownModel::ClaudeOpus46 => "claude-opus-4-6",
            KnownModel::ClaudeSonnet46 => "claude-sonnet-4-6",
            KnownModel::ClaudeHaiku45 => "claude-haiku-4-5",
            KnownModel::ClaudeOpus45 => "claude-opus-4-5",
            KnownModel::ClaudeSonnet45 => "claude-sonnet-4-5",
        }
    }
}

impl From<KnownModel> for Model {
    fn from(value: KnownModel) -> Self {
        Model::Known(value)
    }
}

impl From<String> for Model {
    fn from(value: String) -> Self {
        Model::Other(value)
    }
}

impl From<&str> for Model {
    fn from(value: &str) -> Self {
        Model::Other(value.to_owned())
    }
}
