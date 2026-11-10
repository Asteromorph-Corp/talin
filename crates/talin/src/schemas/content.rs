use serde::{Deserialize, Serialize};

use crate::schemas::media::{AudioContent, DocumentContent, ImageContent, VideoContent};

/// A single part of a message's content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "ContentPartWire", into = "ContentPartWire")]
#[non_exhaustive]
pub enum ContentPart {
    /// A piece of plain text.
    Text(String),
    /// An image.
    Image(ImageContent),
    /// An audio clip.
    Audio(AudioContent),
    /// A video clip.
    Video(VideoContent),
    /// A document.
    Document(DocumentContent),
}

/// Mirrors [`ContentPart`] on the wire.
///
/// An internally tagged enum has to inject its tag into each variant's map, so a variant wrapping
/// a bare `String` cannot be serialized. Spelling the text variant out as a struct gives it the
/// map shape the representation needs while every other variant stays flat.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPartWire {
    Text { text: String },
    Image(ImageContent),
    Audio(AudioContent),
    Video(VideoContent),
    Document(DocumentContent),
}

impl From<ContentPart> for ContentPartWire {
    fn from(part: ContentPart) -> Self {
        match part {
            ContentPart::Text(text) => Self::Text { text },
            ContentPart::Image(image) => Self::Image(image),
            ContentPart::Audio(audio) => Self::Audio(audio),
            ContentPart::Video(video) => Self::Video(video),
            ContentPart::Document(document) => Self::Document(document),
        }
    }
}

impl From<ContentPartWire> for ContentPart {
    fn from(wire: ContentPartWire) -> Self {
        match wire {
            ContentPartWire::Text { text } => Self::Text(text),
            ContentPartWire::Image(image) => Self::Image(image),
            ContentPartWire::Audio(audio) => Self::Audio(audio),
            ContentPartWire::Video(video) => Self::Video(video),
            ContentPartWire::Document(document) => Self::Document(document),
        }
    }
}

impl ContentPart {
    /// Constructs a text part.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Returns whether this is a [`ContentPart::Text`] part.
    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Returns the text of a [`ContentPart::Text`] part.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Consumes the part, returning the text of a [`ContentPart::Text`].
    #[must_use]
    pub fn into_text(self) -> Option<String> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }
}

/// Extension methods for a message's content parts.
pub trait ContentExt {
    /// Returns whether every part is text.
    fn is_text(&self) -> bool;

    /// Concatenates the text parts, ignoring any media.
    fn to_text(&self) -> Option<String>;
}

impl ContentExt for Vec<ContentPart> {
    fn is_text(&self) -> bool {
        self.iter().all(ContentPart::is_text)
    }

    fn to_text(&self) -> Option<String> {
        self.iter()
            .filter_map(ContentPart::as_text)
            .map(|s| s.to_owned())
            .reduce(|mut combined, value| {
                combined.push_str(&value);
                combined
            })
    }
}

impl ContentExt for &[ContentPart] {
    fn is_text(&self) -> bool {
        self.iter().all(ContentPart::is_text)
    }

    fn to_text(&self) -> Option<String> {
        self.iter()
            .filter_map(ContentPart::as_text)
            .map(|s| s.to_owned())
            .reduce(|mut combined, value| {
                combined.push_str(&value);
                combined
            })
    }
}

impl From<String> for ContentPart {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for ContentPart {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl From<ImageContent> for ContentPart {
    fn from(image: ImageContent) -> Self {
        Self::Image(image)
    }
}

impl From<AudioContent> for ContentPart {
    fn from(audio: AudioContent) -> Self {
        Self::Audio(audio)
    }
}

impl From<VideoContent> for ContentPart {
    fn from(video: VideoContent) -> Self {
        Self::Video(video)
    }
}

impl From<DocumentContent> for ContentPart {
    fn from(document: DocumentContent) -> Self {
        Self::Document(document)
    }
}

impl std::fmt::Display for ContentPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentPart::Text(text) => f.write_str(text),
            ContentPart::Image(m) => write!(f, "({} image)", m.mime),
            ContentPart::Audio(m) => write!(f, "({} audio)", m.mime),
            ContentPart::Video(m) => write!(f, "({} video)", m.mime),
            ContentPart::Document(m) => write!(f, "({} document)", m.mime),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{ImageMimeType, MediaSource};

    #[test]
    fn round_trips_every_variant_flat() {
        let parts = vec![
            ContentPart::text("hi"),
            ContentPart::Image(
                MediaSource::from_url("https://e.com/a.png").image(ImageMimeType::Png),
            ),
            ContentPart::Text(String::new()),
        ];

        let json = serde_json::to_string(&parts).expect("text must serialize");
        // The tag sits alongside each variant's own fields rather than nesting them.
        assert!(json.starts_with(r#"[{"type":"text","text":"hi"},{"type":"image","source":"#));
        assert_eq!(
            serde_json::from_str::<Vec<ContentPart>>(&json).unwrap(),
            parts
        );
    }
}
