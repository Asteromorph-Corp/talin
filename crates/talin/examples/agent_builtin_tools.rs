//! An agent using a provider's built-in tools, selected with `--provider`.
//!
//! Built-in tools use a different wiring than user-defined [`Tool`]s: the agent builder must be
//! narrowed to the concrete provider with `.specialize::<L>()` before `.builtin_tools(...)` can
//! accept that provider's tool type.
//!
//! Run with e.g. `cargo run --example agent_builtin_tools -- --provider gemini-interactions`.

use clap::Parser;
use talin::agent::{Agent, AgentError};
use talin::llm::claude_messages::MessagesBuiltinTool;
use talin::llm::claude_messages::types::WebSearchTool20250305;
use talin::llm::gemini_generate_content::GenerateContentBuiltinTool;
use talin::llm::gemini_generate_content::types::CodeExecution as GenerateContentCodeExecution;
use talin::llm::gemini_interactions::InteractionsBuiltinTool;
use talin::llm::gemini_interactions::types::CodeExecution as InteractionsCodeExecution;
use talin::llm::openai_responses::ResponsesBuiltinTool as OpenAiResponsesBuiltinTool;
use talin::llm::openai_responses::types::WebSearchTool as OpenAiWebSearchTool;
use talin::llm::xai_responses::ResponsesBuiltinTool as XaiResponsesBuiltinTool;
use talin::llm::xai_responses::types::WebSearchTool as XaiWebSearchTool;
use talin::llm::{
    ClaudeMessages, GeminiGenerateContent, GeminiInteractions, OpenAIResponses, XaiResponses,
};

use crate::common::provider::Provider;

mod common;

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum)]
    provider: Option<Provider>,
}

const PROMPT: &str = "What is the latest stable Rust release, and what date was it released?";

async fn run_agent_claude_messages() -> Result<String, AgentError> {
    let tool = MessagesBuiltinTool::WebSearch20250305(WebSearchTool20250305::default());
    let agent: Agent<_> = Agent::builder()
        .specialize::<ClaudeMessages>()
        .builtin_tools(vec![tool])
        .build(ClaudeMessages::default());
    agent.run(PROMPT).await
}

async fn run_agent_gemini_interactions() -> Result<String, AgentError> {
    let tool = InteractionsBuiltinTool::CodeExecution(InteractionsCodeExecution::default());
    let agent: Agent<_> = Agent::builder()
        .specialize::<GeminiInteractions>()
        .builtin_tools(vec![tool])
        .build(GeminiInteractions::default());
    agent.run(PROMPT).await
}

async fn run_agent_gemini_generate_content() -> Result<String, AgentError> {
    let tool = GenerateContentBuiltinTool::CodeExecution(GenerateContentCodeExecution::default());
    let agent: Agent<_> = Agent::builder()
        .specialize::<GeminiGenerateContent>()
        .builtin_tools(vec![tool])
        .build(GeminiGenerateContent::default());
    agent.run(PROMPT).await
}

async fn run_agent_openai_responses() -> Result<String, AgentError> {
    let tool = OpenAiResponsesBuiltinTool::WebSearch(OpenAiWebSearchTool::default());
    let agent: Agent<_> = Agent::builder()
        .specialize::<OpenAIResponses>()
        .builtin_tools(vec![tool])
        .build(OpenAIResponses::default());
    agent.run(PROMPT).await
}

async fn run_agent_xai_responses() -> Result<String, AgentError> {
    let tool = XaiResponsesBuiltinTool::WebSearch(XaiWebSearchTool::default());
    let agent: Agent<_> = Agent::builder()
        .specialize::<XaiResponses>()
        .builtin_tools(vec![tool])
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
        Provider::GenericChat | Provider::OpenaiChat => {
            eprintln!(
                "`{provider:?}` does not support built-in tools, so this example cannot be run with that provider."
            );
            return;
        }
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
