use std::error::Error;

use talin::tool;

pub struct SendEmailTool;

#[tool]
impl SendEmailTool {
    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        // A real tool would send the email. This one only pretends to.
        Ok(format!("Email to {} sent.", input.to))
    }

    fn from_config(_: _) -> Self {
        Self
    }
}
