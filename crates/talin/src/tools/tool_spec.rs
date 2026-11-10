use crate::tools::FunctionSpec;

/// Specification for a tool available to the LLM.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ToolSpec {
    /// A specification for a function tool.
    Function(FunctionSpec),
}

impl ToolSpec {
    /// Returns the name of the tool.
    pub fn name(&self) -> &str {
        match self {
            ToolSpec::Function(func) => &func.name,
        }
    }

    /// Extracts the function specs from a collection of tool specifications.
    pub fn into_functions(tools: Vec<ToolSpec>) -> Vec<FunctionSpec> {
        tools
            .into_iter()
            .map(|tool| match tool {
                ToolSpec::Function(func) => func,
            })
            .collect()
    }
}

impl From<FunctionSpec> for ToolSpec {
    fn from(func: FunctionSpec) -> Self {
        ToolSpec::Function(func)
    }
}
