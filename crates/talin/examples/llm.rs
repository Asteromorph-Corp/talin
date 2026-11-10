//! A single-turn `Llm::invoke` call, selected with `--provider`.
//!
//! Run with `cargo run --example llm -- --provider gemini-interactions`.

use clap::Parser;
use talin::llm::{
    ClaudeMessages, GeminiGenerateContent, GeminiInteractions, GenericChat, LlmError, LlmExt,
    OpenAIChat, OpenAIResponses, XaiResponses,
};

use crate::common::provider::Provider;

mod common;

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum)]
    provider: Option<Provider>,
}

const PROMPT: &str = "Hello";

async fn run_llm_generic_chat() -> Result<String, LlmError> {
    let llm = GenericChat::builder().with_model("gpt-4o-mini").build();
    llm.invoke(PROMPT).await
}

async fn run_llm_openai_chat() -> Result<String, LlmError> {
    let llm = OpenAIChat::default();
    llm.invoke(PROMPT).await
}

async fn run_llm_claude_messages() -> Result<String, LlmError> {
    let llm = ClaudeMessages::default();
    llm.invoke(PROMPT).await
}

async fn run_llm_gemini_interactions() -> Result<String, LlmError> {
    let llm = GeminiInteractions::default();
    llm.invoke(PROMPT).await
}

async fn run_llm_gemini_generate_content() -> Result<String, LlmError> {
    let llm = GeminiGenerateContent::default();
    llm.invoke(PROMPT).await
}

async fn run_llm_openai_responses() -> Result<String, LlmError> {
    let llm = OpenAIResponses::default();
    llm.invoke(PROMPT).await
}

async fn run_llm_xai_responses() -> Result<String, LlmError> {
    let llm = XaiResponses::default();
    llm.invoke(PROMPT).await
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let args = Args::parse();
    let provider = args.provider.unwrap_or(Provider::OpenaiResponses);
    let result = match provider {
        Provider::GenericChat => run_llm_generic_chat().await,
        Provider::OpenaiChat => run_llm_openai_chat().await,
        Provider::ClaudeMessages => run_llm_claude_messages().await,
        Provider::GeminiInteractions => run_llm_gemini_interactions().await,
        Provider::GeminiGenerateContent => run_llm_gemini_generate_content().await,
        Provider::OpenaiResponses => run_llm_openai_responses().await,
        Provider::XaiResponses => run_llm_xai_responses().await,
    };

    match result {
        Ok(output) => println!("Model response:\n{output}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
