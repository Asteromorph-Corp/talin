use std::str::FromStr;

use rmcp::model::{ContentBlock, ResourceContents};

use crate::mcp::McpError;
use crate::schemas::{
    AudioContent, AudioMimeType, ContentPart, DocumentContent, DocumentMimeType, ImageContent,
    ImageMimeType, MediaSource,
};

pub(super) fn parse_mcp_contents(
    contents: Vec<ContentBlock>,
) -> Result<Vec<ContentPart>, McpError> {
    contents
        .into_iter()
        .map(parse_mcp_content)
        .collect::<Result<Vec<_>, _>>()
}

/// Parses an MCP content into a [`ContentPart`].
fn parse_mcp_content(content: ContentBlock) -> Result<ContentPart, McpError> {
    let part = match content {
        ContentBlock::Text(t) => ContentPart::Text(t.text),
        ContentBlock::Image(image) => {
            let source = MediaSource::from_base64(image.data.as_bytes()).map_err(|e| {
                McpError::InvalidContent(format!("failed to decode base64 image: {e}"))
            })?;
            let mime = ImageMimeType::from_str(&image.mime_type).map_err(|_| {
                McpError::InvalidContent(format!(
                    "unsupported image MIME type: {}",
                    image.mime_type
                ))
            })?;
            ContentPart::Image(ImageContent::new(source, mime))
        }
        ContentBlock::Audio(audio) => {
            let source = MediaSource::from_base64(audio.data.as_bytes()).map_err(|e| {
                McpError::InvalidContent(format!("failed to decode base64 audio: {e}"))
            })?;
            let mime = AudioMimeType::from_str(&audio.mime_type).map_err(|_| {
                McpError::InvalidContent(format!(
                    "unsupported audio MIME type: {}",
                    audio.mime_type
                ))
            })?;
            ContentPart::Audio(AudioContent::new(source, mime))
        }
        ContentBlock::Resource(content) => match content.resource {
            ResourceContents::TextResourceContents {
                uri,
                mime_type,
                text,
                ..
            } => {
                let mime = mime_type.unwrap_or_else(|| "text/plain".to_string());
                ContentPart::Text(format!("[Resource]({uri}) {mime}:\n{text}"))
            }
            ResourceContents::BlobResourceContents {
                uri,
                mime_type,
                blob,
                ..
            } => {
                let Some(mime_type) = mime_type else {
                    return Err(McpError::invalid_content(
                        "missing MIME type for blob resource",
                    ));
                };

                let source = MediaSource::from_base64(blob.as_bytes()).map_err(|e| {
                    McpError::invalid_content(format!("failed to decode base64 blob: {e}"))
                })?;

                if let Ok(mime) = ImageMimeType::from_str(&mime_type) {
                    ContentPart::Image(ImageContent::new(source, mime).with_name(uri))
                } else if let Ok(mime) = AudioMimeType::from_str(&mime_type) {
                    ContentPart::Audio(AudioContent::new(source, mime).with_name(uri))
                } else if let Ok(mime) = DocumentMimeType::from_str(&mime_type) {
                    ContentPart::Document(DocumentContent::new(source, mime).with_name(uri))
                } else {
                    return Err(McpError::invalid_content(format!(
                        "unsupported resource MIME type: {mime_type}",
                    )));
                }
            }
            _ => return Err(McpError::invalid_content("unsupported resource type")),
        },
        ContentBlock::ResourceLink(_) => {
            return Err(McpError::invalid_content("resource link not supported"));
        }
        _ => return Err(McpError::invalid_content("unsupported `tools/call` body")),
    };
    Ok(part)
}

pub(super) fn parse_mcp_error(contents: Vec<ContentBlock>) -> Result<String, McpError> {
    let mut error_message = String::new();
    for content in contents {
        let ContentBlock::Text(t) = content else {
            continue;
        };
        error_message.push_str(&t.text);
    }
    if error_message.is_empty() {
        return Err(McpError::InvalidContent(
            "missing error message".to_string(),
        ));
    }

    Ok(error_message)
}
