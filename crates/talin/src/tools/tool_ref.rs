use std::sync::Arc;

use crate::tools::{FunctionTool, Tool};

/// A borrowed reference to a [`Tool`].
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum ToolRef<'a> {
    /// A function tool.
    Function(&'a dyn FunctionTool),
}

/// A type that can be borrowed as a [`ToolRef`].
pub trait AsToolRef {
    /// Borrows this value as a [`ToolRef`].
    fn as_tool_ref(&self) -> ToolRef<'_>;
}

impl AsToolRef for Tool {
    fn as_tool_ref(&self) -> ToolRef<'_> {
        match self {
            Tool::Function(func) => ToolRef::Function(func.as_ref()),
        }
    }
}
impl AsToolRef for ToolRef<'_> {
    fn as_tool_ref(&self) -> ToolRef<'_> {
        *self
    }
}

impl<T: FunctionTool> AsToolRef for T {
    fn as_tool_ref(&self) -> ToolRef<'_> {
        ToolRef::Function(self)
    }
}
impl AsToolRef for dyn FunctionTool {
    fn as_tool_ref(&self) -> ToolRef<'_> {
        ToolRef::Function(self)
    }
}
impl AsToolRef for Box<dyn FunctionTool> {
    fn as_tool_ref(&self) -> ToolRef<'_> {
        ToolRef::Function(self.as_ref())
    }
}
impl AsToolRef for Arc<dyn FunctionTool> {
    fn as_tool_ref(&self) -> ToolRef<'_> {
        ToolRef::Function(self.as_ref())
    }
}
