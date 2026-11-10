//! A single-turn `LlmChain` with a system message and a user template.
//!
//! Run with `cargo run --example llm_chain`.

use talin::chain::{Chain, LlmChain};
use talin::llm::OpenAIResponses;
use talin::prompt_template;
use talin::schemas::Message;
use talin::template::{MessageTemplate, TemplateRole};

#[tokio::main]
async fn main() {
    let system_message = Message::new_system_message("You are a seasoned Rust developer");
    let user_message = MessageTemplate::fstring("{input}").role(TemplateRole::User);

    let prompt = prompt_template![system_message, user_message];
    let llm = OpenAIResponses::default();
    let chain: LlmChain<_> = LlmChain::new(prompt, llm);

    match chain.call("What is rust").await {
        Ok(output) => println!("Model response:\n{}", output.content),
        Err(e) => eprintln!("Error: {e:?}"),
    }
}
