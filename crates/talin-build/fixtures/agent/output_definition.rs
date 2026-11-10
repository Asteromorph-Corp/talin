#[derive(Debug, Clone, ::talin::chain::Ctor, ::talin::chain::ChainOutput)]
#[chain(crate = "::talin")]
#[serde(deny_unknown_fields)]
#[schemars(title = "MyResponse", description = "A structured agent response.")]
#[chain(derive_schema)]
#[allow(dead_code)]
pub struct MyAgentOutput {
    ///The final answer.
    pub answer: String,
    #[schemars(with = "i32")]
    pub score: u8,
}
