mod agents;
mod tools;

use agents::{Models, SolverAgent, SolverAgentExt, SolverQuickAgent, SolverQuickAgentExt};
use talin::schemas::Message;
use talin::strategy::{RecordStrategy, UsageTrackStrategy};

const QUESTION: &str = "I have 2,500 EUR, 300 GBP, 120,000 JPY, 3,000,000 KRW and 1,800 CHF. \
Converting any non-USD currency to USD costs a 1.5% fee, taken from the converted amount. \
Rank the five holdings by their USD value after the fee. By how much does the best beat the second? \
How many KRW would I need to add to the last-ranked holding for it to match the best one?";

/// Prints the model's reasoning from every step of a recorded run.
fn print_thoughts(record: &[Message]) {
    for (i, message) in record.iter().enumerate() {
        print!("Step {i}: ");

        let thought = match message {
            Message::Assistant(m) => m.thought.as_ref(),
            Message::ToolCall(m) => m.thought.as_ref(),
            _ => None,
        };
        match thought {
            Some(thought) => println!("{thought}\n"),
            None => println!("[no reasoning for step]\n"),
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let model = std::env::args().nth(1).unwrap_or("claude".into());
    log::info!("Using model: {model}");

    // High effort, reading back what the model thought at each step.
    //
    // Strategies passed as a tuple all run on the same turn, and their outputs come back as a
    // tuple.
    //
    let agent = SolverAgent::any(Models::any(&model).unwrap_or_else(|e| panic!("{e}"))).await;
    let strategies = (RecordStrategy::new(), UsageTrackStrategy::default());
    let output = match agent.run_with_strategy_immed(strategies, QUESTION).await {
        Ok(output) => output,
        Err(e) => return eprintln!("Error: {e}"),
    };
    let (record, usage) = output.extra_content;
    println!("------------THOUGHT (high effort)------------");
    print_thoughts(&record);
    println!("------------RESULT (high effort)------------");
    println!("Answer: {}\n{usage}\n", output.content);

    // Low effort: the same agent with a different `reasoning_effort`, so compare the token usage.
    let agent = SolverQuickAgent::any(Models::any(&model).unwrap_or_else(|e| panic!("{e}"))).await;
    let strategies = (RecordStrategy::new(), UsageTrackStrategy::default());
    let output = match agent.run_with_strategy_immed(strategies, QUESTION).await {
        Ok(output) => output,
        Err(e) => return eprintln!("Error: {e}"),
    };
    let (record, usage) = output.extra_content;
    println!("------------THOUGHT (low effort)------------");
    print_thoughts(&record);
    println!("------------RESULT (low effort)------------");
    println!("Answer: {}\n{usage}\n", output.content);
}
