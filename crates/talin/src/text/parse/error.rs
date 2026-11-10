use thiserror::Error;

/// Failed to parse model output.
#[derive(Debug, Error)]
#[error("failed to parse model output: {source}\noriginal text: {original}")]
pub struct ParseError {
    /// The original text content.
    pub original: String,
    /// The inner deserialization error.
    #[source]
    pub source: serde_json::Error,
}

impl ParseError {
    /// Constructs a new [`ParseError`] from the original text and the error.
    pub fn new(original: impl Into<String>, source: serde_json::Error) -> Self {
        Self {
            original: original.into(),
            source,
        }
    }
}
