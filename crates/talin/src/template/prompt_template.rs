use super::MessageTemplate;
use crate::chain::{ChainInput, MediaReplacements, PlaceholderReplacements, TextReplacements};
use crate::schemas::{AssistantMessage, Message, Prompt, SystemMessage, UserMessage};
use crate::template::TemplateError;

/// A prompt template consisting of a sequence of messages, message templates, and placeholders.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub(crate) messages: Vec<MessageOrTemplate>,
}

/// A concrete message, a message template, or a placeholder for dynamic insertion of messages.
#[derive(Debug, Clone)]
pub enum MessageOrTemplate {
    /// A concrete message.
    Message(Message),
    /// A message template.
    Template(MessageTemplate),
    /// A placeholder for dynamic insertion of messages.
    Placeholder(String),
}

impl PromptTemplate {
    /// Create a new PromptTemplate from a list of messages, message templates, and placeholders.
    pub fn new(messages: impl IntoIterator<Item = MessageOrTemplate>) -> Self {
        Self {
            messages: messages.into_iter().collect(),
        }
    }

    /// Insert variables into a prompt template to create a full-fletched prompt.
    ///
    /// replace_placeholder() must be called before format().
    pub(crate) fn format(&self, input: &impl ChainInput) -> Result<Prompt, TemplateError> {
        let texts = input.text_replacements();
        let mut messages = input.placeholder_replacements();
        let mut media = input.media_replacements();

        let messages = self
            .messages
            .iter()
            .map(|m| m.format(&texts, &mut messages, &mut media))
            .collect::<Result<Vec<Vec<Message>>, TemplateError>>()?
            .concat();

        Ok(Prompt::new(messages))
    }
}

impl MessageOrTemplate {
    pub(crate) fn format(
        &self,
        texts: &TextReplacements<'_>,
        messages: &mut PlaceholderReplacements<'_>,
        media: &mut MediaReplacements<'_>,
    ) -> Result<Vec<Message>, TemplateError> {
        let msgs = match self {
            MessageOrTemplate::Message(m) => vec![m.clone()],
            MessageOrTemplate::Template(t) => vec![t.format(texts, media)?],
            MessageOrTemplate::Placeholder(p) => match messages.remove(p.as_str()) {
                Some(messages) => messages.into_owned(),
                None => vec![],
            },
        };
        Ok(msgs)
    }
}

impl From<MessageTemplate> for PromptTemplate {
    fn from(template: MessageTemplate) -> Self {
        Self::new(vec![MessageOrTemplate::Template(template)])
    }
}

impl From<Message> for MessageOrTemplate {
    fn from(message: Message) -> Self {
        MessageOrTemplate::Message(message)
    }
}

impl From<SystemMessage> for MessageOrTemplate {
    fn from(message: SystemMessage) -> Self {
        MessageOrTemplate::Message(Message::System(message))
    }
}

impl From<UserMessage> for MessageOrTemplate {
    fn from(message: UserMessage) -> Self {
        MessageOrTemplate::Message(Message::User(message))
    }
}

impl From<AssistantMessage> for MessageOrTemplate {
    fn from(message: AssistantMessage) -> Self {
        MessageOrTemplate::Message(Message::Assistant(message))
    }
}

impl From<MessageTemplate> for MessageOrTemplate {
    fn from(template: MessageTemplate) -> Self {
        MessageOrTemplate::Template(template)
    }
}

/// Macro to create a PromptTemplate from a list of messages, message templates, and placeholders.
#[macro_export]
macro_rules! prompt_template {
    ($($x:expr),* $(,)?) => {
        $crate::template::PromptTemplate::new(vec![$($x.into()),*])
    };
}
