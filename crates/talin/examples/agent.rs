//! A tool-using agent.
//!
//! Run with `cargo run --example agent -- --provider gemini-interactions`.

use std::error::Error;

use async_trait::async_trait;
use clap::Parser;
use serde_json::Value;
use talin::agent::{Agent, AgentError};
use talin::llm::{
    ClaudeMessages, GeminiGenerateContent, GeminiInteractions, GenericChat, OpenAIChat,
    OpenAIResponses, XaiResponses,
};
use talin::tools::{EmptyFunctionInput, Function, Tool};

use crate::common::provider::Provider;

mod common;

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum)]
    provider: Option<Provider>,
}

const PROMPT: &str = "What is today's date? Run the tool to confirm.";

#[derive(Default)]
struct Date;
#[async_trait]
impl Function for Date {
    type Input = EmptyFunctionInput;
    type Output = String;

    fn name(&self) -> String {
        "Date".to_string()
    }

    fn description(&self) -> String {
        "Useful when you need to get the date, input should be an empty object ({})".to_string()
    }

    async fn parse_input(&self, _input: Value) -> Result<Self::Input, serde_json::Error> {
        Ok(EmptyFunctionInput {})
    }

    async fn call(&self, _input: Self::Input) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("25 of november of 2025".to_string())
    }
}

async fn run_agent_generic_chat() -> Result<String, AgentError> {
    let date = Tool::from(Date);
    let llm = GenericChat::builder().with_model("gpt-4o-mini").build();
    let agent: Agent<_> = Agent::builder().tools([date]).build(llm);
    agent.run(PROMPT).await
}

async fn run_agent_openai_chat() -> Result<String, AgentError> {
    let date = Tool::from(Date);
    let agent: Agent<_> = Agent::builder().tools([date]).build(OpenAIChat::default());
    agent.run(PROMPT).await
}

async fn run_agent_claude_messages() -> Result<String, AgentError> {
    let date = Tool::from(Date);
    let agent: Agent<_> = Agent::builder()
        .tools([date])
        .build(ClaudeMessages::default());
    agent.run(PROMPT).await
}

async fn run_agent_gemini_interactions() -> Result<String, AgentError> {
    let date = Tool::from(Date);
    let agent: Agent<_> = Agent::builder()
        .tools([date])
        .build(GeminiInteractions::default());
    agent.run(PROMPT).await
}

async fn run_agent_gemini_generate_content() -> Result<String, AgentError> {
    let date = Tool::from(Date);
    let agent: Agent<_> = Agent::builder()
        .tools([date])
        .build(GeminiGenerateContent::default());
    agent.run(PROMPT).await
}

async fn run_agent_openai_responses() -> Result<String, AgentError> {
    let date = Tool::from(Date);
    let agent: Agent<_> = Agent::builder()
        .tools([date])
        .build(OpenAIResponses::default());
    agent.run(PROMPT).await
}

async fn run_agent_xai_responses() -> Result<String, AgentError> {
    let date = Tool::from(Date);
    let agent: Agent<_> = Agent::builder()
        .tools([date])
        .build(XaiResponses::default());
    agent.run(PROMPT).await
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let args = Args::parse();
    let provider = args.provider.unwrap_or(Provider::OpenaiResponses);
    let result = match provider {
        Provider::GenericChat => run_agent_generic_chat().await,
        Provider::OpenaiChat => run_agent_openai_chat().await,
        Provider::ClaudeMessages => run_agent_claude_messages().await,
        Provider::GeminiInteractions => run_agent_gemini_interactions().await,
        Provider::GeminiGenerateContent => run_agent_gemini_generate_content().await,
        Provider::OpenaiResponses => run_agent_openai_responses().await,
        Provider::XaiResponses => run_agent_xai_responses().await,
    };

    match result {
        Ok(output) => println!("Model response:\n{output}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
