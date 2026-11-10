mod agents;
mod tools;

use agents::{GuesserAgent, GuesserAgentExt, Models};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let model = std::env::args().nth(1).unwrap_or("gpt_4o_mini".into());

    log::info!("Using model: {model}");
    let preset = Models::any(&model).unwrap_or_else(|e| panic!("{e}"));
    let agent = GuesserAgent::any(preset).await;
    let result = agent.run_immed().await;

    println!("------------FINISH------------");

    match result {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("{e}"),
    }
}
