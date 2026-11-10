use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// The GoogleMaps Tool that provides geospatial context for the user's query.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GoogleMaps {
    /// Optional. Whether to return a widget context token in the GroundingMetadata of the
    /// response. Developers can use the widget context token to render a Google Maps widget with
    /// geospatial context related to the places that the model references in the response.
    pub enable_widget: Option<bool>,
}

impl GoogleMaps {
    /// Creates a new empty GoogleMaps.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to return a widget context token.
    pub fn with_enable_widget(mut self, enable_widget: bool) -> Self {
        self.enable_widget = Some(enable_widget);
        self
    }
}
