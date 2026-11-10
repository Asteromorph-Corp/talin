use serde::{Deserialize, Serialize};

/// Tool to support URL context retrieval.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UrlContext {}
