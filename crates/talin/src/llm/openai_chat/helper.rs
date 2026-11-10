//! Helper functions for the chat completions api.

use talin_provider::openai_chat::types::{
    ChatChoice, ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageAudio,
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartText, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestSystemMessageContentPart,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    ChatCompletionResponseMessage, ChatRequest, FinishReason, ResponseFormat as ApiResponseFormat,
};

use crate::llm::UnsupportedContent;
use crate::llm::openai_chat::ChatOptions;
use crate::schemas::ContentPart;
use crate::utils::NoneIfEmpty;

pub(super) fn collect_texts(
    content: Vec<ContentPart>,
    content_type: &str,
) -> Result<Vec<String>, UnsupportedContent> {
    content
        .into_iter()
        .map(|part| match part {
            ContentPart::Text(text) => Ok(text),
            _ => Err(UnsupportedContent::new(
                "OpenAI Chat Completions",
                format!("non-text {content_type}"),
            )),
        })
        .collect()
}

/// Applies the given [`ChatOptions`] to the request.
#[allow(deprecated)]
pub(super) fn apply_options(request: &mut ChatRequest, options: ChatOptions) {
    let inner = &mut request.inner;
    inner.stream = options.stream;
    inner.stream_options = options.stream_options;
    inner.tool_choice = options.tool_choice;
    inner.parallel_tool_calls = options.parallel_tool_calls;
    inner.n = options.n;
    inner.reasoning_effort = options.reasoning_effort;
    inner.temperature = options.temperature;
    inner.top_p = options.top_p;
    inner.frequency_penalty = options.frequency_penalty;
    inner.presence_penalty = options.presence_penalty;
    inner.max_tokens = options.max_tokens;
    inner.max_completion_tokens = options.max_completion_tokens;
    inner.stop = options.stop;
    inner.response_format = options.response_format.map(ApiResponseFormat::from);
    request.top_k = options.top_k;
    request.repetition_penalty = options.repetition_penalty;
    if let Some(extra_params) = options.extra_params {
        request.extra_params = extra_params;
    }
}

/// Selects the best [`ChatChoice`] from a list of choices based on finish reason and index.
pub(super) fn select_choice(choices: &[ChatChoice]) -> Option<ChatChoice> {
    let choice = choices.iter().min_by_key(|c| match c.finish_reason {
        // Stop is the most preferred finish reason.
        Some(FinishReason::Stop) => 0,
        // Tool calls is the next preferred finish reason, as long as the tool calls are
        // actually generated.
        Some(FinishReason::ToolCalls)
            if c.message
                .tool_calls
                .as_deref()
                .is_some_and(|calls| !calls.is_empty()) =>
        {
            1
        }
        Some(FinishReason::ContentFilter) | Some(FinishReason::Length) => 2,
        _ => 3,
    });
    choice.cloned()
}

/// Constructs a [`ChatCompletionRequestSystemMessage`] with the given contents.
pub(super) fn system_message(contents: Vec<String>) -> Option<ChatCompletionRequestMessage> {
    if contents.is_empty() {
        return None;
    }

    let content = if contents.len() == 1 {
        let text = contents.into_iter().next().expect("First element exists");
        ChatCompletionRequestSystemMessageContent::Text(text)
    } else {
        let items = contents
            .into_iter()
            .map(|text| {
                ChatCompletionRequestSystemMessageContentPart::Text(
                    ChatCompletionRequestMessageContentPartText {
                        text,
                        prompt_cache_breakpoint: None,
                    },
                )
            })
            .collect::<Vec<_>>();
        ChatCompletionRequestSystemMessageContent::Array(items)
    };

    let msg = ChatCompletionRequestSystemMessage {
        name: None,
        content,
    };
    Some(ChatCompletionRequestMessage::from(msg))
}

/// Builds a plain-text user message.
///
/// Used for the crate's own scaffolding prompts, which carry no media and so bypass the fallible
/// [`UserMessage`](crate::schemas::UserMessage) conversion.
pub(super) fn text_user_message(text: impl Into<String>) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::from(ChatCompletionRequestUserMessage {
        name: None,
        content: ChatCompletionRequestUserMessageContent::Text(text.into()),
    })
}

/// Echoes a model response back as the assistant message of the next request.
///
/// Mapping the wire types directly preserves what a round trip through the canonical [`LlmStep`]
/// cannot carry: the raw `<think>` delimiters exactly as the model wrote them, custom
/// (non-function) tool calls, an audio handle, and a refusal. `annotations` has no request-side
/// counterpart and is dropped.
///
/// [`LlmStep`]: crate::schemas::LlmStep
pub(super) fn response_to_request_message(
    message: ChatCompletionResponseMessage,
) -> ChatCompletionRequestMessage {
    // Use deprecated field `function_call` for construction.
    #[allow(deprecated)]
    let assistant = ChatCompletionRequestAssistantMessage {
        content: message
            .content
            .and_then(NoneIfEmpty::none_if_empty)
            .map(ChatCompletionRequestAssistantMessageContent::Text),
        refusal: message.refusal,
        name: None,
        audio: message
            .audio
            .map(|audio| ChatCompletionRequestAssistantMessageAudio { id: audio.id }),
        tool_calls: message.tool_calls.and_then(NoneIfEmpty::none_if_empty),
        function_call: message.function_call,
    };
    ChatCompletionRequestMessage::Assistant(assistant)
}
