//! Simple toolbox implementation.
//!
//! This module provides [`SimpleToolbox`], a straightforward implementation
//! of the [`Toolbox`] trait that stores tools in a `HashMap`.

use std::collections::HashMap;

use super::Toolbox;
use crate::tools::{AsToolRef, ToolExt, ToolResolver, normalize_tool_name};

/// A simple implementation of [`Toolbox`] using a [`ToolResolver`].
pub struct SimpleToolbox<R>
where
    R: Send + Sync,
    for<'a> &'a R: ToolResolver<'a>,
{
    name: String,
    tools: R,
}

impl<R> SimpleToolbox<R>
where
    R: Send + Sync,
    for<'a> &'a R: ToolResolver<'a>,
{
    /// Creates a new toolbox with the given name and pre-built tools map.
    pub fn new(name: impl Into<String>, tools: R) -> Self {
        Self {
            name: name.into(),
            tools,
        }
    }
}

impl<E: AsToolRef + Send + Sync> SimpleToolbox<HashMap<String, E>> {
    /// Creates a new toolbox from an iterator of tools.
    pub fn from_tools(name: impl Into<String>, tools: impl IntoIterator<Item = E>) -> Self {
        let tools = tools
            .into_iter()
            .map(|tool| (normalize_tool_name(&tool.name()), tool))
            .collect::<HashMap<_, _>>();
        Self {
            name: name.into(),
            tools,
        }
    }
}

impl<R> Toolbox for SimpleToolbox<R>
where
    R: Send + Sync,
    for<'a> &'a R: ToolResolver<'a>,
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn resolver(&self) -> Box<dyn ToolResolver<'_> + '_> {
        Box::new(&self.tools)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::SimpleToolbox;
    use crate::tools::{EmptyFunctionInput, Function, FunctionTool, Toolbox};

    /// A tool whose display name is not already normalized (contains a space and capitals).
    struct SpacedNameTool;

    #[async_trait]
    impl Function for SpacedNameTool {
        type Input = EmptyFunctionInput;
        type Output = String;

        fn name(&self) -> String {
            "Web Scraper".to_string()
        }

        fn description(&self) -> String {
            "A tool with a non-normalized display name.".to_string()
        }

        async fn call(
            &self,
            _input: Self::Input,
        ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
            Ok("scraped".to_string())
        }
    }

    #[test]
    fn from_tools_resolves_by_normalized_name() {
        let tool: Arc<dyn FunctionTool> = Arc::new(SpacedNameTool);
        let toolbox = SimpleToolbox::from_tools("web_tools", [tool]);
        let resolver = toolbox.resolver();

        // "Web Scraper" normalizes to "web_scraper", and the resolver normalizes the queried name,
        // so both forms must resolve. Before the fix the map was keyed by the raw name, so a lookup
        // of the (normalized) name the model actually sees found nothing.
        assert!(
            resolver.resolve_tool("web_scraper").is_some(),
            "should resolve by the normalized name the model is shown"
        );
        assert!(
            resolver.resolve_tool("Web Scraper").is_some(),
            "should also resolve by the raw display name"
        );
        assert!(resolver.resolve_tool("nonexistent").is_none());
    }
}
