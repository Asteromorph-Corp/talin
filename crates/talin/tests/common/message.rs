//! Message JSONs.

// --- Per-provider response shapes -------------------------------------------------------------
// Each returns a spec-accurate body whose only assistant text is `text`.

use serde_json::{Value, json};

/// OpenAI Chat Completions API (`POST /chat/completions`).
pub fn openai_chat_response(text: &str) -> Value {
    json!({
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "created": 0,
        "model": "gpt-test",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": text },
            "finish_reason": "stop"
        }],
        "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 }
    })
}

/// OpenAI Responses API (`POST /v1/responses`), via `async-openai`'s `Response` type.
pub fn openai_responses_response(text: &str) -> Value {
    json!({
        "id": "resp_test",
        "object": "response",
        "created_at": 0,
        "status": "completed",
        "model": "gpt-test",
        "output": [{
            "type": "message",
            "id": "msg_test",
            "role": "assistant",
            "status": "completed",
            "content": [{ "type": "output_text", "annotations": [], "text": text }]
        }]
    })
}

/// xAI Responses API (`POST /v1/responses`).
pub fn xai_responses_response(text: &str) -> Value {
    json!({
        "id": "resp_test",
        "object": "response",
        "created_at": 0,
        "status": "completed",
        "model": "grok-test",
        "background": false,
        "frequency_penalty": 0.0,
        "presence_penalty": 0.0,
        "metadata": {},
        "service_tier": "default",
        "tool_choice": "auto",
        "top_logprobs": 0,
        "truncation": "disabled",
        "parallel_tool_calls": false,
        "store": false,
        "text": {},
        "tools": [],
        "output": [{
            "type": "message",
            "id": "msg_test",
            "status": "completed",
            "role": "assistant",
            "content": [{ "type": "output_text", "text": text, "annotations": [] }]
        }],
        "usage": {
            "input_tokens": 1,
            "input_tokens_details": { "cached_tokens": 0 },
            "num_server_side_tools_used": 0,
            "num_sources_used": 0,
            "output_tokens": 1,
            "output_tokens_details": { "reasoning_tokens": 0 },
            "total_tokens": 2
        }
    })
}

/// Anthropic Messages API (`POST /v1/messages`).
pub fn claude_messages_response(text: &str) -> Value {
    json!({
        "id": "msg_test",
        "type": "message",
        "role": "assistant",
        "model": "claude-test",
        "content": [{ "type": "text", "text": text }],
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": { "input_tokens": 1, "output_tokens": 1 }
    })
}

/// Gemini Generate Content API (`POST …/models/{model}:generateContent`).
pub fn gemini_generate_content_response(text: &str) -> Value {
    json!({
        "candidates": [{
            "content": { "parts": [{ "text": text }], "role": "model" },
            "finishReason": "STOP",
            "index": 0
        }],
        "usageMetadata": {
            "promptTokenCount": 1,
            "candidatesTokenCount": 1,
            "totalTokenCount": 2
        }
    })
}

/// Gemini Interactions API (`POST …/interactions`) — an interaction of model-output steps.
pub fn gemini_interactions_response(text: &str) -> Value {
    json!({
        "id": "interaction-test",
        "created": "2026-01-01T00:00:00Z",
        "updated": "2026-01-01T00:00:00Z",
        "status": "completed",
        "steps": [{
            "type": "model_output",
            "content": [{ "type": "text", "text": text }]
        }]
    })
}

/// SGLang native generate endpoint (`POST /generate`).
pub fn sglang_generate_response(text: &str) -> Value {
    json!({
        "text": text,
        "meta_info": {
            "id": "test",
            "finish_reason": { "type": "stop" },
            "prompt_tokens": 1,
            "num_retractions": 0,
            "completion_tokens": 1,
            "cached_tokens": 0
        }
    })
}

// --- Agent axis: mock tool + tool-call response shapes ----------------------------------------
// The agent loop is: turn 1 the model calls the given tool, the runner executes it, and turn 2's
// *request* must carry that call + its result — which is what `*_tool_result_request` matches.

pub fn generic_chat_tool_result_request(_id: &str, _name: &str) -> Value {
    json!({
        "messages": [
            { "role": "system" },
            { "role": "user" },
            { "role": "assistant" },
            { "role": "user" },
        ]
    })
}

pub fn openai_chat_tool_result_request(id: &str, name: &str) -> Value {
    json!({
        "messages": [
            { "role": "user" },
            { "role": "assistant", "tool_calls": [
                { "id": id, "function": { "name": name } },
            ]},
            { "role": "tool", "tool_call_id": id },
        ]
    })
}

pub fn openai_responses_tool_result_request(id: &str, name: &str) -> Value {
    json!({
        "input": [
            { "type": "message" },
            { "type": "function_call", "call_id": id, "name": name },
            { "type": "function_call_output", "call_id": id },
        ]
    })
}

pub fn xai_responses_tool_result_request(id: &str, name: &str) -> Value {
    json!({
        "input": [
            { "role": "user" },
            { "type": "function_call", "call_id": id, "name": name },
            { "type": "function_call_output", "call_id": id },
        ]
    })
}

pub fn claude_messages_tool_result_request(id: &str, name: &str) -> Value {
    json!({
        "messages": [
            { "role": "user" },
            {
                "role": "assistant",
                "content": [{ "type": "tool_use", "id": id, "name": name }]
            },
            {
                "role": "user",
                "content": [{ "type": "tool_result", "tool_use_id": id }]
            },
        ]
    })
}

pub fn gemini_generate_content_tool_result_request(_id: &str, name: &str) -> Value {
    json!({
        "contents": [
            { "role": "user" },
            { "role": "model", "parts": [{ "functionCall": { "name": name } }] },
            { "role": "user", "parts": [{ "functionResponse": { "name": name } }] },
        ]
    })
}

pub fn gemini_interactions_tool_result_request(id: &str, name: &str) -> Value {
    json!({
        "input": [
            { "type": "user_input" },
            { "type": "function_call", "id": id, "name": name },
            { "type": "function_result", "call_id": id },
        ]
    })
}

pub fn sglang_generate_tool_result_request(_id: &str, name: &str) -> mockito::Matcher {
    mockito::Matcher::Regex(name.to_string())
}

/// [`GenericChat`] turn-1 response. Unlike [`openai_chat`], [`GenericChat`] emulates tools in
/// **text** via `DefaultInstructor` — the tool call is a JSON object (`{name, arguments}`) in the
/// assistant `content`, not a native `tool_calls` array.
pub fn generic_chat_tool_call_response(id: &str, name: &str, arguments: &Value) -> Value {
    let call = json!({ "id": id, "name": name, "arguments": arguments }).to_string();
    json!({
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "created": 0,
        "model": "generic-test",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": call },
            "finish_reason": "stop"
        }],
        "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 }
    })
}

/// OpenAI Chat Completions turn-1 response: a `tool_calls` entry for `name`.
/// (`arguments` is a JSON *string*, per the spec.)
pub fn openai_chat_tool_call_response(id: &str, name: &str, arguments: &Value) -> Value {
    json!({
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "created": 0,
        "model": "gpt-test",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": arguments.to_string()
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 }
    })
}

/// OpenAI Responses turn-1 response: a `function_call` output item for `name`.
pub fn openai_responses_tool_call_response(id: &str, name: &str, arguments: &Value) -> Value {
    json!({
        "id": "resp_test",
        "object": "response",
        "created_at": 0,
        "status": "completed",
        "model": "gpt-test",
        "output": [{
            "type": "function_call",
            "id": "fc_test",
            "call_id": id,
            "name": name,
            "arguments": arguments.to_string(),
            "status": "completed"
        }]
    })
}

/// xAI Responses turn-1 response: a `function_call` output item for `name`.
pub fn xai_responses_tool_call_response(id: &str, name: &str, arguments: &Value) -> Value {
    json!({
        "id": "resp_test",
        "object": "response",
        "created_at": 0,
        "status": "completed",
        "model": "grok-test",
        "background": false,
        "frequency_penalty": 0.0,
        "presence_penalty": 0.0,
        "metadata": {},
        "service_tier": "default",
        "tool_choice": "auto",
        "top_logprobs": 0,
        "truncation": "disabled",
        "parallel_tool_calls": false,
        "store": false,
        "text": {},
        "tools": [],
        "output": [{
            "type": "function_call",
            "call_id": id,
            "id": "fc_test",
            "name": name,
            "arguments": arguments.to_string()
        }],
        "usage": {
            "input_tokens": 1,
            "input_tokens_details": { "cached_tokens": 0 },
            "num_server_side_tools_used": 0,
            "num_sources_used": 0,
            "output_tokens": 1,
            "output_tokens_details": { "reasoning_tokens": 0 },
            "total_tokens": 2
        }
    })
}

/// Anthropic Messages turn-1 response: a `tool_use` block for `name`.
pub fn claude_messages_tool_call_response(id: &str, name: &str, arguments: &Value) -> Value {
    json!({
        "id": "msg_test",
        "type": "message",
        "role": "assistant",
        "model": "claude-test",
        "content": [{
            "type": "tool_use",
            "id": id,
            "name": name,
            "input": arguments,
            "caller": { "type": "direct" }
        }],
        "stop_reason": "tool_use",
        "usage": { "input_tokens": 1, "output_tokens": 1 }
    })
}

/// Gemini generate-content turn-1 response: a `functionCall` part for `name`
/// (`args` is a JSON object here, not a string).
pub fn gemini_generate_content_tool_call_response(
    _id: &str,
    name: &str,
    arguments: &Value,
) -> Value {
    json!({
        "candidates": [{
            "content": {
                "parts": [{ "functionCall": { "name": name, "args": arguments } }],
                "role": "model"
            },
            "finishReason": "STOP",
            "index": 0
        }],
        "usageMetadata": {
            "promptTokenCount": 1,
            "candidatesTokenCount": 1,
            "totalTokenCount": 2
        }
    })
}

/// Gemini Interactions turn-1 response: a `function_call` step for `name`.
pub fn gemini_interactions_tool_call_response(id: &str, name: &str, arguments: &Value) -> Value {
    json!({
        "id": "interaction-test",
        "created": "2026-01-01T00:00:00Z",
        "updated": "2026-01-01T00:00:00Z",
        "status": "completed",
        "steps": [{
            "type": "function_call",
            "id": id,
            "name": name,
            "arguments": arguments
        }]
    })
}

/// SGLang generate turn-1 response: the tool call as text (like [`generic_chat`],
/// `DefaultInstructor` parses a `{name, arguments}` JSON out of the completion), wrapped in the
/// native `/generate` body.
pub fn sglang_generate_tool_call_response(id: &str, name: &str, arguments: &Value) -> Value {
    let call = json!({ "id": id, "name": name, "arguments": arguments }).to_string();
    json!({
        "text": call,
        "meta_info": {
            "id": "test",
            "finish_reason": { "type": "stop" },
            "prompt_tokens": 1,
            "num_retractions": 0,
            "completion_tokens": 1,
            "cached_tokens": 0
        }
    })
}

// --- Truncated response shapes ----------------------------------------------------------------
// Each patches the provider's own "cut off by the output token limit" marker into an otherwise
// normal response, so `LlmResponse::truncation` reports it and the runner asks for a continuation.

/// OpenAI Chat Completions, and Generic chat.
pub fn openai_chat_truncated_response(text: &str) -> Value {
    let mut response = openai_chat_response(text);
    response["choices"][0]["finish_reason"] = json!("length");
    response
}

/// OpenAI Responses.
pub fn openai_responses_truncated_response(text: &str) -> Value {
    let mut response = openai_responses_response(text);
    response["status"] = json!("incomplete");
    response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
    response["output"][0]["status"] = json!("incomplete");
    response
}

/// xAI Responses.
pub fn xai_responses_truncated_response(text: &str) -> Value {
    let mut response = xai_responses_response(text);
    response["status"] = json!("incomplete");
    response["incomplete_details"] = json!({ "reason": "max_output_tokens" });
    response["output"][0]["status"] = json!("incomplete");
    response
}

/// Anthropic Messages.
pub fn claude_messages_truncated_response(text: &str) -> Value {
    let mut response = claude_messages_response(text);
    response["stop_reason"] = json!("max_tokens");
    response
}

/// Gemini Generate Content.
pub fn gemini_generate_content_truncated_response(text: &str) -> Value {
    let mut response = gemini_generate_content_response(text);
    response["candidates"][0]["finishReason"] = json!("MAX_TOKENS");
    response
}

/// Gemini Interactions.
pub fn gemini_interactions_truncated_response(text: &str) -> Value {
    let mut response = gemini_interactions_response(text);
    response["status"] = json!("incomplete");
    response
}

/// SGLang native generate.
pub fn sglang_generate_truncated_response(text: &str) -> Value {
    let mut response = sglang_generate_response(text);
    response["meta_info"]["finish_reason"] = json!({ "type": "length", "length": 8 });
    response
}

// --- Reasoning: turn-1 responses that carry the model's reasoning next to the tool call --------
// Each wraps the provider's `*_tool_call_response` and adds `thought` in that provider's own
// reasoning shape, so the round trip can be asserted on `thought` appearing in turn 2's request.

/// A signature the provider would reject if it did not come back verbatim.
pub const THOUGHT_SIGNATURE: &str = "dGhvdWdodC1zaWduYXR1cmU=";

/// OpenAI Chat Completions: reasoning is emulated in text, closed by a `</think>` tag.
pub fn openai_chat_thought_tool_call_response(
    id: &str,
    name: &str,
    arguments: &Value,
    thought: &str,
) -> Value {
    let mut response = openai_chat_tool_call_response(id, name, arguments);
    response["choices"][0]["message"]["content"] = json!(format!("{thought}\n</think>"));
    response
}

/// Generic chat: the tool call is text too, so the reasoning simply precedes it.
pub fn generic_chat_thought_tool_call_response(
    id: &str,
    name: &str,
    arguments: &Value,
    thought: &str,
) -> Value {
    let mut response = generic_chat_tool_call_response(id, name, arguments);
    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .expect("generic chat puts the call in `content`")
        .to_string();
    response["choices"][0]["message"]["content"] = json!(format!("{thought}\n</think>\n{content}"));
    response
}

/// SGLang native generate: text, like generic chat.
pub fn sglang_generate_thought_tool_call_response(
    id: &str,
    name: &str,
    arguments: &Value,
    thought: &str,
) -> Value {
    let mut response = sglang_generate_tool_call_response(id, name, arguments);
    let text = response["text"]
        .as_str()
        .expect("SGLang puts the call in `text`")
        .to_string();
    response["text"] = json!(format!("{thought}\n</think>\n{text}"));
    response
}

/// OpenAI Responses: a `reasoning` item before the function call.
pub fn openai_responses_thought_tool_call_response(
    id: &str,
    name: &str,
    arguments: &Value,
    thought: &str,
) -> Value {
    let mut response = openai_responses_tool_call_response(id, name, arguments);
    let reasoning = json!({
        "type": "reasoning",
        "id": "rs_test",
        "summary": [{ "type": "summary_text", "text": thought }],
        "encrypted_content": THOUGHT_SIGNATURE,
    });
    prepend_output(&mut response["output"], reasoning);
    response
}

/// xAI Responses: a `reasoning` item before the function call.
pub fn xai_responses_thought_tool_call_response(
    id: &str,
    name: &str,
    arguments: &Value,
    thought: &str,
) -> Value {
    let mut response = xai_responses_tool_call_response(id, name, arguments);
    let reasoning = json!({
        "type": "reasoning",
        "id": "rs_test",
        "content": [{ "type": "reasoning_text", "text": thought }],
        "summary": [],
        "encrypted_content": THOUGHT_SIGNATURE,
    });
    prepend_output(&mut response["output"], reasoning);
    response
}

/// Anthropic Messages: a signed `thinking` block before the `tool_use` block.
pub fn claude_messages_thought_tool_call_response(
    id: &str,
    name: &str,
    arguments: &Value,
    thought: &str,
) -> Value {
    let mut response = claude_messages_tool_call_response(id, name, arguments);
    let thinking = json!({
        "type": "thinking",
        "thinking": thought,
        "signature": THOUGHT_SIGNATURE,
    });
    prepend_output(&mut response["content"], thinking);
    response
}

/// Gemini Generate Content: a signed part flagged `thought`.
pub fn gemini_generate_content_thought_tool_call_response(
    id: &str,
    name: &str,
    arguments: &Value,
    thought: &str,
) -> Value {
    let mut response = gemini_generate_content_tool_call_response(id, name, arguments);
    let part = json!({
        "text": thought,
        "thought": true,
        "thoughtSignature": THOUGHT_SIGNATURE,
    });
    prepend_output(&mut response["candidates"][0]["content"]["parts"], part);
    response
}

/// Gemini Interactions: a signed `thought` step before the `function_call` step.
pub fn gemini_interactions_thought_tool_call_response(
    id: &str,
    name: &str,
    arguments: &Value,
    thought: &str,
) -> Value {
    let mut response = gemini_interactions_tool_call_response(id, name, arguments);
    let step = json!({
        "type": "thought",
        "signature": THOUGHT_SIGNATURE,
        "summary": [{ "type": "text", "text": thought }],
    });
    prepend_output(&mut response["steps"], step);
    response
}

/// Puts `item` at the front of an array in a response body.
fn prepend_output(array: &mut Value, item: Value) {
    let items = array.as_array_mut().expect("response holds an array");
    items.insert(0, item);
}

// --- Structured output: where each provider carries the JSON schema ----------------------------
// Matched against the request body; `property` is one property of the schema, whose value is a
// string.

pub fn openai_chat_json_schema_request(name: &str, property: &str) -> Value {
    json!({
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": name,
                "strict": true,
                "schema": { "properties": { property: { "type": "string" } } }
            }
        }
    })
}

pub fn openai_responses_json_schema_request(name: &str, property: &str) -> Value {
    json!({
        "text": {
            "format": {
                "type": "json_schema",
                "name": name,
                "strict": true,
                "schema": { "properties": { property: { "type": "string" } } }
            }
        }
    })
}

pub fn xai_responses_json_schema_request(name: &str, property: &str) -> Value {
    json!({
        "text": {
            "format": {
                "type": "json_schema",
                "name": name,
                "strict": true,
                "schema": { "properties": { property: { "type": "string" } } }
            }
        }
    })
}

/// Anthropic takes the bare schema — no format name, no strict flag.
pub fn claude_messages_json_schema_request(_name: &str, property: &str) -> Value {
    json!({
        "output_config": {
            "format": {
                "type": "json_schema",
                "schema": { "properties": { property: { "type": "string" } } }
            }
        }
    })
}

/// The schema only applies alongside the JSON mime type.
pub fn gemini_generate_content_json_schema_request(_name: &str, property: &str) -> Value {
    json!({
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseJsonSchema": { "properties": { property: { "type": "string" } } }
        }
    })
}

pub fn gemini_interactions_json_schema_request(_name: &str, property: &str) -> Value {
    json!({
        "response_format": {
            "mime_type": "application/json",
            "schema": { "properties": { property: { "type": "string" } } }
        }
    })
}

/// [`GenericChat`] has no schema field: the schema is rendered into the system prompt.
pub fn generic_chat_json_schema_request(_name: &str, property: &str) -> mockito::Matcher {
    mockito::Matcher::Regex(format!(r#"{property}\\": string"#))
}

/// sglang carries the schema as a JSON *string* in `sampling_params`.
pub fn sglang_generate_json_schema_request(_name: &str, property: &str) -> mockito::Matcher {
    mockito::Matcher::Regex(format!(r#"json_schema.*{property}"#))
}

// --- Multimodal input: how each provider encodes attached media -------------------------------
// Matched against the request body. `data` is the media's base64, `mime` its MIME type.

pub fn openai_chat_image_request(data: &str, mime: &str) -> Value {
    json!({
        "messages": [
            { "role": "system" },
            { "role": "user", "content": [
                { "type": "text" },
                { "type": "image_url", "image_url": { "url": data_url(data, mime) } },
            ]},
        ]
    })
}

pub fn openai_responses_image_request(data: &str, mime: &str) -> Value {
    json!({
        "input": [
            { "type": "message" },
            { "type": "message", "content": [
                { "type": "input_text" },
                { "type": "input_image", "image_url": data_url(data, mime) },
            ]},
        ]
    })
}

pub fn xai_responses_image_request(data: &str, mime: &str) -> Value {
    json!({
        "input": [
            { "type": "message" },
            { "type": "message", "content": [
                { "type": "input_text" },
                { "type": "input_image", "image_url": data_url(data, mime) },
            ]},
        ]
    })
}

pub fn claude_messages_image_request(data: &str, mime: &str) -> Value {
    json!({
        "messages": [{ "role": "user", "content": [
            { "type": "text" },
            { "type": "image", "source": {
                "type": "base64", "data": data, "media_type": mime,
            }},
        ]}]
    })
}

pub fn gemini_generate_content_image_request(data: &str, mime: &str) -> Value {
    json!({
        "contents": [{ "role": "user", "parts": [
            {},
            { "inlineData": { "mimeType": mime, "data": data } },
        ]}]
    })
}

pub fn gemini_interactions_image_request(data: &str, mime: &str) -> Value {
    json!({
        "input": [{ "type": "user_input", "content": [
            { "type": "text" },
            { "type": "image", "data": data, "mime_type": mime },
        ]}]
    })
}

/// OpenAI names the file rather than typing it: the MIME lives in the data URL.
pub fn openai_chat_document_request(data: &str, mime: &str, name: &str) -> Value {
    json!({
        "messages": [
            { "role": "system" },
            { "role": "user", "content": [
                { "type": "text" },
                { "type": "file", "file": {
                    "file_data": data_url(data, mime), "filename": name,
                }},
            ]},
        ]
    })
}

pub fn openai_responses_document_request(data: &str, mime: &str, name: &str) -> Value {
    json!({
        "input": [
            { "type": "message" },
            { "type": "message", "content": [
                { "type": "input_text" },
                { "type": "input_file", "file_data": data_url(data, mime), "filename": name },
            ]},
        ]
    })
}

/// xAI takes the bare base64 and the MIME type beside it, unlike OpenAI's data URL.
pub fn xai_responses_document_request(data: &str, mime: &str, name: &str) -> Value {
    json!({
        "input": [
            { "type": "message" },
            { "type": "message", "content": [
                { "type": "input_text" },
                {
                    "type": "input_file",
                    "file_data": data,
                    "filename": name,
                    "mime_type": mime,
                },
            ]},
        ]
    })
}

pub fn claude_messages_document_request(data: &str, mime: &str, name: &str) -> Value {
    json!({
        "messages": [{ "role": "user", "content": [
            { "type": "text" },
            { "type": "document", "title": name, "source": {
                "type": "base64", "data": data, "media_type": mime,
            }},
        ]}]
    })
}

/// Gemini encodes every medium the same way, and carries no file name.
pub fn gemini_generate_content_document_request(data: &str, mime: &str, _name: &str) -> Value {
    json!({
        "contents": [{ "role": "user", "parts": [
            {},
            { "inlineData": { "mimeType": mime, "data": data } },
        ]}]
    })
}

pub fn gemini_interactions_document_request(data: &str, mime: &str, _name: &str) -> Value {
    json!({
        "input": [{ "type": "user_input", "content": [
            { "type": "text" },
            { "type": "document", "data": data, "mime_type": mime },
        ]}]
    })
}

/// The `data:` URL that the OpenAI shapes inline media into.
fn data_url(data: &str, mime: &str) -> String {
    format!("data:{mime};base64,{data}")
}
