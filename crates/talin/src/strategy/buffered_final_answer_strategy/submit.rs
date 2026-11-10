use async_trait::async_trait;

use crate::tools::{EmptyFunctionInput, Function};

/// A tool that submits the accumulated answer. The call is intercepted by
/// [`BufferedFinalAnswerStrategy`] and never actually executed.
///
/// [`BufferedFinalAnswerStrategy`]: super::BufferedFinalAnswerStrategy
pub(super) struct SubmitFinalAnswer;

pub(super) const SUBMIT_FINAL_ANSWER: &str = "submit_final_answer";

#[async_trait]
impl Function for SubmitFinalAnswer {
    type Input = EmptyFunctionInput;
    type Output = String;

    fn name(&self) -> String {
        SUBMIT_FINAL_ANSWER.to_owned()
    }

    fn description(&self) -> String {
        "Submit the accumulated answer as the final answer to the user.".to_owned()
    }

    async fn call(
        &self,
        _input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
        Ok(String::new())
    }
}
