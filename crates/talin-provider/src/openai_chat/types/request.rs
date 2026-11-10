use std::collections::HashMap;

use async_openai::types::chat::{
    ChatCompletionRequestMessage, ChatCompletionTools, CreateChatCompletionRequest,
    CreateChatCompletionRequestArgs,
};
use serde::Serialize;

/// Request payload sent to an OpenAPI-compatible API with extra parameters
#[derive(Serialize, Debug, Clone)]
pub struct ChatRequest {
    /// The inner request object.
    #[serde(flatten)]
    pub inner: CreateChatCompletionRequest,

    /// The parameter for the top-k sampling method. The model considers the top `top_k` tokens
    /// with the highest probability. Lower values will make the output more deterministic.
    ///
    /// Not part of the OpenAI chat completions API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Number between 0.0 and 2.0. Positive values discourage the model from repeating the same
    /// line verbatim.
    ///
    /// Not part of the OpenAI chat completions API, but used by some other providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,

    /// Extra parameters for custom api.
    #[serde(flatten)]
    pub extra_params: HashMap<String, serde_json::Value>,
}

impl ChatRequest {
    /// Constructs a new [`ChatRequest`].
    pub fn new(
        model: String,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Option<Vec<ChatCompletionTools>>,
    ) -> Self {
        let mut args = CreateChatCompletionRequestArgs::default();
        args.model(model).messages(messages);
        if let Some(tools) = tools {
            args.tools(tools);
        }
        let inner = args.build().expect("Required fields are present");

        ChatRequest {
            inner,
            top_k: None,
            repetition_penalty: None,
            extra_params: HashMap::new(),
        }
    }
}
