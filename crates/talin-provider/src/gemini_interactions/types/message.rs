use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Error message returned from an interaction.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ErrorMessage {
    /// A URI that identifies the error type.
    #[cfg_attr(test, schema(nullable = false))]
    pub code: Option<String>,

    /// A human-readable error message.
    #[cfg_attr(test, schema(nullable = false))]
    pub message: Option<String>,
}

impl std::fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_deref().unwrap_or("Unknown error"))
    }
}
