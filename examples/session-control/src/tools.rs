use std::cmp::Ordering;
use std::error::Error;
use std::fmt::Display;

use talin::schemas::ContentPart;
use talin::tool;

pub struct UpOrDownTool {
    answer: u8,
}

pub struct UpOrDownToolOutput(pub Ordering);

impl Display for UpOrDownToolOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Ordering::Less => write!(f, "Your guess is lower than the answer"),
            Ordering::Equal => write!(f, "Your guess is correct"),
            Ordering::Greater => write!(f, "Your guess is higher than the answer"),
        }
    }
}

impl From<UpOrDownToolOutput> for ContentPart {
    fn from(output: UpOrDownToolOutput) -> Self {
        ContentPart::Text(output.to_string())
    }
}

#[tool]
impl UpOrDownTool {
    type Output = UpOrDownToolOutput;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok(UpOrDownToolOutput(input.guess.cmp(&self.answer)))
    }

    fn summarize_input(&self, input: &Self::Input) -> Option<String> {
        Some(format!("Guessed {}", input.guess))
    }

    fn summarize_output(&self, result: &Self::Output) -> Option<String> {
        match result.0 {
            Ordering::Less => Some(", lower than the answer".to_string()),
            Ordering::Equal => Some(", correct!".to_string()),
            Ordering::Greater => Some(", higher than the answer".to_string()),
        }
    }

    fn from_config(config: _) -> Self {
        Self {
            answer: config.answer,
        }
    }
}
