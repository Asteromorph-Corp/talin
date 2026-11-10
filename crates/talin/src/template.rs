//! A module for message templates that can be used to generate messages with dynamic content.

mod error;
mod message_template;
mod prompt_template;

pub use error::TemplateError;
pub(crate) use message_template::TemplateContent;
pub use message_template::{MessageTemplate, TemplateRole};
pub use prompt_template::{MessageOrTemplate, PromptTemplate};
