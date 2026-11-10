use std::str::FromStr;

use syn::parse_quote;

use crate::error::InputVariableError;
use crate::ty::{extract_array_like_type, is_string_like_type, lifetime_a, rename_lifetimes};

/// Represents an input variable with a name and an optional type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct InputVariable {
    pub ident: syn::Ident,
    pub type_: syn::Type,
}

impl InputVariable {
    pub(crate) fn new(name: &str, type_: Option<&str>) -> Result<Self, InputVariableError> {
        let ident = syn::parse_str::<syn::Ident>(name)
            .map_err(|e| InputVariableError::InvalidIdent(name.to_owned(), e))?;
        let lifetime = lifetime_a();
        let type_ = match type_ {
            Some(type_) => {
                let ty = syn::parse_str::<syn::Type>(type_).map_err(|e| {
                    InputVariableError::InvalidType(name.to_owned(), type_.to_owned(), e)
                })?;

                if is_string_like_type(&ty) {
                    parse_quote! { std::borrow::Cow<#lifetime, str> }
                } else if let Some(inner) = extract_array_like_type(&ty) {
                    let inner = rename_lifetimes(inner.clone(), Some(&lifetime));
                    parse_quote! { std::borrow::Cow<#lifetime, [#inner]> }
                } else {
                    rename_lifetimes(ty, Some(&lifetime))
                }
            }
            None => parse_quote! { std::borrow::Cow<#lifetime, str> },
        };

        Ok(InputVariable { ident, type_ })
    }
}

impl FromStr for InputVariable {
    type Err = InputVariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, type_) = match s.trim().split_once(':') {
            Some((name, type_)) => (name.trim(), Some(type_.trim())),
            None => (s.trim(), None),
        };
        Self::new(name, type_)
    }
}
