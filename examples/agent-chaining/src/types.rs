#![allow(dead_code)]

use std::borrow::Cow;

use talin::chain::{ChainOutput, Ctor};

use crate::agents::{
    BudgetEstimatorAgentInput, PlaceRecommenderAgentInput, SummaryGeneratorAgentInput,
};

#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema, from_input = PlaceRecommenderAgentInput<'a>)]
#[serde(deny_unknown_fields)]
pub struct PlaceRecommenderAgentOutput<'a> {
    pub places: Vec<String>,
    #[chain(from = input)]
    pub destination: Cow<'a, str>,
}

#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema, from_input = BudgetEstimatorAgentInput<'a>)]
#[serde(deny_unknown_fields)]
pub struct BudgetEstimatorAgentOutput<'a> {
    /// The estimated budget in USD recommended by the BudgetEstimatorAgent
    pub estimated_budget_usd: f64,

    /// The places recommended by the PlaceRecommenderAgent
    #[chain(from = input)]
    pub places: Vec<String>,

    /// The destination provided to the PlaceRecommenderAgent
    #[chain(from = input)]
    pub destination: Cow<'a, str>,
}

#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema, from_input = SummaryGeneratorAgentInput<'a>)]
#[serde(deny_unknown_fields)]
pub struct SummaryGeneratorAgentOutput<'a> {
    #[chain(from = response)]
    pub summary: String,
    #[chain(from = input)]
    pub estimated_budget_usd: f64,
    #[chain(from = input)]
    pub places: Vec<String>,
    #[chain(from = input)]
    pub destination: Cow<'a, str>,
}

impl<'a> From<PlaceRecommenderAgentOutput<'a>> for BudgetEstimatorAgentInput<'a> {
    fn from(output: PlaceRecommenderAgentOutput<'a>) -> Self {
        BudgetEstimatorAgentInput {
            places: output.places.into(),
            destination: output.destination,
        }
    }
}

impl<'a> From<BudgetEstimatorAgentOutput<'a>> for SummaryGeneratorAgentInput<'a> {
    fn from(output: BudgetEstimatorAgentOutput<'a>) -> Self {
        SummaryGeneratorAgentInput {
            estimated_budget_usd: output.estimated_budget_usd,
            places: output.places.into(),
            destination: output.destination,
        }
    }
}
