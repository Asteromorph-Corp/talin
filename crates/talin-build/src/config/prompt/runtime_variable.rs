use heck::ToSnakeCase;

use crate::config::{InputVariable, PresetVariable};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum RuntimeVariable {
    Input(Box<InputVariable>),
    InputMedia(Box<InputVariable>),
    Preset(PresetVariable),
    Env(String),
}

impl RuntimeVariable {
    pub(crate) fn name(&self) -> String {
        match self {
            RuntimeVariable::Input(var) => var.ident.to_string(),
            RuntimeVariable::InputMedia(var) => var.ident.to_string(),
            RuntimeVariable::Preset(var) => format!("_preset_{}", var.name()),
            RuntimeVariable::Env(name) => format!("_env_{}", name.to_snake_case()),
        }
    }

    pub(crate) fn get_ident(&self) -> syn::Ident {
        match self {
            RuntimeVariable::Input(var) => var.ident.clone(),
            RuntimeVariable::InputMedia(var) => var.ident.clone(),
            _ => syn::Ident::new(&self.name(), proc_macro2::Span::call_site()),
        }
    }

    pub(crate) fn get_type(&self) -> syn::Type {
        match self {
            RuntimeVariable::Input(var) => var.type_.clone(),
            RuntimeVariable::InputMedia(var) => var.type_.clone(),
            RuntimeVariable::Preset(var) => var.get_type(),
            RuntimeVariable::Env(_) => syn::parse_str("String").unwrap(),
        }
    }
}
