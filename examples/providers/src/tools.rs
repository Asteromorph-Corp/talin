use std::error::Error;

use talin::tool;

pub struct CurrentTimeTool;

#[tool]
impl CurrentTimeTool {
    async fn call(
        &self,
        _input: Self::Input,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        // A fixed answer keeps runs comparable across providers.
        Ok("2026-10-29 12:00 UTC".into())
    }

    fn from_config(_: _) -> Self {
        Self
    }
}
