use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::{serde_as, skip_serializing_none};

/// Google search entry point.
#[skip_serializing_none]
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SearchEntryPoint {
    /// Optional. Web content snippet that can be embedded in a web page or an app webview.
    pub rendered_content: Option<String>,

    /// Optional. Base64 encoded JSON representing array of <search term, search url> tuple.
    #[serde_as(as = "Option<Base64>")]
    pub sdk_blob: Option<Vec<u8>>,
}
