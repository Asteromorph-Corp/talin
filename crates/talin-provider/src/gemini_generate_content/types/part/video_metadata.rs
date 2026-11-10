use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

use crate::gemini_generate_content::types::duration::GoogleDuration;

/// Metadata describes the input video content.
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct VideoMetadata {
    /// The start offset of the video.
    #[serde(default)]
    #[serde_as(as = "Option<GoogleDuration>")]
    pub start_offset: Option<Duration>,
    /// The end offset of the video.
    #[serde(default)]
    #[serde_as(as = "Option<GoogleDuration>")]
    pub end_offset: Option<Duration>,
    /// The frame rate of the video sent to the model. If not specified, the default
    /// value will be 1.0. The fps range is (0.0, 24.0].
    pub fps: Option<f64>,
}

impl VideoMetadata {
    /// Creates a new [`VideoMetadata`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `start_offset` field.
    pub fn with_start_offset(mut self, start_offset: Duration) -> Self {
        self.start_offset = Some(start_offset);
        self
    }

    /// Sets the `end_offset` field.
    pub fn with_end_offset(mut self, end_offset: Duration) -> Self {
        self.end_offset = Some(end_offset);
        self
    }

    /// Sets the `fps` field.
    pub fn with_fps(mut self, fps: f64) -> Self {
        self.fps = Some(fps);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_integer_seconds() {
        let v = VideoMetadata {
            start_offset: Some(Duration::from_secs(30)),
            end_offset: Some(Duration::from_secs(90)),
            fps: Some(1.0),
        };
        let json = serde_json::to_string(&v).unwrap();
        assert!(json.contains("\"startOffset\":\"30s\""));
        assert!(json.contains("\"endOffset\":\"90s\""));
        let back: VideoMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.start_offset, v.start_offset);
        assert_eq!(back.end_offset, v.end_offset);
    }

    #[test]
    fn roundtrip_fractional_seconds() {
        let v = VideoMetadata {
            start_offset: Some(Duration::new(3, 500_000_000)),
            end_offset: None,
            fps: None,
        };
        let json = serde_json::to_string(&v).unwrap();
        assert!(json.contains("\"startOffset\":\"3.5s\""));
        let back: VideoMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.start_offset, v.start_offset);
    }
}
