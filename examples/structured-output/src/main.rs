use crate::agents::{
    CustomOutputAgent, CustomOutputAgentExt, CustomOutputAgentOutput, StructuredOutputAgent,
    StructuredOutputAgentExt, StructuredOutputAgentOutput,
};
use crate::strategy::ValidationStrategy;

mod agents;
mod strategy;
mod types;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let input = "My name is John Doe, I live in New York, and my email is john.doe@example.com";

    // Schema declared in `agents.toml`
    let agent = StructuredOutputAgent::default().await;
    let result: Result<StructuredOutputAgentOutput, _> = agent.run_immed(input).await;

    // Schema declared by the custom struct above
    let agent = CustomOutputAgent::default().await;
    let custom_result: Result<CustomOutputAgentOutput, _> = agent.run_immed(input).await;

    // Custom struct validated by `ValidationStrategy`
    let validated_result = agent
        .run_with_strategy_immed(ValidationStrategy, input)
        .await
        .map(|output| output.content);

    println!("------------FINISH------------");

    match result {
        Ok(output) => println!("{output:#?}"),
        Err(e) => eprintln!("{e}"),
    }
    match custom_result {
        Ok(output) => println!("{output:#?}"),
        Err(e) => eprintln!("{e}"),
    }
    match validated_result {
        Ok(output) => println!("{output:#?}"),
        Err(e) => eprintln!("{e}"),
    }
}
