use std::collections::HashMap;
use std::convert::Infallible;

use async_trait::async_trait;
use futures::future::try_join_all;

use crate::schemas::{
    AssistantMessage, BodyMessage, ContentPart, ToolCallMessage, ToolMessage, ToolOutput,
    UserMessage,
};
use crate::transcript::edit::{EntryEditScheme, SystemEditScheme};
use crate::transcript::{EntryEdit, EntryView};

/// Renders every `{{key}}` placeholder in the entries against a [`PlaceholderResolver`], preserving
/// each entry's provider-specific native base (`retain = true`).
///
/// This edit scheme is meant to be used in
/// [`project_transcript`](crate::strategy::Strategy::project_transcript) implementation.
/// Note that using this in [`edit_transcript`](crate::strategy::Strategy::edit_transcript) will
/// mutate the canonical entries, and remove the placeholder for the future.
pub struct Interpolate<'a, R: PlaceholderResolver + ?Sized>(pub &'a R);

#[async_trait]
impl<R: PlaceholderResolver + ?Sized> SystemEditScheme for Interpolate<'_, R> {
    type Error = R::Error;

    async fn edit_system(&self, contents: &[String]) -> Result<Option<Vec<String>>, Self::Error> {
        self.0.render_texts(contents).await
    }
}

#[async_trait]
impl<R: PlaceholderResolver + ?Sized> EntryEditScheme for Interpolate<'_, R> {
    type Error = R::Error;

    async fn edit(&self, views: Vec<EntryView<'_>>) -> Result<Vec<EntryEdit>, Self::Error> {
        let futures = views.into_iter().map(|view| async move {
            let EntryView::Message(id, message) = view else {
                return Ok(EntryEdit::from(view));
            };
            match self.0.render_message(message).await? {
                Some(rendered) => Ok(EntryEdit::Replace(id, rendered, true)),
                None => Ok(EntryEdit::Keep(id)),
            }
        });
        try_join_all(futures).await
    }
}

/// Resolves a `{{key}}` placeholder to its current value.
///
/// Used by [`Interpolate`] to render placeholders in a transcript.
///
/// # Implementations
///
/// - `HashMap<String, String>` — resolves keys to their corresponding values in the map.
/// - `Fn(&str) -> Result<Option<String>, E>` — resolves keys using the provided function.
#[async_trait]
pub trait PlaceholderResolver: Send + Sync {
    /// The error returned when resolving a key fails.
    type Error: std::error::Error + Send + Sync;

    /// Returns the value for `key`, or `None` to leave the placeholder intact.
    async fn resolve(&self, key: &str) -> Result<Option<String>, Self::Error>;
}

#[async_trait]
impl PlaceholderResolver for HashMap<String, String> {
    type Error = Infallible;

    async fn resolve(&self, key: &str) -> Result<Option<String>, Self::Error> {
        Ok(self.get(key).cloned())
    }
}

#[async_trait]
impl<F, E> PlaceholderResolver for F
where
    F: Fn(&str) -> Result<Option<String>, E> + Send + Sync,
    E: std::error::Error + Send + Sync,
{
    type Error = E;

    async fn resolve(&self, key: &str) -> Result<Option<String>, Self::Error> {
        self(key)
    }
}

/// A helper trait that extends [`PlaceholderResolver`] with methods for rendering text, messages,
/// and tool data.
#[async_trait]
trait PlaceholderResolverExt: PlaceholderResolver {
    /// Substitutes `{{key}}` occurrences in `template` using `resolver`, returning the rendered
    /// string only if it differs from the input. Unknown keys are left intact.
    async fn render_text(&self, template: &str) -> Result<Option<String>, Self::Error> {
        if !template.contains("{{") {
            return Ok(None);
        }

        let mut out = String::with_capacity(template.len());
        let mut rest = template;
        while let Some(open) = rest.find("{{") {
            out.push_str(&rest[..open]);
            let after = &rest[open + 2..];
            // Unterminated `{{` — emit the remainder verbatim and stop.
            let Some(close) = after.find("}}") else {
                rest = &rest[open..];
                break;
            };
            let key = after[..close].trim();
            match self.resolve(key).await? {
                Some(value) => out.push_str(&value),
                None => out.push_str(&rest[open..open + 2 + close + 2]),
            }
            rest = &after[close + 2..];
        }
        out.push_str(rest);

        if out == template {
            return Ok(None);
        }
        Ok(Some(out))
    }

    async fn render_part(&self, part: &ContentPart) -> Result<Option<ContentPart>, Self::Error> {
        let ContentPart::Text(text) = part else {
            return Ok(None);
        };
        let Some(rendered) = self.render_text(text).await? else {
            return Ok(None);
        };
        Ok(Some(ContentPart::Text(rendered)))
    }

    /// Renders each of `texts`, returning a new vector only if at least one entry changed.
    async fn render_texts(&self, texts: &[String]) -> Result<Option<Vec<String>>, Self::Error> {
        let rendered = try_join_all(texts.iter().map(|c| self.render_text(c))).await?;

        if rendered.iter().all(Option::is_none) {
            return Ok(None);
        }

        let out = texts
            .iter()
            .zip(rendered)
            .map(|(orig, r)| r.unwrap_or_else(|| orig.clone()))
            .collect();
        Ok(Some(out))
    }

    async fn render_parts(
        &self,
        parts: &[ContentPart],
    ) -> Result<Option<Vec<ContentPart>>, Self::Error> {
        let rendered = try_join_all(parts.iter().map(|p| self.render_part(p))).await?;

        if rendered.iter().all(Option::is_none) {
            return Ok(None);
        }

        let out = parts
            .iter()
            .zip(rendered)
            .map(|(orig, r)| r.unwrap_or_else(|| orig.clone()))
            .collect();
        Ok(Some(out))
    }

    async fn render_thought(&self, thought: Option<&str>) -> Result<Option<String>, Self::Error> {
        match thought {
            Some(t) => self.render_text(t).await,
            None => Ok(None),
        }
    }

    /// Renders the text held in a [`ContentPart`], returning new data only if anything changed.
    async fn render_tool_data(
        &self,
        content: &[ContentPart],
    ) -> Result<Option<Vec<ContentPart>>, Self::Error> {
        let rendered = try_join_all(content.iter().map(|item| self.render_part(item))).await?;
        if rendered.iter().all(Option::is_none) {
            return Ok(None);
        }

        let items = content
            .iter()
            .zip(rendered)
            .map(|(orig, r)| r.unwrap_or_else(|| orig.clone()))
            .collect();
        Ok(Some(items))
    }

    async fn render_user_message(
        &self,
        message: &UserMessage,
    ) -> Result<Option<UserMessage>, Self::Error> {
        let Some(content) = self.render_parts(&message.content).await? else {
            return Ok(None);
        };
        Ok(Some(UserMessage { content }))
    }

    async fn render_assistant_message(
        &self,
        message: &AssistantMessage,
    ) -> Result<Option<AssistantMessage>, Self::Error> {
        let thought = self.render_thought(message.thought.as_deref()).await?;
        let content = self.render_parts(&message.content).await?;

        if thought.is_none() && content.is_none() {
            return Ok(None);
        }
        Ok(Some(AssistantMessage {
            thought: thought.or_else(|| message.thought.clone()),
            content: content.unwrap_or_else(|| message.content.clone()),
        }))
    }

    async fn render_tool_call_message(
        &self,
        message: &ToolCallMessage,
    ) -> Result<Option<ToolCallMessage>, Self::Error> {
        let thought = self.render_thought(message.thought.as_deref()).await?;
        let content = self.render_parts(&message.content).await?;

        Ok(Some(ToolCallMessage {
            thought: thought.or_else(|| message.thought.clone()),
            content: content.unwrap_or_else(|| message.content.clone()),
            tool_calls: message.tool_calls.clone(),
        }))
    }

    async fn render_tool_message(
        &self,
        message: &ToolMessage,
    ) -> Result<Option<ToolMessage>, Self::Error> {
        let Some(content) = self.render_tool_data(&message.output.content).await? else {
            return Ok(None);
        };

        Ok(Some(ToolMessage {
            call_id: message.call_id.clone(),
            name: message.name.clone(),
            output: ToolOutput {
                content,
                summary: message.output.summary.clone(),
            },
        }))
    }

    /// Renders the text content of a [`BodyMessage`], returning a new message only if anything
    /// changed.
    async fn render_message(
        &self,
        message: &BodyMessage,
    ) -> Result<Option<BodyMessage>, Self::Error> {
        let out = match message {
            BodyMessage::User(m) => self.render_user_message(m).await?.map(BodyMessage::User),
            BodyMessage::Assistant(m) => self
                .render_assistant_message(m)
                .await?
                .map(BodyMessage::Assistant),
            BodyMessage::ToolCall(m) => self
                .render_tool_call_message(m)
                .await?
                .map(BodyMessage::ToolCall),
            BodyMessage::Tool(m) => self.render_tool_message(m).await?.map(BodyMessage::Tool),
        };
        Ok(out)
    }
}

impl<T: ?Sized> PlaceholderResolverExt for T where T: PlaceholderResolver {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::Interpolate;
    use crate::schemas::{BodyMessage, ContentExt, UserMessage};
    use crate::transcript::edit::{EntryEditScheme, SystemEditScheme};
    use crate::transcript::{AgentTranscript, EntryEdit, EntryView};

    fn resolver() -> HashMap<String, String> {
        HashMap::from([("j1".to_string(), "complete".to_string())])
    }

    fn user_msg(text: &str) -> BodyMessage {
        BodyMessage::User(UserMessage::new(text))
    }

    fn keep_id(edit: &EntryEdit) -> Option<Uuid> {
        match edit {
            EntryEdit::Keep(id) => Some(*id),
            _ => None,
        }
    }

    #[tokio::test]
    async fn renders_system() {
        let map = resolver();
        let scheme = Interpolate(&map);

        assert_eq!(
            scheme
                .edit_system(&["status: {{j1}}".to_string()])
                .await
                .unwrap(),
            Some(vec!["status: complete".to_string()]),
        );
        // No placeholder anywhere → no change.
        assert_eq!(
            scheme
                .edit_system(&["static line".to_string()])
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn renders_key() {
        let map = resolver();
        let body = user_msg("job {{j1}} done");
        let id = Uuid::new_v4();

        let edits = Interpolate(&map)
            .edit(vec![EntryView::Message(id, &body)])
            .await
            .unwrap();

        let EntryEdit::Replace(replaced_id, rendered, retain) = &edits[0] else {
            panic!("expected Replace");
        };
        assert_eq!(*replaced_id, id);
        assert!(*retain); // native base reused
        let BodyMessage::User(u) = rendered else {
            panic!("expected a user body");
        };
        assert_eq!(u.content.to_text().as_deref(), Some("job complete done"));
    }

    #[tokio::test]
    async fn keeps_no_key() {
        let map = resolver();
        let body = user_msg("plain text");
        let id = Uuid::new_v4();

        let edits = Interpolate(&map)
            .edit(vec![EntryView::Message(id, &body)])
            .await
            .unwrap();

        assert_eq!(keep_id(&edits[0]), Some(id));
    }

    #[tokio::test]
    async fn keeps_unknown_keys() {
        let map = resolver();
        let body = user_msg("job {{unknown}} done");
        let id = Uuid::new_v4();

        // Unknown key isn't resolved, so the content is unchanged and the entry is kept as-is.
        let edits = Interpolate(&map)
            .edit(vec![EntryView::Message(id, &body)])
            .await
            .unwrap();

        assert_eq!(keep_id(&edits[0]), Some(id));
    }

    #[tokio::test]
    async fn applies_edit() {
        let mut transcript = AgentTranscript::<Vec<String>, BodyMessage>::new(
            vec!["status: {{j1}}".to_string()],
            vec![user_msg("job {{j1}} done")],
        )
        .unwrap();

        transcript
            .apply_edits(&Interpolate(&resolver()))
            .await
            .unwrap();

        // Both passes commit, so the system prompt and the entry are rendered in place.
        let (system, items) = transcript.materialize();
        assert_eq!(system, vec!["status: complete".to_string()]);
        assert!(
            matches!(&items[0], BodyMessage::User(u) if u.content.to_text().as_deref() == Some("job complete done"))
        );
    }
}
