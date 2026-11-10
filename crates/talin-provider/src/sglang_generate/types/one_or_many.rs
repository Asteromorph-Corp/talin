use serde::{Deserialize, Serialize};

/// A value that may be supplied either as a single item or a list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// A single item.
    One(T),
    /// A list of items.
    Many(Vec<T>),
}
