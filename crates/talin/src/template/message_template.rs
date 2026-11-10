use std::borrow::Cow;
use std::collections::HashSet;

use regex::Regex;

use crate::chain::{MediaReplacements, TextReplacements};
use crate::schemas::{AssistantMessage, ContentExt, ContentPart, Message, UserMessage};
use crate::template::TemplateError;

/// A message template that can be formatted with input variables to produce a [`Message`].
#[derive(Debug, Clone)]
pub struct MessageTemplate {
    /// The template role.
    role: TemplateRole,

    /// The template content.
    content: TemplateContent,
}

/// The role of the template message.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TemplateRole {
    /// A system message.
    System,
    /// A user message.
    User,
    /// An assistant message.
    Assistant,
}

/// The message template format.
#[derive(Debug, Clone)]
enum TemplateFormat {
    /// Python-style f-string formatting. ("{variable}")
    FString,
    /// Jinja2-style formatting. ("{{variable}}")
    Jinja2,
}

/// The content of the message template.
#[derive(Debug, Clone)]
pub struct TemplateContent {
    /// The message template format (jinja2 or fstring).
    format: TemplateFormat,

    /// The template string with placeholders for variables.
    template: String,

    /// The set of variable names required by the template.
    variables: HashSet<String>,

    /// Named media slots whose content parts will be attached to the produced message.
    media: Vec<String>,
}

impl MessageTemplate {
    /// Constructs a new [`MessageTemplate`].
    pub fn new(role: TemplateRole, content: impl Into<TemplateContent>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Constructs a new [`MessageTemplate`] from a Python-style f-string template.
    ///
    /// Call `.role()` to specify a role for the template.
    pub fn fstring(template: impl Into<String>) -> TemplateContent {
        TemplateContent::parse(TemplateFormat::FString, template)
    }

    /// Constructs a new [`MessageTemplate`] from a Jinja2-style template.
    ///
    /// Call `.role()` to specify a role for the template.
    pub fn jinja2(template: impl Into<String>) -> TemplateContent {
        TemplateContent::parse(TemplateFormat::Jinja2, template)
    }

    /// Formats the template with the provided input variables and media replacements, returning a
    /// [`Message`].
    ///
    /// # Errors
    ///
    /// from `input`.
    /// Media slots with no matching entry in `media` are silently skipped.
    pub fn format(
        &self,
        input: &TextReplacements<'_>,
        media: &mut MediaReplacements<'_>,
    ) -> Result<Message, TemplateError> {
        let content = self.content.format(input, media)?;

        Ok(match self.role {
            // System messages don't support media; keep the text alone.
            TemplateRole::System => {
                Message::new_system_message(content.to_text().unwrap_or_default())
            }
            TemplateRole::User => Message::User(UserMessage::from_parts(content)),
            TemplateRole::Assistant => {
                Message::Assistant(AssistantMessage::from_parts(None, content))
            }
        })
    }
}

impl TemplateContent {
    /// Constructs a new [`TemplateContent`].
    fn new(format: TemplateFormat, template: String, variables: HashSet<String>) -> Self {
        Self {
            format,
            template,
            variables,
            media: vec![],
        }
    }

    /// Constructs a new [`TemplateContent`] from a format and template.
    ///
    /// Call [`role`](Self::role) to specify a role for the template.
    fn parse(format: TemplateFormat, template: impl Into<String>) -> Self {
        let template = template.into();
        let re = format.regex();
        let variables = re
            .captures_iter(&template)
            .map(|cap| cap[1].to_string())
            .collect();
        Self::new(format, template, variables)
    }

    /// Formats the template with the provided input variables and media replacements.
    ///
    /// # Errors
    ///
    /// Returns a [`TemplateError::MissingVariable`] if any required text variables are missing
    /// from `input`.
    /// Media slots with no matching entry in `media` are silently skipped.
    fn format(
        &self,
        input: &TextReplacements<'_>,
        media: &mut MediaReplacements<'_>,
    ) -> Result<Vec<ContentPart>, TemplateError> {
        self.validate_input(input)?;

        // Substitute every placeholder in a single pass over the original template, so an inserted
        // value that happens to contain placeholder syntax is never itself substituted into.
        let text = self
            .format
            .regex()
            .replace_all(&self.template, |caps: &regex::Captures<'_>| {
                let name = &caps[1];
                input.get(name).map_or_else(
                    || Cow::Owned(caps[0].to_owned()),
                    |value| Cow::Borrowed(value.as_ref()),
                )
            })
            .into_owned();
        // Collect media from all declared slots, consuming each matched entry.
        let media: Vec<_> = self
            .media
            .iter()
            .filter_map(|slot| media.remove(slot.as_str()))
            .flat_map(Cow::into_owned)
            .collect();

        let content = [ContentPart::Text(text)].into_iter().chain(media).collect();
        Ok(content)
    }

    /// Validates that all required variables are present in the input.
    ///
    /// # Errors
    /// Returns a [`TemplateError::MissingVariable`] if any required variables are missing from the
    /// input.
    fn validate_input(&self, input: &TextReplacements<'_>) -> Result<(), TemplateError> {
        let variables: HashSet<&str> = self.variables.iter().map(String::as_str).collect();
        let input: HashSet<&str> = input.keys().cloned().collect();
        let missing_variables = variables.difference(&input).cloned().collect::<Vec<_>>();

        if !missing_variables.is_empty() {
            return Err(TemplateError::MissingVariable(missing_variables.join(", ")));
        }

        Ok(())
    }

    /// Sets the role for this template content, producing a [`MessageTemplate`].
    pub fn role(self, role: TemplateRole) -> MessageTemplate {
        MessageTemplate::new(role, self)
    }

    /// Attaches media slots to this template.
    ///
    /// # Example
    ///
    /// ```
    /// # use talin::template::MessageTemplate;
    /// let tmpl =
    ///     MessageTemplate::fstring("Describe these: {context}").with_media(["screenshots".into()]);
    /// ```
    pub fn with_media(mut self, media: impl IntoIterator<Item = String>) -> Self {
        self.media = media.into_iter().collect();
        self
    }
}

impl TemplateFormat {
    fn regex(&self) -> Regex {
        match self {
            TemplateFormat::FString => Regex::new(r"\{(\w+)\}").expect("static regex is valid"),
            TemplateFormat::Jinja2 => Regex::new(r"\{\{(\w+)\}\}").expect("static regex is valid"),
        }
    }
}

impl<T: Into<String>> From<T> for TemplateContent {
    fn from(content: T) -> Self {
        MessageTemplate::jinja2(content)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::chain::MediaReplacements;
    use crate::schemas::{ContentExt, DocumentMimeType, ImageMimeType, MediaSource};

    #[rstest]
    #[case(
        "Hello {name}, how are you?",
        TextReplacements::from([("name", "Alice".into())]),
        "Hello Alice, how are you?"
    )]
    // A value that looks like another placeholder must not itself be substituted into, regardless
    // of iteration order.
    #[case(
        "{a}{b}",
        TextReplacements::from([("a", "{b}".into()), ("b", "X".into())]),
        "{b}X"
    )]
    fn test_template_fstring(
        #[case] template: &str,
        #[case] input: TextReplacements<'_>,
        #[case] expected: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template = MessageTemplate::fstring(template);
        let content = template.format(&input, &mut Default::default())?;
        assert_eq!(content.to_text().as_deref(), Some(expected));
        Ok(())
    }

    #[rstest]
    #[case::simple(
        "Hello {{name}}, how are you?",
        TextReplacements::from([("name", "Alice".into())]),
        "Hello Alice, how are you?"
    )]
    #[case::multiple(
        "Hello {{name}}, your order {{order_id}} is confirmed.",
        TextReplacements::from([("name", "Alice".into()), ("order_id", "12345".into())]),
        "Hello Alice, your order 12345 is confirmed."
    )]
    #[case::duplicate(
        "Your order {{id}} is confirmed. Order {{id}} will arrive soon.",
        TextReplacements::from([("id", "12345".into())]),
        "Your order 12345 is confirmed. Order 12345 will arrive soon."
    )]
    #[case::empty(
        "Braces with no variable: {{ }}",
        TextReplacements::new(),
        "Braces with no variable: {{ }}"
    )]
    #[case::weird(
        "Edge {{name}}}} test",
        TextReplacements::from([("name", "Charlie".into())]),
        "Edge Charlie}} test"
    )]

    fn test_template_jinja2(
        #[case] template: &str,
        #[case] input: TextReplacements<'_>,
        #[case] expected: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template = MessageTemplate::jinja2(template);
        let content = template.format(&input, &mut Default::default())?;
        assert_eq!(content.to_text().as_deref(), Some(expected));
        Ok(())
    }

    #[rstest]
    #[case(
        "Hello {{name}}",
        TextReplacements::new() // missing "name"
    )]
    #[case(
        "Hello {{first}} {{last}}",
        TextReplacements::from([("first", "Alice".into())]) // missing "last"
    )]
    fn test_template_jinja2_missing_vars(
        #[case] template: &str,
        #[case] input: TextReplacements<'_>,
    ) {
        let template = MessageTemplate::jinja2(template);
        let result = template.format(&input, &mut Default::default());

        let Err(TemplateError::MissingVariable(_)) = result else {
            panic!("Expected TemplateError::MissingVariable");
        };
    }

    /// Counts the non-text parts of a formatted content.
    fn media_count(content: &[ContentPart]) -> usize {
        content.iter().filter(|part| !part.is_text()).count()
    }

    /// Builds a media replacement value out of any media contents.
    fn media(
        items: impl IntoIterator<Item = impl Into<ContentPart>>,
    ) -> Cow<'static, [ContentPart]> {
        Cow::Owned(items.into_iter().map(Into::into).collect())
    }

    #[test]
    fn test_with_media_attaches_to_human_message() -> Result<(), Box<dyn std::error::Error>> {
        let template =
            MessageTemplate::fstring("Describe: {context}").with_media(["screenshots".into()]);

        let input = TextReplacements::from([("context", "the chart".into())]);
        let screenshots =
            vec![MediaSource::from_url("https://example.com/img.png").image(ImageMimeType::Png)];
        let mut replacements = MediaReplacements::from([("screenshots", media(screenshots))]);

        let content = template.format(&input, &mut replacements)?;

        assert_eq!(content.to_text().as_deref(), Some("Describe: the chart"));
        let [ContentPart::Text(_), ContentPart::Image(image)] = content.as_slice() else {
            panic!("Expected the text followed by an image part");
        };
        let MediaSource::Url(url) = &image.source else {
            panic!("Expected a URL image");
        };
        assert_eq!(url, "https://example.com/img.png");
        Ok(())
    }

    #[test]
    fn test_with_media_attaches_non_image_media() -> Result<(), Box<dyn std::error::Error>> {
        let template = MessageTemplate::fstring("Summarize.").with_media(["files".into()]);

        let files = vec![
            MediaSource::from_url("https://example.com/report.pdf").document(DocumentMimeType::Pdf),
        ];
        let mut replacements = MediaReplacements::from([("files", media(files))]);

        let content = template.format(&TextReplacements::new(), &mut replacements)?;

        assert!(matches!(
            content.as_slice(),
            [ContentPart::Text(_), ContentPart::Document(_)]
        ));
        Ok(())
    }

    #[test]
    fn test_with_media_attaches_to_ai_message() -> Result<(), Box<dyn std::error::Error>> {
        let template = MessageTemplate::fstring("Here is my output.").with_media(["items".into()]);

        let items = vec![
            MediaSource::from_url("https://example.com/a.png").image(ImageMimeType::Png),
            MediaSource::from_url("https://example.com/b.png").image(ImageMimeType::Png),
        ];
        let mut replacements = MediaReplacements::from([("items", media(items))]);

        let content = template.format(&TextReplacements::new(), &mut replacements)?;

        assert_eq!(content.to_text().as_deref(), Some("Here is my output."));
        assert_eq!(media_count(&content), 2);
        Ok(())
    }

    #[test]
    fn test_with_media_multiple_slots_merged() -> Result<(), Box<dyn std::error::Error>> {
        let template = MessageTemplate::fstring("Compare these.")
            .with_media(["before".into(), "after".into()]);

        let before =
            vec![MediaSource::from_url("https://example.com/before.png").image(ImageMimeType::Png)];
        let after =
            vec![MediaSource::from_url("https://example.com/after.png").image(ImageMimeType::Png)];
        let mut replacements =
            MediaReplacements::from([("before", media(before)), ("after", media(after))]);

        let content = template.format(&TextReplacements::new(), &mut replacements)?;

        assert_eq!(content.to_text().as_deref(), Some("Compare these."));
        assert_eq!(media_count(&content), 2);
        Ok(())
    }

    #[test]
    fn test_with_media_missing_slot_is_skipped() -> Result<(), Box<dyn std::error::Error>> {
        let template =
            MessageTemplate::fstring("Describe: {context}").with_media(["optional".into()]);

        let input = TextReplacements::from([("context", "something".into())]);
        // No media provided — the slot is simply absent.
        let mut replacements = MediaReplacements::new();

        let content = template.format(&input, &mut replacements)?;

        assert_eq!(content.to_text().as_deref(), Some("Describe: something"));
        assert_eq!(
            media_count(&content),
            0,
            "Expected no media when slot is absent"
        );
        Ok(())
    }

    #[test]
    fn test_media_slots_consumed_from_replacements() -> Result<(), Box<dyn std::error::Error>> {
        // After formatting, consumed slots should be removed from the map.
        let template = MessageTemplate::fstring("Look at this.").with_media(["pics".into()]);

        let pics =
            vec![MediaSource::from_url("https://example.com/img.png").image(ImageMimeType::Png)];
        let mut replacements = MediaReplacements::from([("pics", media(pics))]);

        template.format(&TextReplacements::new(), &mut replacements)?;

        assert!(
            !replacements.contains_key("pics"),
            "Consumed slot should be removed from the map"
        );
        Ok(())
    }
}
