mod agents;
mod tools;

use agents::{ShopperAgent, ShopperAgentExt, UptownShopperAgent, UptownShopperAgentExt};

// The second title is missing an "s" on purpose, so `check_stock` fails once and the model has to
// recover.
const REQUEST: &str = "Order one copy of 'The Glass Orchid' and two copies of 'Salt and Lantern', if they are in stock.";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let agent = ShopperAgent::default().await;
    match agent.run_immed(REQUEST).await {
        Ok(output) => println!("Downtown: {output}"),
        Err(e) => return eprintln!("{e}"),
    }

    // The same tools, configured for another store.
    let agent = UptownShopperAgent::default().await;
    match agent.run_immed(REQUEST).await {
        Ok(output) => println!("Uptown: {output}"),
        Err(e) => eprintln!("{e}"),
    }
}
