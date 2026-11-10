//! Optional recorder endpoint override. Normal live runs use the generated Models factories.
use std::error::Error;

use talin::llm::claude_messages::MessagesOptions;
use talin::llm::openai_responses::ResponsesOptions;
use talin::llm::openai_responses::types::{Reasoning, ReasoningEffort};
use talin::llm::{ClaudeMessages, OpenAIResponses};
use serde::Deserialize;

#[derive(Deserialize)]
struct EnvKey {
    env: String,
}

#[derive(Deserialize)]
struct Model {
    model: String,
    api_key: EnvKey,
    max_tokens: Option<u32>,
    max_completion_tokens: Option<u32>,
    reasoning_effort: Option<ReasoningEffort>,
    #[serde(default)]
    anthropic_beta: Vec<String>,
}

#[derive(Deserialize)]
struct Models {
    openai: Model,
    claude: Model,
    claude_compact: Model,
    claude_custom: Model,
    openai_compact: Model,
}

pub fn openai(base: &str) -> Result<OpenAIResponses, Box<dyn Error>> {
    let models = toml::from_str::<Models>(include_str!("../configs/models.toml"))?;
    build_openai(models.openai, base)
}

pub fn openai_compact(base: &str) -> Result<OpenAIResponses, Box<dyn Error>> {
    let models = toml::from_str::<Models>(include_str!("../configs/models.toml"))?;
    build_openai(models.openai_compact, base)
}

fn build_openai(model: Model, base: &str) -> Result<OpenAIResponses, Box<dyn Error>> {
    let mut options = ResponsesOptions::default();
    if let Some(max) = model.max_completion_tokens {
        options = options.with_max_output_tokens(max);
    }
    if let Some(effort) = model.reasoning_effort {
        options = options.with_reasoning(Reasoning {
            effort: Some(effort),
            ..Default::default()
        });
    }
    let llm = OpenAIResponses::builder()
        .with_api_base(base)
        .with_api_key(std::env::var(&model.api_key.env)?)
        .with_model(&model.model)
        .with_default_options(options)
        .build();
    Ok(llm)
}

pub fn claude(base: &str, stream: bool) -> Result<ClaudeMessages, Box<dyn Error>> {
    let models = toml::from_str::<Models>(include_str!("../configs/models.toml"))?;
    build_claude(models.claude, base, stream)
}

pub fn claude_compact(base: &str, stream: bool) -> Result<ClaudeMessages, Box<dyn Error>> {
    let models = toml::from_str::<Models>(include_str!("../configs/models.toml"))?;
    build_claude(models.claude_compact, base, stream)
}

pub fn claude_custom(base: &str, stream: bool) -> Result<ClaudeMessages, Box<dyn Error>> {
    let models = toml::from_str::<Models>(include_str!("../configs/models.toml"))?;
    build_claude(models.claude_custom, base, stream)
}

fn build_claude(model: Model, base: &str, stream: bool) -> Result<ClaudeMessages, Box<dyn Error>> {
    let mut options = MessagesOptions::default().with_stream(stream);
    if let Some(max) = model.max_tokens {
        options = options.with_max_tokens(max);
    }
    let llm = ClaudeMessages::builder()
        .with_api_base(base)
        .with_api_key(std::env::var(&model.api_key.env)?)
        .with_model(&model.model)
        .with_anthropic_betas(model.anthropic_beta)
        .with_default_options(options)
        .build();
    Ok(llm)
}
