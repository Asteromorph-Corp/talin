use talin_provider::openai_chat::Config;

use crate::llm::instructor::{CompletionInstructor, Instructor};
use crate::llm::{GenericChat, Llm, LlmOptions, LlmRequestOptions, SglangGenerate};

/// The model preset, which consists of the model, strategy, and default options.
#[derive(Clone)]
pub struct LlmPreset<L: Llm, S> {
    /// The model.
    pub model: L,
    /// The strategy used with the model.
    pub strategy: S,
    /// Default call options for the model.
    pub options: L::Options,
}

impl<L: Llm, S> LlmPreset<L, S> {
    /// Creates a new [`LlmPreset`] with the given model and preset.
    pub fn new(model: L, strategy: S) -> Self {
        Self {
            model,
            strategy,
            options: L::Options::from_options(LlmOptions::default()),
        }
    }

    /// Merges `options` into the preset's options.
    ///
    /// For any conflicting options, the new values are preferred over existing values.
    pub fn with_options(mut self, options: impl Into<L::Options>) -> Self {
        self.options = options.into().merge(self.options);
        self
    }
}

impl<C: Config + Clone + Send + Sync + 'static, S> LlmPreset<GenericChat<C>, S> {
    /// Sets the [instructor](`Instructor`) for the model.
    ///
    /// The instructor is used to create tool use instruction and parse tool calls.
    pub fn with_instructor(mut self, instructor: impl Instructor + 'static) -> Self {
        self.model = self.model.with_instructor(instructor);
        self
    }
}

impl<S> LlmPreset<SglangGenerate, S> {
    /// Sets the [instructor](`CompletionInstructor`) for the model.
    ///
    /// The instructor is used to format prompts, create tool use instruction, and parse tool calls.
    pub fn with_instructor(mut self, instructor: impl CompletionInstructor + 'static) -> Self {
        self.model = self.model.with_instructor(instructor);
        self
    }
}
