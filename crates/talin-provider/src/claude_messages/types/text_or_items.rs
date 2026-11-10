use serde::{Deserialize, Serialize};

/// A plain text string, or a list of structured items of type T.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum TextOrItems<T> {
    /// A plain text string.
    Text(String),

    /// A list of structured items of type T.
    Items(Vec<T>),
}

impl<T> From<String> for TextOrItems<T> {
    fn from(value: String) -> Self {
        TextOrItems::Text(value)
    }
}

impl<T> From<&str> for TextOrItems<T> {
    fn from(value: &str) -> Self {
        TextOrItems::Text(value.to_owned())
    }
}

impl<T> From<Vec<T>> for TextOrItems<T> {
    fn from(value: Vec<T>) -> Self {
        TextOrItems::Items(value)
    }
}
