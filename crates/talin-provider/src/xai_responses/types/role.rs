use serde::{Deserialize, Serialize};

/// The role of a message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Role {
    /// The system role.
    System,
    /// The developer role.
    Developer,
    /// The user role.
    User,
    /// The assistant role.
    Assistant,
    /// The tool role.
    Tool,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}
