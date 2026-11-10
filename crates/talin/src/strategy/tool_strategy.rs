use std::convert::Infallible;

use async_trait::async_trait;

use crate::chain::OutputCtor;
use crate::llm::Llm;
use crate::strategy::Strategy;
use crate::tools::{FunctionTool, ToolRef, ToolResolver, normalize_tool_name};

/// A [`Strategy`] that provides an additional function tool to the agent.
///
/// This can be used to dynamically add tools to an agent's toolkit.
///
/// Returns the tool itself upon finalization.
pub struct ToolStrategy<T: FunctionTool + 'static> {
    tool: T,
}

impl<T: FunctionTool> ToolStrategy<T> {
    /// Creates a new [`ToolStrategy`] with the given function tool.
    pub fn new(tool: T) -> Self {
        Self { tool }
    }
}

impl<'a, T: FunctionTool> ToolResolver<'a> for &'a ToolStrategy<T> {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        vec![ToolRef::Function(&self.tool)]
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
        let tool: &dyn FunctionTool = &self.tool;
        (normalize_tool_name(&self.tool.name()) == normalize_tool_name(tool_name)).then_some(tool)
    }
}

/// Implementation of [`Strategy`] for [`ToolStrategy`].
///
/// This strategy provides an additional function tool to the agent and returns the tool itself upon
/// finalization.
#[async_trait]
impl<T, L, O> Strategy<L, O> for ToolStrategy<T>
where
    T: FunctionTool,
    L: Llm,
    O: OutputCtor,
{
    type Output = T;
    type Error = Infallible;

    fn additional_tools<'a>(&'a self) -> impl ToolResolver<'a> + 'a {
        self
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(self.tool)
    }
}
