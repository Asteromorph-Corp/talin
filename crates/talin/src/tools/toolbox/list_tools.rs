use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use super::Toolbox;
use crate::tools::{Function, ToolExt, ToolSpec};

/// A tool that lists all tools available in a toolbox.
///
/// This tool is added to the agent's tool list when providing agents with toolboxes.
pub struct ListTools<T>(Arc<T>)
where
    T: Toolbox + ?Sized;

impl<T> ListTools<T>
where
    T: Toolbox + ?Sized + 'static,
{
    /// Creates a new `ListTools` instance wrapping the given toolbox.
    pub fn new(toolbox: &Arc<T>) -> Self {
        Self(Arc::clone(toolbox))
    }
}

impl<T> std::fmt::Debug for ListTools<T>
where
    T: Toolbox + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolWrapper")
            .field(
                "tool",
                &format_args!(
                    "{{ name: {}, description: {}, parameters: {:#?}, strict: {} }}",
                    Function::name(self),
                    self.description(),
                    self.parameters(),
                    self.strict()
                ),
            )
            .finish()
    }
}

#[async_trait]
impl<T> Function for ListTools<T>
where
    T: Toolbox + ?Sized,
{
    type Input = ();
    type Output = String;

    fn name(&self) -> String {
        format!("List tools in {}", self.0.name())
    }

    fn description(&self) -> String {
        format!("List all tools in the toolbox {}", self.0.name())
    }

    async fn parse_input(&self, _input: Value) -> Result<Self::Input, serde_json::Error> {
        Ok(())
    }

    async fn call(&self, _: ()) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let tool_descriptions: Vec<String> = self
            .0
            .resolver()
            .list_tools()
            .iter()
            .map(|tool| match tool.get_spec() {
                ToolSpec::Function(func) => func.describe(),
            })
            .collect();
        Ok(tool_descriptions.join("\n---\n"))
    }
}
