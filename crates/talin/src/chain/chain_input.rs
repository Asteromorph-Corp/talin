use std::borrow::Cow;
use std::collections::HashMap;

pub use talin_macros::ChainInput;

use crate::schemas::{ContentPart, Message};

/// A map of placeholder names to their text replacement values.
///
/// Used by [`ChainInput::text_replacements`] to provide values for template placeholders.
pub type TextReplacements<'a> = HashMap<&'a str, Cow<'a, str>>;

/// A map of media slot names to content part sequences.
///
/// Used by [`ChainInput::media_replacements`] to attach images, audio, video, or documents to
/// messages produced by templates that declare media slots.
pub type MediaReplacements<'a> = HashMap<&'a str, Cow<'a, [ContentPart]>>;

/// A map of placeholder names to message sequences.
///
/// Used by [`ChainInput::placeholder_replacements`] to provide message history or
/// other structured content for template placeholders.
pub type PlaceholderReplacements<'a> = HashMap<&'a str, Cow<'a, [Message]>>;

/// A trait for types that can be used as input to a chain.
///
/// This trait provides methods to extract replacement values from the input,
/// which are then used to fill in template placeholders in prompts.
///
/// # Deriving
///
/// Use the `#[derive(ChainInput)]` macro to automatically implement this trait
/// for custom structs.
///
/// # Built-in Implementations
///
/// - `String` and `&str`: Map to a single `"input"` placeholder.
/// - `()`: Provides no replacements (empty input).
/// - `HashMap<K, V>`: Provides key-value replacements directly.
pub trait ChainInput: Send + Sync {
    /// Returns the text replacements for template placeholders.
    ///
    /// Each entry maps a placeholder name (without braces) to its replacement value.
    fn text_replacements(&self) -> TextReplacements<'_>;

    /// Returns media replacements for template media slots.
    ///
    /// This is used for attaching media to messages produced by templates that
    /// declare media slots.
    fn media_replacements(&self) -> MediaReplacements<'_> {
        HashMap::new()
    }

    /// Returns message sequence replacements for template placeholders.
    ///
    /// This is used for inserting conversation history or other message sequences
    /// into prompts. Defaults to an empty map.
    fn placeholder_replacements(&self) -> PlaceholderReplacements<'_> {
        HashMap::new()
    }
}

/// Built-in implementation of `ChainInput` for `String`.
impl ChainInput for String {
    /// Maps to a single `"input"` placeholder.
    fn text_replacements(&self) -> TextReplacements<'_> {
        TextReplacements::from_iter([("input", Cow::Borrowed(self.as_str()))])
    }
}

/// Built-in implementation of `ChainInput` for `&str`.
impl ChainInput for &'_ str {
    /// Maps to a single `"input"` placeholder.
    fn text_replacements(&self) -> TextReplacements<'_> {
        TextReplacements::from_iter([("input", Cow::Borrowed(*self))])
    }
}

/// Built-in implementation of `ChainInput` for the unit type `()`.
impl ChainInput for () {
    /// Provides no replacements (empty input).
    fn text_replacements(&self) -> TextReplacements<'_> {
        HashMap::new()
    }
}

/// Built-in implementation of `ChainInput` for `HashMap<String, String>`.
impl ChainInput for HashMap<String, String> {
    /// Provides key-value replacements directly.
    fn text_replacements(&self) -> TextReplacements<'_> {
        self.iter()
            .map(|(k, v)| (k.as_str(), Cow::Borrowed(v.as_str())))
            .collect()
    }
}

/// Built-in implementation of `ChainInput` for `HashMap<&str, String>`.
impl ChainInput for HashMap<&str, String> {
    /// Provides key-value replacements directly.
    fn text_replacements(&self) -> TextReplacements<'_> {
        self.iter()
            .map(|(&k, v)| (k, Cow::Borrowed(v.as_str())))
            .collect()
    }
}

/// Built-in implementation of `ChainInput` for `HashMap<String, &str>`.
impl ChainInput for HashMap<&str, &str> {
    /// Provides key-value replacements directly.
    fn text_replacements(&self) -> TextReplacements<'_> {
        self.iter().map(|(&k, &v)| (k, Cow::Borrowed(v))).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{
        ContentExt, ContentPart, DocumentContent, DocumentMimeType, ImageContent, ImageMimeType,
        MediaSource, UserMessage,
    };
    use crate::template::{MessageTemplate, PromptTemplate, TemplateRole};

    /// Counts the image parts carried by a user message.
    fn image_count(message: &UserMessage) -> usize {
        message
            .content
            .iter()
            .filter(|part| matches!(part, ContentPart::Image(_)))
            .count()
    }

    #[derive(ChainInput)]
    struct InputWithImages {
        #[chain(into = "text")]
        context: String,
        #[chain(into = "images")]
        screenshots: Vec<ImageContent>,
    }

    #[derive(ChainInput)]
    struct InputWithMedia {
        #[chain(into = "text")]
        context: String,
        #[chain(into = "documents")]
        files: Vec<DocumentContent>,
        #[chain(into = "media")]
        mixed: Vec<ContentPart>,
    }

    #[derive(ChainInput)]
    struct InputWithOptionalImages {
        #[chain(into = "text")]
        question: String,
        #[chain(into = "images")]
        attachments: Option<Vec<ImageContent>>,
    }

    #[test]
    fn test_derive_media_replacements_vec() {
        let input = InputWithImages {
            context: "the dashboard".into(),
            screenshots: vec![
                MediaSource::from_url("https://example.com/dash.png").image(ImageMimeType::Png),
            ],
        };

        let replacements = input.media_replacements();
        assert!(replacements.contains_key("screenshots"));
        assert_eq!(replacements["screenshots"].len(), 1);
    }

    #[test]
    fn test_derive_media_replacements_empty_vec() {
        let input = InputWithImages {
            context: "nothing".into(),
            screenshots: vec![],
        };

        let replacements = input.media_replacements();
        assert!(replacements.contains_key("screenshots"));
        assert_eq!(replacements["screenshots"].len(), 0);
    }

    #[test]
    fn test_derive_media_replacements_option_some() {
        let input = InputWithOptionalImages {
            question: "What is this?".into(),
            attachments: Some(vec![
                MediaSource::from_url("https://example.com/a.png").image(ImageMimeType::Png),
                MediaSource::from_url("https://example.com/b.png").image(ImageMimeType::Png),
            ]),
        };

        let replacements = input.media_replacements();
        assert!(replacements.contains_key("attachments"));
        assert_eq!(replacements["attachments"].len(), 2);
    }

    #[test]
    fn test_derive_media_replacements_option_none() {
        let input = InputWithOptionalImages {
            question: "What is this?".into(),
            attachments: None,
        };

        let replacements = input.media_replacements();
        assert!(replacements.contains_key("attachments"));
        assert_eq!(replacements["attachments"].len(), 0);
    }

    #[test]
    fn test_derive_media_replacements_mixed_kinds() {
        let input = InputWithMedia {
            context: "the report".into(),
            files: vec![
                MediaSource::from_url("https://example.com/report.pdf")
                    .document(DocumentMimeType::Pdf),
            ],
            mixed: vec![
                MediaSource::from_url("https://example.com/clip.png")
                    .image(ImageMimeType::Png)
                    .into(),
            ],
        };

        let replacements = input.media_replacements();
        assert!(matches!(
            replacements["files"].as_ref(),
            [ContentPart::Document(_)]
        ));
        assert!(matches!(
            replacements["mixed"].as_ref(),
            [ContentPart::Image(_)]
        ));
    }

    #[test]
    fn test_derive_text_replacements_unaffected() {
        let input = InputWithImages {
            context: "hello".into(),
            screenshots: vec![],
        };

        let text = input.text_replacements();
        assert_eq!(text["context"], "hello");
        assert!(!text.contains_key("screenshots"));
    }

    #[test]
    fn test_prompt_template_round_trip_with_images() -> Result<(), Box<dyn std::error::Error>> {
        let template = MessageTemplate::fstring("Describe: {context}")
            .with_media(["screenshots".into()])
            .role(TemplateRole::User);

        let prompt_template = PromptTemplate::from(template);

        let input = InputWithImages {
            context: "the chart".into(),
            screenshots: vec![
                MediaSource::from_url("https://example.com/chart.png").image(ImageMimeType::Png),
            ],
        };

        let prompt = prompt_template.format(&input)?;
        let messages = prompt.to_messages();

        assert_eq!(messages.len(), 1);

        let Message::User(user_msg) = &messages[0] else {
            panic!("Expected a User message");
        };
        assert_eq!(
            user_msg.content.to_text().as_deref(),
            Some("Describe: the chart")
        );
        assert_eq!(image_count(user_msg), 1);

        Ok(())
    }

    #[test]
    fn test_prompt_template_images_bound_to_correct_message()
    -> Result<(), Box<dyn std::error::Error>> {
        // Two templates in one prompt: only the one declaring the slot gets the images.
        use crate::prompt_template;

        let system_tmpl =
            MessageTemplate::fstring("You are a helpful assistant.").role(TemplateRole::System);
        let human_tmpl = MessageTemplate::fstring("Describe: {context}")
            .with_media(["screenshots".into()])
            .role(TemplateRole::User);

        let prompt_template = prompt_template![system_tmpl, human_tmpl];

        let input = InputWithImages {
            context: "the map".into(),
            screenshots: vec![
                MediaSource::from_url("https://example.com/map.png").image(ImageMimeType::Png),
            ],
        };

        let prompt = prompt_template.format(&input)?;
        let messages = prompt.to_messages();

        assert_eq!(messages.len(), 2);

        // System message must not carry images.
        let Message::System(_) = &messages[0] else {
            panic!("Expected System message first");
        };

        // Human message must carry the image.
        let Message::User(user_msg) = &messages[1] else {
            panic!("Expected User message second");
        };
        assert_eq!(image_count(user_msg), 1);

        Ok(())
    }
}
