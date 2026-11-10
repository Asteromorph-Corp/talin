use proc_macro2_diagnostics::{Diagnostic, Level};
use syn::parse_quote;
use syn::spanned::Spanned;

use crate::macros::crate_path::talin;
use crate::macros::symbols::get_config_symbol;

pub(super) fn impl_from_config(
    tool_id: &str,
    tool_ty: &syn::Type,
    fn_from_config: &syn::ImplItemFn,
) -> Result<syn::ItemImpl, Diagnostic> {
    let talin = talin();
    let mut fn_from_config = fn_from_config.clone();

    let [syn::FnArg::Typed(syn::PatType { ty, .. })] =
        &mut fn_from_config.sig.inputs.iter_mut().collect::<Vec<_>>()[..]
    else {
        return Err(Diagnostic::spanned(
            fn_from_config.sig.span(),
            Level::Error,
            "The `from_config` method should receive exactly one typed argument.",
        ));
    };

    let config_ty = match &**ty {
        syn::Type::Infer(_) => {
            let config_ident = get_config_symbol(tool_id);
            parse_quote! { #config_ident }
        }
        other => other.clone(),
    };

    **ty = parse_quote! { Self::Config };

    let expanded = parse_quote! {
        impl #talin::tools::FromConfig for #tool_ty {
            type Config = #config_ty;

            #fn_from_config
        }
    };
    Ok(expanded)
}
