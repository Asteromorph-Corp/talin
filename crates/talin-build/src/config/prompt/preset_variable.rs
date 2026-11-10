use std::str::FromStr;

use syn::parse_quote;

use crate::codegen::common::talin;
use crate::error::PromptError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PresetVariable {
    CurrentDate,
    CurrentDatetime,
}

impl PresetVariable {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            PresetVariable::CurrentDate => "current_date",
            PresetVariable::CurrentDatetime => "current_datetime",
        }
    }

    pub(crate) fn get_type(&self) -> syn::Type {
        match self {
            PresetVariable::CurrentDate | PresetVariable::CurrentDatetime => {
                parse_quote!(String)
            }
        }
    }

    pub(crate) fn get_expr(&self) -> syn::Expr {
        let talin = talin();
        match self {
            PresetVariable::CurrentDate => {
                parse_quote! { #talin::__private::chrono::Local::now().format("%Y-%m-%d").to_string() }
            }
            PresetVariable::CurrentDatetime => {
                parse_quote! { #talin::__private::chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string() }
            }
        }
    }
}

impl FromStr for PresetVariable {
    type Err = PromptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "current_date" => Ok(PresetVariable::CurrentDate),
            "current_datetime" | "current_date_time" => Ok(PresetVariable::CurrentDatetime),
            other => Err(PromptError::UnknownPresetVariable(other.to_owned())),
        }
    }
}
