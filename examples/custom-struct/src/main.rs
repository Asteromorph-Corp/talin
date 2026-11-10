mod agents;
mod tools;

use agents::{SchedulerAgent, SchedulerAgentExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let agent = SchedulerAgent::default().await;
    let request =
        "Book the large room tomorrow at 14:00 for Ana, Ben and Chloe. We need two hours.";
    match agent.run_immed(request).await {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("{e}"),
    }
}
