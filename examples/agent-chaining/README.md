# `agent-chaining`

This example shows how to run agents in sequence, so the output of one becomes the input of the next.

```sh
OPENAI_API_KEY=... cargo run
```

A trip planner chains three agents:

1. `place_recommender` suggests places to visit in a destination.
2. `budget_estimator` estimates the cost of visiting those places.
3. `summary_generator` writes a summary of the trip and its budget.

```rust
let sequential = sequential_chain!(
    PlaceRecommenderAgent::default().await,
    BudgetEstimatorAgent::default().await,
    SummaryGeneratorAgent::default().await
);
let result = sequential.call(PlaceRecommenderAgentInput::new("Seoul, Korea")).await;
```

## Passing the Output

In a multi-agent chain, the output of one agent becomes the input of the next.
To make this handoff seamless and type-safe, you need to define how the preceding agent's output maps to the next agent's input fields.

This mapping is implemented using Rust's From trait:
```rust
impl<'a> From<PlaceRecommenderAgentOutput<'a>> for BudgetEstimatorAgentInput<'a> { /* ... */ }
```

The chain will compile only if all required `From` implementations exist.

## Input Passthrough

You may want to pass some fields from the input to the output of an agent, so that the next agent can use them.
Rather than asking the model to repeat it, the output structs can copy fields from their input as part of the `ChainOutput` derive:

```rust
#[derive(ChainOutput)]
#[chain(derive_schema, from_input = BudgetEstimatorAgentInput<'a>)]
pub struct BudgetEstimatorAgentOutput<'a> {
    pub estimated_budget_usd: f64,        // from the model's answer

    #[chain(from = input)]
    pub destination: Cow<'a, str>,        // copied from the input
}
```

Since you need to customize your `ChainOutput` implementation, you cannot rely on the output struct generated with the `response_format`.
Instead, you need to set `response_format = "custom"` and define the output struct yourself, see [`custom-struct`](../custom-struct) and [`structured-output`](../structured-output).
