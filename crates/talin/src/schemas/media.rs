use std::sync::Arc;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Serialize};

/// A piece of media attached to a message, tagged with the MIME types its kind allows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaContent<M> {
    /// Where the bytes come from.
    pub source: MediaSource,
    /// The MIME type of the media.
    pub mime: M,
    /// An optional file name, used by providers that display or cite the media.
    pub name: Option<String>,
}

/// An image attached to a message.
pub type ImageContent = MediaContent<ImageMimeType>;
/// An audio clip attached to a message.
pub type AudioContent = MediaContent<AudioMimeType>;
/// A video clip attached to a message.
pub type VideoContent = MediaContent<VideoMimeType>;
/// A document attached to a message.
pub type DocumentContent = MediaContent<DocumentMimeType>;

/// The source from which media content is provided.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MediaSource {
    /// Media content provided directly as raw bytes.
    Bytes(Arc<[u8]>),
    /// A URL from which the provider can fetch the media content.
    Url(String),
    /// The provider-specific identifier of an already uploaded media file.
    FileId(String),
}

impl MediaSource {
    /// Creates a [`MediaSource`] containing the given raw media bytes.
    #[must_use]
    pub fn from_raw(content: Vec<u8>) -> Self {
        Self::Bytes(Arc::from(content))
    }

    /// Creates a [`MediaSource`] by decoding base64-encoded media content.
    ///
    /// Returns an error if `content` is not valid base64.
    pub fn from_base64(content: impl AsRef<[u8]>) -> Result<Self, base64::DecodeError> {
        let content = BASE64_STANDARD.decode(content)?;
        Ok(Self::Bytes(Arc::from(content)))
    }

    /// Creates a [`MediaSource`] referencing media at the given URL.
    #[must_use]
    pub fn from_url(url: impl Into<String>) -> Self {
        Self::Url(url.into())
    }

    /// Creates a [`MediaSource`] referencing an already uploaded media file.
    #[must_use]
    pub fn from_file_id(file_id: impl Into<String>) -> Self {
        Self::FileId(file_id.into())
    }

    /// Returns the media encoded as base64, if it is held as raw bytes.
    #[must_use]
    pub fn to_base64(&self) -> Option<String> {
        match self {
            Self::Bytes(bytes) => Some(BASE64_STANDARD.encode(bytes)),
            Self::Url(_) | Self::FileId(_) => None,
        }
    }

    /// Tags the source as an image of the given MIME type.
    #[must_use]
    pub fn image(self, mime: ImageMimeType) -> ImageContent {
        MediaContent::new(self, mime)
    }

    /// Tags the source as an audio clip of the given MIME type.
    #[must_use]
    pub fn audio(self, mime: AudioMimeType) -> AudioContent {
        MediaContent::new(self, mime)
    }

    /// Tags the source as a video clip of the given MIME type.
    #[must_use]
    pub fn video(self, mime: VideoMimeType) -> VideoContent {
        MediaContent::new(self, mime)
    }

    /// Tags the source as a document of the given MIME type.
    #[must_use]
    pub fn document(self, mime: DocumentMimeType) -> DocumentContent {
        MediaContent::new(self, mime)
    }
}

impl<M> MediaContent<M> {
    /// Creates a new [`MediaContent`] from a source and its MIME type.
    #[must_use]
    pub fn new(source: MediaSource, mime: M) -> Self {
        Self {
            source,
            mime,
            name: None,
        }
    }

    /// Sets the media file name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Declares a MIME type enum, keeping each variant's wire string in a single place.
macro_rules! mime_types {
    ($(
        $(#[$meta:meta])*
        $name:ident { $($variant:ident = $mime:literal),* $(,)? }
    )*) => {$(
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[non_exhaustive]
        pub enum $name {$(
            #[doc = concat!("`\"", $mime, "\"`")]
            #[serde(rename = $mime)]
            $variant,
        )*}

        impl $name {
            /// Returns the MIME type as it appears on the wire.
            #[must_use]
            pub const fn as_str(&self) -> &'static str {
                match self {$(Self::$variant => $mime,)*}
            }
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {$( $mime => Ok(Self::$variant), )* _ => Err(()),}
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    )*};
}

mime_types! {
    /// A media type of an image.
    ImageMimeType {
        Jpeg = "image/jpeg",
        Png = "image/png",
        Gif = "image/gif",
        Webp = "image/webp",
    }

    /// A media type of an audio clip.
    AudioMimeType {
        Mpeg = "audio/mpeg",
        Mp3 = "audio/mp3",
        Wav = "audio/wav",
        Ogg = "audio/ogg",
    }

    /// A media type of a video clip.
    VideoMimeType {
        Mpeg = "video/mpeg",
        Mp4 = "video/mp4",
        Webm = "video/webm",
        Ogg = "video/ogg",
    }

    /// A media type of a document.
    DocumentMimeType {
        Pdf = "application/pdf",
        Doc = "application/msword",
        Xls = "application/vnd.ms-excel",
        Txt = "text/plain",
    }
}
