use heck::ToSnakeCase;
use indoc::indoc;
use proc_macro2_diagnostics::{Diagnostic, Level};
use syn::spanned::Spanned;

pub(crate) fn extract_fields(
    input: syn::Data,
) -> Result<impl IntoIterator<Item = syn::Field>, Diagnostic> {
    match input {
        syn::Data::Struct(data_struct) => match data_struct.fields {
            syn::Fields::Named(fields_named) => Ok(fields_named.named),
            fields => Err(Diagnostic::spanned(
                fields.span(),
                Level::Error,
                "This derive macro can only be applied for structs with named fields",
            )),
        },
        syn::Data::Enum(data_enum) => Err(Diagnostic::spanned(
            data_enum.enum_token.span(),
            Level::Error,
            "This derive macro can only be applied for structs",
        )),
        syn::Data::Union(data_union) => Err(Diagnostic::spanned(
            data_union.union_token.span(),
            Level::Error,
            "This derive macro can only be applied for structs",
        )),
    }
}

pub(crate) fn extract_tool_id(ty: &syn::Type) -> Result<String, Diagnostic> {
    let syn::Type::Path(type_path) = ty else {
        return Err(Diagnostic::spanned(
            ty.span(),
            Level::Error,
            "Expected a named struct type in impl block, but found a different type.",
        ));
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Err(Diagnostic::spanned(
            type_path.path.span(),
            Level::Error,
            "Failed to extract struct name: type path has no segments.",
        ));
    };

    let tool_id = segment.ident.to_string();
    let tool_id = tool_id
        .strip_suffix("Tool")
        .unwrap_or(&tool_id)
        .to_snake_case();

    Ok(tool_id)
}

pub(crate) struct ImplItems<'a> {
    pub output_ty: Option<&'a syn::ImplItemType>,
    pub fn_call: &'a syn::ImplItemFn,
    pub fn_name: Option<&'a syn::ImplItemFn>,
    pub fn_description: Option<&'a syn::ImplItemFn>,
    pub fn_available: Option<&'a syn::ImplItemFn>,
    pub fn_from_config: Option<&'a syn::ImplItemFn>,
    pub fn_summarize_input: Option<&'a syn::ImplItemFn>,
    pub fn_summarize_output: Option<&'a syn::ImplItemFn>,
}

fn insert_or_err<T: Spanned>(name: &str, opt: &mut Option<T>, new: T) -> Result<(), Diagnostic> {
    if opt.is_none() {
        *opt = Some(new);
        Ok(())
    } else {
        Err(Diagnostic::spanned(
            new.span(),
            Level::Error,
            format!("Multiple {name} found in the impl block."),
        ))
    }
}

pub(crate) fn extract_items(impl_block: &syn::ItemImpl) -> Result<ImplItems<'_>, Diagnostic> {
    let mut output_ty: Option<&syn::ImplItemType> = None;
    let mut fn_run: Option<&syn::ImplItemFn> = None;
    let mut fn_name: Option<&syn::ImplItemFn> = None;
    let mut fn_description: Option<&syn::ImplItemFn> = None;
    let mut fn_available: Option<&syn::ImplItemFn> = None;
    let mut fn_summarize_input: Option<&syn::ImplItemFn> = None;
    let mut fn_summarize_output: Option<&syn::ImplItemFn> = None;
    let mut fn_from_config: Option<&syn::ImplItemFn> = None;

    for item in &impl_block.items {
        match item {
            syn::ImplItem::Type(ty) if ty.ident == "Output" => {
                insert_or_err("`Output` types", &mut output_ty, ty)?;
            }
            syn::ImplItem::Fn(func) if func.sig.ident == "call" => {
                insert_or_err("`call` function", &mut fn_run, func)?;
            }
            syn::ImplItem::Fn(func) if func.sig.ident == "name" => {
                insert_or_err("`name` function", &mut fn_name, func)?;
            }
            syn::ImplItem::Fn(func) if func.sig.ident == "description" => {
                insert_or_err("`description` function", &mut fn_description, func)?;
            }
            syn::ImplItem::Fn(func) if func.sig.ident == "available" => {
                insert_or_err("`available` function", &mut fn_available, func)?;
            }
            syn::ImplItem::Fn(func) if func.sig.ident == "summarize_input" => {
                insert_or_err("`summarize_input` functions", &mut fn_summarize_input, func)?;
            }
            syn::ImplItem::Fn(func) if func.sig.ident == "summarize_output" => {
                insert_or_err(
                    "`summarize_output` functions",
                    &mut fn_summarize_output,
                    func,
                )?;
            }
            syn::ImplItem::Fn(func) if func.sig.ident == "from_config" => {
                insert_or_err("`from_config` functions", &mut fn_from_config, func)?;
            }
            _ => {
                return Err(Diagnostic::spanned(
                    item.span(),
                    Level::Error,
                    indoc! {"
                        Found unexpected item in the impl block.
                        Expected one of:
                        - `Output` type definition
                        - `call` function
                        - `name` function
                        - `description` function
                        - `available` function
                        - `summarize_input` function
                        - `summarize_output` function
                        - `from_config` function"
                    },
                ));
            }
        }
    }

    let Some(fn_call) = fn_run else {
        return Err(Diagnostic::spanned(
            impl_block.span(),
            Level::Error,
            "Expected a function named 'run' in the impl block, but found none.",
        ));
    };

    Ok(ImplItems {
        output_ty,
        fn_call,
        fn_name,
        fn_description,
        fn_available,
        fn_summarize_input,
        fn_summarize_output,
        fn_from_config,
    })
}
