use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Retrieval config.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RetrievalConfig {
    /// Optional. The location of the user.
    pub lat_lng: Option<LatLng>,
    /// Optional. The language code of the user. Language code for content. Use language tags
    /// defined by BCP47.
    pub language_code: Option<String>,
}

impl RetrievalConfig {
    /// Creates a new RetrievalConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the location of the user.
    pub fn with_lat_lng(mut self, lat_lng: LatLng) -> Self {
        self.lat_lng = Some(lat_lng);
        self
    }

    /// Sets the language code of the user (BCP47).
    pub fn with_language_code(mut self, language_code: impl Into<String>) -> Self {
        self.language_code = Some(language_code.into());
        self
    }
}

/// An object that represents a latitude/longitude pair.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LatLng {
    /// The latitude in degrees. It must be in the range [-90.0, +90.0].
    pub latitude: f64,
    /// The longitude in degrees. It must be in the range [-180.0, +180.0].
    pub longitude: f64,
}

impl LatLng {
    /// Creates a new LatLng with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the latitude in degrees ([-90.0, +90.0]).
    pub fn with_latitude(mut self, latitude: f64) -> Self {
        self.latitude = latitude;
        self
    }

    /// Sets the longitude in degrees ([-180.0, +180.0]).
    pub fn with_longitude(mut self, longitude: f64) -> Self {
        self.longitude = longitude;
        self
    }
}
