use proc_macro2_diagnostics::Diagnostic;

use crate::attrs::{FieldAttrs, StructAttrs};

fn extract_attr<T: Default>(
    attrs: &[syn::Attribute],
    get_attr: impl Fn(T, &syn::Attribute) -> Result<T, Diagnostic>,
) -> Result<T, Diagnostic> {
    attrs.iter().try_fold(T::default(), get_attr)
}

pub(crate) fn extract_field_attr(attrs: &[syn::Attribute]) -> Result<FieldAttrs, Diagnostic> {
    extract_attr(attrs, FieldAttrs::aggregate)
}

pub(crate) fn extract_struct_attr(attrs: &[syn::Attribute]) -> Result<StructAttrs, Diagnostic> {
    extract_attr(attrs, StructAttrs::aggregate)
}
