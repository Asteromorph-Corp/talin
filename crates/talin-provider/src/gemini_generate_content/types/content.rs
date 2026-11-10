use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// The producer of the content. Must be either 'user' or 'model'.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// `"user"`
    User,
    /// `"model"`
    Model,
}

/// Content contains a collection of Parts that can be text, image, video, etc.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Content {
    /// The producer of the content.
    pub role: Option<Role>,

    /// Ordered parts that constitute a single message.
    pub parts: Option<Vec<super::Part>>,
}
