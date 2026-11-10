use std::collections::HashMap;

use crate::tools::{AsToolRef, FunctionTool, ToolExt, ToolRef, normalize_tool_name};

/// A `ToolResolver` is a type that can list and resolve tools by name.
///
/// It is used to abstract over different ways of storing and accessing tools.
pub trait ToolResolver<'a> {
    /// Returns a list of all tools in this resolver, in no particular order.
    fn list_tools(&self) -> Vec<ToolRef<'a>>;
    /// Returns a reference to the tool with the given name, if it exists in this resolver.
    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool>;
}

impl<'a, E: AsToolRef> ToolResolver<'a> for &'a E {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        vec![self.as_tool_ref()]
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
        let normalized = normalize_tool_name(tool_name);
        let func = self.as_function()?;
        (normalize_tool_name(&func.name()) == normalized).then_some(func)
    }
}

impl<'a, E: AsToolRef> ToolResolver<'a> for &'a [E] {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        self.iter().map(AsToolRef::as_tool_ref).collect()
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
        let normalized = normalize_tool_name(tool_name);
        self.iter().find_map(|entry| {
            let func = entry.as_function()?;
            (normalize_tool_name(&func.name()) == normalized).then_some(func)
        })
    }
}

impl<'a, E: AsToolRef> ToolResolver<'a> for &'a HashMap<String, E> {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        self.values().map(AsToolRef::as_tool_ref).collect()
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
        self.get(&normalize_tool_name(tool_name))
            .and_then(|entry| entry.as_function())
    }
}

impl<'a, R: ToolResolver<'a>> ToolResolver<'a> for Option<R> {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        self.as_ref().map(|r| r.list_tools()).unwrap_or_default()
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
        self.as_ref().and_then(|r| r.resolve_tool(tool_name))
    }
}

impl<'a> ToolResolver<'a> for () {
    fn list_tools(&self) -> Vec<ToolRef<'a>> {
        vec![]
    }

    fn resolve_tool(&self, _tool_name: &str) -> Option<&'a dyn FunctionTool> {
        None
    }
}

/// Composes resolvers: tools are listed/resolved left-to-right, so earlier tuple members override
/// later ones on name collision. This is what lets a tuple of strategies expose all their tools
/// without boxing or allocation.
macro_rules! impl_tool_resolver_for_tuple {
    ( $( $R:ident ),+ ) => {
        #[allow(non_snake_case)]
        impl<'a, $( $R: ToolResolver<'a> ),+> ToolResolver<'a> for ( $( $R, )+ ) {
            fn list_tools(&self) -> Vec<ToolRef<'a>> {
                let ( $( $R, )+ ) = self;
                let mut specs = Vec::new();
                $( specs.extend($R.list_tools()); )+
                specs
            }

            fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn FunctionTool> {
                let ( $( $R, )+ ) = self;
                None $( .or_else(|| $R.resolve_tool(tool_name)) )+
            }
        }
    };
}

impl_tool_resolver_for_tuple!(R0);
impl_tool_resolver_for_tuple!(R0, R1);
impl_tool_resolver_for_tuple!(R0, R1, R2);
impl_tool_resolver_for_tuple!(R0, R1, R2, R3);
impl_tool_resolver_for_tuple!(R0, R1, R2, R3, R4);
impl_tool_resolver_for_tuple!(R0, R1, R2, R3, R4, R5);
impl_tool_resolver_for_tuple!(R0, R1, R2, R3, R4, R5, R6);
impl_tool_resolver_for_tuple!(R0, R1, R2, R3, R4, R5, R6, R7);
impl_tool_resolver_for_tuple!(R0, R1, R2, R3, R4, R5, R6, R7, R8);
impl_tool_resolver_for_tuple!(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9);
