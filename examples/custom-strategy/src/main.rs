mod agents;
mod notebook;

use agents::{ReaderAgent, ReaderAgentExt};
use notebook::NotebookStrategy;

const TEXT: &str = "The Rust 1.0 release came out on May 15, 2015, after about five years of development. \
Its borrow checker rules out data races at compile time. Mozilla sponsored the language until 2020, \
when the Rust Foundation was founded by AWS, Google, Huawei, Microsoft and Mozilla.";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let agent = ReaderAgent::default().await;
    match agent
        .run_with_strategy_immed(NotebookStrategy::new(), TEXT)
        .await
    {
        Ok(output) => {
            // The agent's answer, and what the strategy produced next to it.
            println!("Summary: {}\n", output.content);
            for (i, note) in output.extra_content.notes.iter().enumerate() {
                println!("{}. {note}", i + 1);
            }
            println!("\nTool calls: {:?}", output.extra_content.calls);
        }
        Err(e) => eprintln!("{e}"),
    }
}
