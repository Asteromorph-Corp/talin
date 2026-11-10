use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::quote;
use syn::ext::IdentExt;
use syn::spanned::Spanned;

use crate::ast::extract_fields;
use crate::macros::crate_path::talin;

pub(crate) fn derive_tool_config(
    input: syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let talin = talin();
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = extract_fields(input.data.clone())?
        .into_iter()
        .map(|field| {
            let Some(ident) = &field.ident else {
                return Err(Diagnostic::spanned(
                    field.span(),
                    Level::Error,
                    "Expected a named field",
                ));
            };
            let name = ident.unraw().to_string();
            let ty = &field.ty;

            // The schema of the field's type, or a required leaf if the type has none.
            Ok(quote! {
                #talin::tools::config::SchemaField::new(
                    #name,
                    #talin::__private::ConfigProbe::<#ty>::SCHEMA,
                )
            })
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #talin::tools::ToolConfig
            for #struct_name #ty_generics #where_clause
        {
            const SCHEMA: #talin::tools::config::SchemaDef = {
                #[allow(unused_imports)]
                use #talin::__private::ConfigProbeFallback as _;
                const FIELDS: &[#talin::tools::config::SchemaField] = &[#(#fields),*];
                #talin::tools::config::SchemaDef::node(FIELDS)
            };
        }
    })
}
