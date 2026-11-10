use serde::Serialize;
use serde_with::skip_serializing_none;

/// An object describing metadata about the request.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct Metadata {
    /// An external identifier for the user who is associated with the request.
    ///
    /// See [`user_id`](https://docs.claude.com/en/api/messages#body-metadata-user-id)
    pub user_id: Option<String>,
}

impl Metadata {
    /// Creates a new [`Metadata`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`user_id`](Self::user_id) for the request.
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}
