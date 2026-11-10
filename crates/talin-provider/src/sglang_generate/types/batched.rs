use serde::{Deserialize, Serialize};

/// An input field that can be either a single value or a batch of values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Batched<T> {
    /// The value for a single request.
    Single(T),
    /// One value per request in a batch.
    Batch(Vec<T>),
}
