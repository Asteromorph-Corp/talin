use proc_macro2_diagnostics::{Diagnostic, Level};
use syn::spanned::Spanned;

#[derive(Default)]
pub(crate) struct StructAttrs {
    pub crate_path: Option<syn::Path>,
    pub derive_schema: bool,
    pub from_input: Option<syn::Type>,
}

impl StructAttrs {
    pub(super) fn aggregate(mut acc: Self, attr: &syn::Attribute) -> Result<Self, Diagnostic> {
        if !attr.path().is_ident("chain") {
            return Ok(acc);
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                if acc.crate_path.is_some() {
                    return Err(syn::Error::new(
                        meta.path.span(),
                        "Duplicate 'crate' attribute",
                    ));
                }

                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                acc.crate_path = Some(syn::parse_str(&lit.value()).expect("Invalid crate path"));
            } else if meta.path.is_ident("from_input") {
                if acc.from_input.is_some() {
                    return Err(syn::Error::new(
                        meta.path.span(),
                        "Duplicate 'from_input' attribute",
                    ));
                }

                let value = meta.value()?;
                acc.from_input = Some(value.parse()?);
            } else if meta.path.is_ident("derive_schema") {
                if acc.derive_schema {
                    return Err(syn::Error::new(
                        meta.path.span(),
                        "Duplicate 'derive_schema' attribute",
                    ));
                }

                acc.derive_schema = true;
            } else {
                return Err(syn::Error::new(meta.path.span(), "Unknown attribute"));
            }

            Ok(())
        })
        .map_err(|e| {
            Diagnostic::spanned(
                e.span(),
                Level::Error,
                format!("Failed to parse serde attribute: {e}"),
            )
        })?;

        Ok(acc)
    }
}
