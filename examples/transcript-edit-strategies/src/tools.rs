use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};

use talin::tool;

/// A deployment that is further along each time it is checked.
pub struct CheckDeployTool {
    checks: AtomicUsize,
}

#[tool]
impl CheckDeployTool {
    async fn call(
        &self,
        _input: Self::Input,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        let progress = match self.checks.fetch_add(1, Ordering::SeqCst) {
            0 => 20,
            1 => 55,
            2 => 90,
            _ => 100,
        };
        Ok(format!("The deployment is {progress}% done."))
    }

    fn from_config(_: _) -> Self {
        Self {
            checks: AtomicUsize::new(0),
        }
    }
}
