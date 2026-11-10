use talin::chain::Chain;
use talin::sequential_chain;

use crate::agents::{
    BudgetEstimatorAgent, PlaceRecommenderAgent, PlaceRecommenderAgentInput, SummaryGeneratorAgent,
};

mod agents;
mod types;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let sequential = sequential_chain!(
        PlaceRecommenderAgent::default().await,
        BudgetEstimatorAgent::default().await,
        SummaryGeneratorAgent::default().await
    );

    let input = PlaceRecommenderAgentInput::new("Seoul, Korea");
    let result = sequential.call(input).await;

    println!("------------FINISH------------");

    println!("Result: {result:?}");
}
