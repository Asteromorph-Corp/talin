mod agents;
mod tools;

use agents::{AssistantAgent, AssistantAgentExt, Models};
use talin::strategy::UsageTrackStrategy;

/// Usage: `providers <name>`, where `<name>` is an entry of `configs/models.toml`, such as
/// `claude_messages`.
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let name = std::env::args().nth(1).unwrap_or("openai_responses".into());

    // `Models::any` looks a model up by the name it has in `models.toml`.
    let model = Models::any(&name).unwrap_or_else(|e| panic!("{e}"));
    let agent = AssistantAgent::any(model).await;

    let question = "What is the date today, and which day of the week is it?";
    match agent
        .run_with_strategy_immed(UsageTrackStrategy::default(), question)
        .await
    {
        Ok(output) => println!("{}\n\n{}", output.content, output.extra_content),
        Err(e) => eprintln!("{e}"),
    }
}
