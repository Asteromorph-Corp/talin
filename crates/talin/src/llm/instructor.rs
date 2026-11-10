//! Instructors that provide model-specific adapters for features that models or their APIs do not
//! support natively.
//!
//! See [`Instructor`] and [`CompletionInstructor`] for details.

use crate::schemas::{BodyMessage, LlmStep};
use crate::text::parse::ParseError;
use crate::text::render::render_chatml;
use crate::tools::FunctionSpec;

mod default;
mod qwen3;

pub use default::DefaultInstructor;
use dyn_clone::DynClone;
pub use qwen3::Qwen3Instructor;

/// Defines how tool calls are represented and parsed for a specific model.
///
/// Most model providers now support tool calling natively at the API level.
/// For models that do not, tool calling can still be implemented by prompting the model to
/// emit calls in a specific format and parsing the generated output into structured tool calls.
///
/// The optimal format may vary between models depending on their training data and conventions.
/// The [`Instructor`] trait provides a per-model customization for defining both the prompt format
/// and the corresponding parsing behavior.
///
/// Instructors are used by [`GenericChat`] and [`SglangGenerate`].
///
/// [`GenericChat`]: crate::llm::GenericChat
/// [`SglangGenerate`]: crate::llm::SglangGenerate
pub trait Instructor: DynClone + Send + Sync {
    /// Generates an instruction for using the provided tools.
    ///
    /// The instruction should define the appropriate schema for tool call as well as descriptions
    /// for the available tools. The instruction is appended at the end of the system message.
    fn tool_use_instruction(&self, tools: &[FunctionSpec]) -> String;

    /// Parses the LLM output into a tool call object.
    fn parse_tool_use(&self, output: String) -> Result<LlmStep, ParseError>;
}

/// Defines how a conversation is formatted as a completion prompt for a specific model.
///
/// The crate abstracts the LLM conversation as a sequence of messages.
/// While this representation maps directly to most chat-oriented provider APIs, some model APIs
/// instead accept a single plain-text prompt.
///
/// Because the expected prompt format may vary between models depending on their training data
/// and conventions, [`CompletionInstructor`] provides per-model customization for converting the
/// system prompts and conversation into a single completion prompt.
/// Tool-use instruction and parsing behavior are inherited from [`Instructor`].
///
/// Completion instructors are used by [`SglangGenerate`].
///
/// [`SglangGenerate`]: crate::llm::SglangGenerate
pub trait CompletionInstructor: Instructor {
    /// Formats the system prompts and conversation into a single prompt string.
    ///
    /// The default implementation uses the [ChatML] layout shared by most open chat models.
    ///
    /// [ChatML]: https://github.com/openai/openai-python/blob/120d225b91a8453e15240a49fb1c6794d8119326/chatml.md
    fn format_prompt(&self, system: &[String], messages: &[BodyMessage]) -> String {
        render_chatml(system, messages)
    }
}
