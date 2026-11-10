use async_openai::error::ApiError;
use async_openai::types::chat::{
    ChatChoice, ChatChoiceLogprobs, ChatChoiceStream, ChatCompletionMessageToolCall,
    ChatCompletionMessageToolCalls, ChatCompletionResponseMessage, ChatCompletionResponseStream,
    ChatCompletionStreamResponseDelta, CompletionTokensDetails, CompletionUsage,
    CreateChatCompletionResponse, CreateChatCompletionStreamResponse, FunctionCall,
    PromptTokensDetails, Role,
};
use tokio_stream::StreamExt;

use crate::error::{Error, StreamError};
use crate::stream::{Merge, PreferNew};

/// Constructs a full [`CreateChatCompletionResponse`] from a stream of
/// [`ChatCompletionResponseStream`].
pub(super) async fn assemble(
    mut stream: ChatCompletionResponseStream,
) -> Result<CreateChatCompletionResponse, Error<ApiError>> {
    let mut response = None;
    while let Some(chunk) = stream.next().await {
        response.merge(chunk?);
    }
    response.ok_or(Error::StreamError(StreamError::UnexpectedClose))
}

/// Starts a response from the first chunk, taking its metadata and leaving the choices and usage
/// to be merged in.
impl Merge<CreateChatCompletionStreamResponse> for Option<CreateChatCompletionResponse> {
    fn merge(&mut self, rhs: CreateChatCompletionStreamResponse) {
        // Use deprecated field `system_fingerprint` for construction.
        #[allow(deprecated)]
        let response = self.get_or_insert_with(|| CreateChatCompletionResponse {
            id: rhs.id.clone(),
            choices: Vec::new(),
            created: rhs.created,
            model: rhs.model.clone(),
            service_tier: rhs.service_tier.clone(),
            system_fingerprint: rhs.system_fingerprint.clone(),
            object: rhs.object.clone(),
            usage: None,
            metadata: None,
            moderation: None,
        });
        response.merge(rhs);
    }
}

impl Merge<CreateChatCompletionStreamResponse> for CreateChatCompletionResponse {
    fn merge(&mut self, rhs: CreateChatCompletionStreamResponse) {
        for choice_stream in rhs.choices {
            let position = match self
                .choices
                .iter()
                .position(|choice| choice.index == choice_stream.index)
            {
                Some(position) => position,
                None => {
                    self.choices.push(ChatChoice {
                        index: choice_stream.index,
                        // Use deprecated field `function_call` for construction.
                        #[allow(deprecated)]
                        message: ChatCompletionResponseMessage {
                            role: Role::Assistant,
                            content: None,
                            refusal: None,
                            tool_calls: None,
                            function_call: None,
                            audio: None,
                            annotations: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    });
                    self.choices.sort_by_key(|choice| choice.index);
                    self.choices
                        .iter()
                        .position(|choice| choice.index == choice_stream.index)
                        .expect("Choice was just inserted")
                }
            };
            self.choices[position].merge(choice_stream);
        }
        self.usage.merge(rhs.usage);
        self.moderation.merge(PreferNew(rhs.moderation));
    }
}

impl Merge<ChatChoiceStream> for ChatChoice {
    fn merge(&mut self, rhs: ChatChoiceStream) {
        self.message.merge(rhs.delta);
        self.logprobs.merge(rhs.logprobs);
        self.finish_reason.merge(PreferNew(rhs.finish_reason));
    }
}

impl Merge<ChatCompletionStreamResponseDelta> for ChatCompletionResponseMessage {
    fn merge(&mut self, rhs: ChatCompletionStreamResponseDelta) {
        if let Some(role) = rhs.role {
            self.role = role;
        }
        self.content.merge(rhs.content);
        self.refusal.merge(rhs.refusal);

        for chunk in rhs.tool_calls.into_iter().flatten() {
            let calls = self.tool_calls.get_or_insert_with(Vec::new);
            let idx = chunk.index as usize;
            if calls.len() <= idx {
                calls.resize_with(idx + 1, || {
                    ChatCompletionMessageToolCalls::Function(ChatCompletionMessageToolCall {
                        id: String::new(),
                        function: FunctionCall {
                            name: String::new(),
                            arguments: String::new(),
                        },
                    })
                });
            }

            let ChatCompletionMessageToolCalls::Function(call) = &mut calls[idx] else {
                continue;
            };
            call.id.merge(chunk.id.unwrap_or_default());
            if let Some(function) = chunk.function {
                call.function.name.merge(function.name.unwrap_or_default());
                call.function
                    .arguments
                    .merge(function.arguments.unwrap_or_default());
            }
        }

        // Handle deprecated field `function_call` for completeness.
        #[allow(deprecated)]
        if let Some(function_call) = rhs.function_call {
            let current = self.function_call.get_or_insert_with(|| FunctionCall {
                name: String::new(),
                arguments: String::new(),
            });
            current.name.merge(function_call.name.unwrap_or_default());
            current
                .arguments
                .merge(function_call.arguments.unwrap_or_default());
        }
    }
}

impl Merge<ChatChoiceLogprobs> for ChatChoiceLogprobs {
    fn merge(&mut self, rhs: ChatChoiceLogprobs) {
        self.content.merge(rhs.content);
        self.refusal.merge(rhs.refusal);
    }
}

/// Merges by replacing with the newer counts, as a chunk that reports usage reports it in full.
impl Merge<CompletionUsage> for CompletionUsage {
    fn merge(&mut self, rhs: CompletionUsage) {
        self.prompt_tokens.merge(rhs.prompt_tokens);
        self.completion_tokens.merge(rhs.completion_tokens);
        self.total_tokens.merge(rhs.total_tokens);
        self.prompt_tokens_details.merge(rhs.prompt_tokens_details);
        self.completion_tokens_details
            .merge(rhs.completion_tokens_details);
    }
}

impl Merge<PromptTokensDetails> for PromptTokensDetails {
    fn merge(&mut self, rhs: PromptTokensDetails) {
        self.audio_tokens.merge(rhs.audio_tokens);
        self.cached_tokens.merge(rhs.cached_tokens);
        self.text_tokens.merge(rhs.text_tokens);
        self.image_tokens.merge(rhs.image_tokens);
        self.cache_write_tokens.merge(rhs.cache_write_tokens);
    }
}

impl Merge<CompletionTokensDetails> for CompletionTokensDetails {
    fn merge(&mut self, rhs: CompletionTokensDetails) {
        self.accepted_prediction_tokens
            .merge(rhs.accepted_prediction_tokens);
        self.audio_tokens.merge(rhs.audio_tokens);
        self.text_tokens.merge(rhs.text_tokens);
        self.reasoning_tokens.merge(rhs.reasoning_tokens);
        self.rejected_prediction_tokens
            .merge(rhs.rejected_prediction_tokens);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn chunk(
        choices: serde_json::Value,
        usage: serde_json::Value,
    ) -> CreateChatCompletionStreamResponse {
        serde_json::from_value(json!({
            "id": "chatcmpl-1",
            "object": "chat.completion.chunk",
            "created": 1,
            "model": "m",
            "choices": choices,
            "usage": usage,
        }))
        .unwrap()
    }

    #[test]
    fn merges_stream_into_response() {
        let chunks = [
            chunk(
                json!([{"index": 0, "delta": {"role": "assistant", "content": "Hel"}}]),
                json!(null),
            ),
            chunk(
                json!([{"index": 0, "delta": {"content": "lo", "tool_calls": [
                    {"index": 0, "id": "call_1", "function": {"name": "f", "arguments": "{\"a\""}}]}}]),
                json!(null),
            ),
            chunk(
                json!([{"index": 0, "delta": {"tool_calls": [
                    {"index": 0, "function": {"arguments": ":1}"}}]}, "finish_reason": "tool_calls"}]),
                json!(null),
            ),
            chunk(
                json!([]),
                json!({"prompt_tokens": 3, "completion_tokens": 4, "total_tokens": 7,
                       "completion_tokens_details": {"reasoning_tokens": 2}}),
            ),
        ];

        let mut response = None;
        for chunk in chunks {
            response.merge(chunk);
        }
        let response = response.unwrap();

        assert_eq!(response.id, "chatcmpl-1");
        let [choice] = response.choices.as_slice() else {
            panic!("expected one choice");
        };
        assert_eq!(choice.message.content.as_deref(), Some("Hello"));
        let Some([ChatCompletionMessageToolCalls::Function(call)]) =
            choice.message.tool_calls.as_deref()
        else {
            panic!("expected one function call");
        };
        assert_eq!(call.id, "call_1");
        assert_eq!(call.function.name, "f");
        assert_eq!(call.function.arguments, "{\"a\":1}");
        assert_eq!(
            choice.finish_reason,
            Some(async_openai::types::chat::FinishReason::ToolCalls)
        );
        let usage = response.usage.unwrap();
        assert_eq!(usage.total_tokens, 7);
        assert_eq!(
            usage.completion_tokens_details.unwrap().reasoning_tokens,
            Some(2)
        );
    }
}
