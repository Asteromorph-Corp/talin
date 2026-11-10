use async_trait::async_trait;
use talin::chain::PartialTarget;
use talin::llm::Llm;
use talin::strategy::Strategy;

use crate::types::CustomOutputAgentOutputCtor;

pub struct ValidationStrategy;

#[async_trait]
impl<L> Strategy<L, CustomOutputAgentOutputCtor> for ValidationStrategy
where
    L: Llm,
{
    type Output = ();
    type Error = ValidationError;

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    async fn process_output(
        &mut self,
        output: PartialTarget<CustomOutputAgentOutputCtor>,
    ) -> Result<PartialTarget<CustomOutputAgentOutputCtor>, Self::Error> {
        log::info!("Validation is running");
        if output.name.len() > 20 {
            return Err(ValidationError::NameTooLong);
        }
        Ok(output)
    }
}

#[derive(Debug)]
pub enum ValidationError {
    NameTooLong,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::NameTooLong => write!(f, "`name` is too long"),
        }
    }
}

impl std::error::Error for ValidationError {}
