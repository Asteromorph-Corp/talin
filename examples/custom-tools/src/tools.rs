//! The tools.

use std::collections::HashMap;
use std::error::Error;

use talin::schemas::{ContentPart, ToolOutput};
use talin::tool;
use talin::transcript::EntryView;
use talin::transcript::query::count_tool_calls;

/// The books the shop carries, and how many copies the downtown and uptown stores have.
const UPTOWN_STOCK: [(&str, u32); 3] = [
    ("The Glass Orchid", 0),
    ("Salt and Lanterns", 5),
    ("The Cartographer's Daughter", 2),
];
const DOWNTOWN_STOCK: [(&str, u32); 3] = [
    ("The Glass Orchid", 3),
    ("Salt and Lanterns", 1),
    ("The Cartographer's Daughter", 0),
];

fn stock_for(store: &str) -> [(&'static str, u32); 3] {
    match store {
        "Downtown" => DOWNTOWN_STOCK,
        "Uptown" => UPTOWN_STOCK,
        _ => panic!("Unknown store: {}", store),
    }
}

pub struct CheckStockTool {
    store: String,
    stock: HashMap<&'static str, u32>,
}

pub struct PlaceOrderTool;

#[tool]
impl CheckStockTool {
    // Implementing a custom `description` lets you attach extra context to the tool's description.
    //
    // You can reference the description in `tools.toml` using `TOOL_DESCRIPTION`.
    fn description(&self) -> String {
        format!("{TOOL_DESCRIPTION} Looks in the {} store.", self.store)
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        match self.stock.get(&input.title.as_str()) {
            Some(copies) => Ok(format!("{copies} copies of {} in stock", input.title)),
            // Should the tool fail, provide a helpful error message so the model can adjust its
            // query.
            None => {
                let known = self.stock.keys().cloned().collect::<Vec<_>>().join(",");
                Err(format!("no book titled {:?}. Known titles: {}", input.title, known).into())
            }
        }
    }

    // Short descriptions of each call, for the logs.
    fn summarize_input(&self, input: &Self::Input) -> Option<String> {
        Some(format!("Checked the stock of {}", input.title))
    }

    fn from_config(config: _) -> Self {
        let stock = stock_for(&config.store);
        Self {
            store: config.store,
            stock: HashMap::from_iter(stock),
        }
    }
}

#[tool]
impl PlaceOrderTool {
    type Output = ToolOutput;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        // `ToolOutput` can hold several content parts and a summary, unlike a plain `String`.
        Ok(ToolOutput::new(vec![
            ContentPart::Text(format!(
                "Ordered {} copies of {}.",
                input.copies, input.title
            )),
            ContentPart::Text("The order ships in 3 days.".into()),
        ])
        .with_sumary(format!("ordered {}x {}", input.copies, input.title)))
    }

    // The model may call this tool at most twice. After that the tool is left out of requests and
    // further calls are rejected.
    fn available(&self, transcript: &[EntryView<'_>]) -> bool {
        count_tool_calls(transcript, "place_order") < 2
    }

    // A tool with no `config` in `tools.toml` still has to say how it is built.
    fn from_config(_: _) -> Self {
        Self
    }
}
