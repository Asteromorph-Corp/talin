//! Shared helpers for rendering a conversation into a single prompt string.
//!
//! Completion-style endpoints (e.g. SGLang's `/generate`) take a single text input rather than a
//! structured list of messages, so the conversation must be flattened into one string.

use itertools::Itertools;

use crate::schemas::{
    AssistantMessage, BodyMessage, ContentExt, ToolCallMessage, ToolMessage, UserMessage,
};

const IM_START: &str = "<|im_start|>";
const IM_END: &str = "<|im_end|>";

/// Renders a conversation into a single [ChatML] prompt.
///
/// The non-empty `system` prompts are joined into a leading `system` turn, each message becomes a
/// `user`/`assistant` turn, and the prompt is left open on an `assistant` turn so the model
/// continues from there.
///
/// [ChatML]: https://github.com/openai/openai-python/blob/120d225b91a8453e15240a49fb1c6794d8119326/chatml.md
pub(crate) fn render_chatml(system: &[String], messages: &[BodyMessage]) -> String {
    let mut out = String::new();

    let system = system.iter().filter(|s| !s.is_empty()).join("\n\n");
    if !system.is_empty() {
        write_turn(&mut out, "system", &system);
    }

    for message in messages {
        match message {
            BodyMessage::User(m) => write_turn(&mut out, "user", &render_user(m)),
            BodyMessage::Assistant(m) => write_turn(&mut out, "assistant", &render_assistant(m)),
            BodyMessage::ToolCall(m) => write_turn(&mut out, "assistant", &render_tool_call(m)),
            BodyMessage::Tool(m) => write_turn(&mut out, "user", &render_tool_response(m)),
        }
    }

    // Leave the prompt open on an assistant turn for the model to continue.
    out.push_str(IM_START);
    out.push_str("assistant\n");
    out
}

/// Writes a single ChatML turn: `<|im_start|>{role}\n{content}<|im_end|>\n`.
fn write_turn(out: &mut String, role: &str, content: &str) {
    out.push_str(IM_START);
    out.push_str(role);
    out.push('\n');
    out.push_str(content);
    out.push('\n');
    out.push_str(IM_END);
    out.push('\n');
}

fn render_user(message: &UserMessage) -> String {
    message.content.to_text().unwrap_or_default()
}

fn render_assistant(message: &AssistantMessage) -> String {
    prepend_thought(
        message.thought.as_deref(),
        message.content.to_text().unwrap_or_default(),
    )
}

fn render_tool_call(message: &ToolCallMessage) -> String {
    let calls = message
        .tool_calls
        .iter()
        .map(|call| format!("<tool_call>\n{call}\n</tool_call>"))
        .join("\n");
    prepend_thought(message.thought.as_deref(), calls)
}

fn render_tool_response(message: &ToolMessage) -> String {
    format!("<tool_response>\n{}\n</tool_response>", message.output)
}

/// Prefixes `body` with a `<think>` block when a non-empty thought is present.
fn prepend_thought(thought: Option<&str>, body: String) -> String {
    match thought.filter(|t| !t.is_empty()) {
        Some(thought) => format!("<think>\n{thought}\n</think>\n\n{body}"),
        None => body,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use serde_json::json;

    use super::*;
    use crate::schemas::{ToolCall, ToolOutput};

    fn user(content: &str) -> BodyMessage {
        BodyMessage::User(UserMessage::new(content))
    }

    fn assistant(thought: Option<&str>, content: &str) -> BodyMessage {
        BodyMessage::Assistant(AssistantMessage::new(thought.map(String::from), content))
    }

    fn tool_call(thought: Option<&str>, calls: Vec<ToolCall>) -> BodyMessage {
        BodyMessage::ToolCall(ToolCallMessage::new(thought.map(String::from), calls))
    }

    fn tool_response(output: impl Into<ToolOutput>) -> BodyMessage {
        BodyMessage::Tool(ToolMessage::new("call_1", "search", output.into()))
    }

    /// With nothing to render, the prompt is just an open assistant turn.
    #[test]
    fn empty() {
        assert_eq!(render_chatml(&[], &[]), "<|im_start|>assistant\n");
    }

    /// Non-empty system prompts are joined into a single leading `system` turn.
    #[test]
    fn joins_system_prompts_and_skips_empty() {
        let out = render_chatml(&["".into(), "Be terse.".into(), "Be kind.".into()], &[]);
        assert_eq!(
            out,
            indoc! {"
                <|im_start|>system
                Be terse.

                Be kind.
                <|im_end|>
                <|im_start|>assistant
            "}
        );
    }

    /// An all-empty system slice produces no `system` turn.
    #[test]
    fn omits_empty_system() {
        assert_eq!(render_chatml(&["".into()], &[]), "<|im_start|>assistant\n");
    }

    /// User and assistant messages become `user`/`assistant` turns.
    #[test]
    fn renders_user_and_assistant_turns() {
        let out = render_chatml(&[], &[user("Hi"), assistant(None, "Hello!")]);
        assert_eq!(
            out,
            indoc! {"
                <|im_start|>user
                Hi
                <|im_end|>
                <|im_start|>assistant
                Hello!
                <|im_end|>
                <|im_start|>assistant
            "}
        );
    }

    /// A non-empty thought is rendered as a leading `<think>` block.
    #[test]
    fn renders_assistant_thought() {
        let out = render_chatml(&[], &[assistant(Some("reasoning"), "answer")]);
        assert_eq!(
            out,
            indoc! {"
                <|im_start|>assistant
                <think>
                reasoning
                </think>

                answer
                <|im_end|>
                <|im_start|>assistant
            "}
        );
    }

    /// An empty thought is treated as absent, so no `<think>` block is emitted.
    #[test]
    fn empty_thought_is_omitted() {
        let out = render_chatml(&[], &[assistant(Some(""), "answer")]);
        assert_eq!(
            out,
            "<|im_start|>assistant\nanswer\n<|im_end|>\n<|im_start|>assistant\n"
        );
    }

    /// Tool calls become an assistant turn wrapping each call in `<tool_call>` tags.
    #[test]
    fn renders_tool_calls() {
        let a = ToolCall::new(Some("1".into()), "search", Some(json!({"q": "rust"})));
        let b = ToolCall::new(Some("2".into()), "clock", None);
        let out = render_chatml(&[], &[tool_call(None, vec![a.clone(), b.clone()])]);
        let expected = format!(
            "<|im_start|>assistant\n<tool_call>\n{a}\n</tool_call>\n<tool_call>\n{b}\n</tool_call>\n<|im_end|>\n<|im_start|>assistant\n"
        );
        assert_eq!(out, expected);
    }

    /// A thought preceding tool calls is rendered as a `<think>` block above them.
    #[test]
    fn renders_thought_before_tool_calls() {
        let call = ToolCall::new(Some("1".into()), "search", Some(json!({"q": "rust"})));
        let out = render_chatml(&[], &[tool_call(Some("let me look"), vec![call.clone()])]);
        let expected = format!(
            "<|im_start|>assistant\n<think>\nlet me look\n</think>\n\n<tool_call>\n{call}\n</tool_call>\n<|im_end|>\n<|im_start|>assistant\n"
        );
        assert_eq!(out, expected);
    }

    /// Tool results become a `user` turn wrapped in `<tool_response>` tags.
    #[test]
    fn renders_tool_response() {
        let out = render_chatml(&[], &[tool_response("42".to_string())]);
        assert_eq!(
            out,
            indoc! {"
                <|im_start|>user
                <tool_response>
                42
                </tool_response>
                <|im_end|>
                <|im_start|>assistant
            "}
        );
    }

    /// A list-valued tool result is flattened by [`ToolData`]'s own formatting.
    #[test]
    fn renders_list_tool_response() {
        let output: ToolOutput = vec!["a".to_string(), "b".to_string()].into();
        let data = output.to_string();
        let out = render_chatml(&[], &[tool_response(output)]);
        let expected = format!(
            "<|im_start|>user\n<tool_response>\n{data}\n</tool_response>\n<|im_end|>\n<|im_start|>assistant\n"
        );
        assert_eq!(out, expected);
    }
}
