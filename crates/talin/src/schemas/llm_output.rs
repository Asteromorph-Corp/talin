use crate::chain::{ChainOutput, Ctor, PartialOutput};
use crate::schemas::{
    AssistantMessage, BodyMessage, ContentPart, Message, ToolCall, ToolCallMessage, WithUsage,
};
use crate::utils::NoneIfEmpty;

/// LLM output with usage information.
pub type LlmOutput = WithUsage<LlmStep>;

/// A single LLM step, either a final response or a tool call.
#[derive(Debug, Clone, Ctor, PartialEq)]
pub enum LlmStep {
    /// A tool call step.
    ToolCall(ToolCallMessage),
    /// A text step. This is interpreted as the final response in the agent loop.
    Finish(AssistantMessage),
}

/// A step cut short by the output token limit, reduced to the part that can be continued from.
///
/// [`TruncatedStep::concat`] joins it with its continuation.
#[derive(Debug, Clone, PartialEq)]
pub struct TruncatedStep(pub AssistantMessage);

impl LlmStep {
    /// Creates a text step with the provided contents.
    pub fn new_text(thought: Option<String>, text: String) -> Self {
        LlmStep::Finish(AssistantMessage::new(thought, text))
    }

    /// Creates a tool call step with the provided contents.
    pub fn new_tool_calls(
        thought: Option<String>,
        tool_calls: impl IntoIterator<Item = ToolCall>,
    ) -> Self {
        LlmStep::ToolCall(ToolCallMessage {
            thought,
            content: vec![],
            tool_calls: tool_calls.into_iter().collect(),
        })
    }

    /// Creates a new step from the provided contents.
    ///
    /// If `tool_calls` is non-empty, a tool call step is created; otherwise, a text step is
    /// created.
    pub fn from_parts(
        thoughts: Vec<String>,
        texts: Vec<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        let thought = thoughts.join("").none_if_empty();
        let parts = texts.into_iter().map(ContentPart::Text).collect::<Vec<_>>();
        if tool_calls.is_empty() {
            LlmStep::Finish(AssistantMessage::from_parts(thought, parts))
        } else {
            LlmStep::ToolCall(ToolCallMessage::from_parts(thought, parts, tool_calls))
        }
    }

    /// Returns `thought` of the step, if any.
    pub fn thought(&self) -> Option<&str> {
        match &self {
            LlmStep::ToolCall(m) => m.thought.as_deref(),
            LlmStep::Finish(m) => m.thought.as_deref(),
        }
    }

    /// Sets the `thought` for the step.
    pub fn with_thought(mut self, thought: impl Into<String>) -> Self {
        let thought = thought.into();
        match &mut self {
            LlmStep::ToolCall(m) => m.thought = Some(thought),
            LlmStep::Finish(m) => m.thought = Some(thought),
        }
        self
    }

    /// Drops the `thought` from the step.
    pub fn drop_thought(mut self) -> Self {
        match &mut self {
            LlmStep::ToolCall(m) => m.thought = None,
            LlmStep::Finish(m) => m.thought = None,
        }
        self
    }

    /// Returns the part of a step cut short by the output token limit that can be continued from.
    ///
    /// A text message is returned as-is, while a tool call message is trimmed down to the text the
    /// model wrote before the tool call.
    pub fn into_truncated(self) -> Option<TruncatedStep> {
        let msg = match self {
            LlmStep::Finish(m) => m,
            LlmStep::ToolCall(m) => AssistantMessage::new(None, m.thought?),
        };
        Some(TruncatedStep(msg))
    }
}

impl TruncatedStep {
    /// Appends a continuation produced after this step was cut short by the output token limit.
    ///
    /// The two texts are joined verbatim: the model was interrupted mid-sentence, so the
    /// continuation picks up where this step left off.
    pub fn concat(self, continuation: LlmStep) -> LlmStep {
        match continuation {
            LlmStep::Finish(mut m) => {
                m.thought = join_thoughts(self.0.thought, m.thought);
                m.content = [self.0.content, m.content].concat();
                LlmStep::Finish(m)
            }
            // A step carrying both text and tool calls keeps the text as its thought.
            LlmStep::ToolCall(mut m) => {
                m.thought = join_thoughts(self.0.thought, m.thought);
                m.content = [self.0.content, m.content].concat();
                LlmStep::ToolCall(m)
            }
        }
    }
}

/// Joins the thoughts of a step and its continuation, which come from separate model calls.
fn join_thoughts(thought: Option<String>, continuation: Option<String>) -> Option<String> {
    match (thought, continuation) {
        (Some(thought), Some(continuation)) => Some(format!("{thought}\n---\n{continuation}")),
        (thought, continuation) => thought.or(continuation),
    }
}

impl From<ToolCallMessage> for LlmStep {
    fn from(message: ToolCallMessage) -> Self {
        LlmStep::ToolCall(message)
    }
}

impl From<AssistantMessage> for LlmStep {
    fn from(message: AssistantMessage) -> Self {
        LlmStep::Finish(message)
    }
}

impl From<LlmStep> for BodyMessage {
    fn from(step: LlmStep) -> Self {
        match step {
            LlmStep::ToolCall(m) => BodyMessage::ToolCall(m),
            LlmStep::Finish(m) => BodyMessage::Assistant(m),
        }
    }
}

impl From<TruncatedStep> for BodyMessage {
    fn from(step: TruncatedStep) -> Self {
        BodyMessage::Assistant(step.0)
    }
}

impl From<LlmStep> for Message {
    fn from(step: LlmStep) -> Self {
        match step {
            LlmStep::ToolCall(m) => Message::ToolCall(m),
            LlmStep::Finish(m) => Message::Assistant(m),
        }
    }
}

impl ChainOutput for LlmStep {
    type Partial = Self;
}

impl PartialOutput for LlmStep {
    fn from_text(text: &str) -> Result<Self, crate::text::parse::ParseError> {
        Ok(LlmStep::new_text(None, text.to_owned()))
    }
}

impl std::fmt::Display for LlmStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmStep::ToolCall(m) => write!(f, "{m}"),
            LlmStep::Finish(m) => write!(f, "{m}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{ContentExt, ContentPart};

    fn tool_call() -> ToolCall {
        ToolCall::new(Some("call_1".to_owned()), "search", None)
    }

    #[test]
    fn truncated_prefix_keeps_the_text() {
        let msg = AssistantMessage::new(Some("thinking".to_owned()), "The answer is".to_owned());
        let step = LlmStep::Finish(msg.clone());
        let expected = TruncatedStep(msg);

        assert_eq!(step.clone().into_truncated(), Some(expected));
    }

    #[test]
    fn truncated_prefix_drops_the_tool_calls() {
        let step = LlmStep::new_tool_calls(Some("Let me search.".to_owned()), [tool_call()]);
        let truncated = step.into_truncated().expect("text is kept");

        let expected = AssistantMessage::new(None, "Let me search.".to_owned());
        assert_eq!(truncated, TruncatedStep(expected));
    }

    #[test]
    fn truncated_prefix_of_a_textless_step_is_none() {
        let step = LlmStep::new_tool_calls(None, [tool_call()]);
        assert_eq!(step.into_truncated(), None);
    }

    #[test]
    fn concat_joins_the_texts_verbatim() {
        let step = LlmStep::new_text(Some("first".to_owned()), "The answer ".to_owned());
        let continuation = LlmStep::new_text(Some("second".to_owned()), "is 42.".to_owned());

        let expected = AssistantMessage::from_parts(
            Some("first\n---\nsecond".to_owned()),
            [
                ContentPart::text("The answer "),
                ContentPart::text("is 42."),
            ],
        );
        let LlmStep::Finish(joined) = step.into_truncated().unwrap().concat(continuation) else {
            panic!("expected a text step");
        };
        assert_eq!(joined, expected);
        // The halves stay separate parts, but they read back as one verbatim string.
        assert_eq!(
            joined.content.to_text().as_deref(),
            Some("The answer is 42.")
        );
    }

    #[test]
    fn concat_carries_the_text_into_a_tool_call_continuation() {
        let step = LlmStep::new_text(None, "Let me ".to_owned());
        let continuation = LlmStep::new_tool_calls(Some("search.".to_owned()), [tool_call()]);

        // The partial text stays in the content; the continuation's thought stays a thought.
        let expected = ToolCallMessage::from_parts(
            Some("search.".to_owned()),
            [ContentPart::text("Let me ")],
            vec![tool_call()],
        );
        assert_eq!(
            step.into_truncated().unwrap().concat(continuation),
            LlmStep::ToolCall(expected)
        );
    }
}
