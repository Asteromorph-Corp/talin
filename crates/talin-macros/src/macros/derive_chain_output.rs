use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::{ToTokens, format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_quote, parse_quote_spanned};

use crate::ast::extract_fields;
use crate::attrs::{OutputSource, extract_field_attr, extract_struct_attr, filter_attrs};
use crate::check_type::{is_cow_str_type, is_string_like_type};
use crate::macros::crate_path::talin;

const MULTIPLE_RESPONSE_ERR: &str = "ChainOutput struct cannot have more than one field with `#[chain(from = \"response\")]` attribute";
const INVALID_RESPONSE_ERR: &str = "ChainOutput struct cannot have both a field with `#[chain(from = \"response\")] attribute and fields with `#[chain(from = \"response_json\")] attribute";
const MISSING_INPUT_ERR: &str = "ChainOutput struct must have a `#[chain(from_input = \"...\")]` attribute when it has fields with `#[chain(from = \"input\")]` attribute";

struct ChainOutputField {
    field: syn::Field,
    from: OutputSource,
}

impl TryFrom<syn::Field> for ChainOutputField {
    type Error = Diagnostic;

    fn try_from(field: syn::Field) -> Result<Self, Self::Error> {
        let attrs = extract_field_attr(&field.attrs)?;
        let from = attrs.from.unwrap_or_default();
        Ok(Self { field, from })
    }
}

enum PartialFields<'a> {
    Response(&'a syn::Field),
    ResponseJson(Vec<&'a syn::Field>),
    None,
}

pub(crate) fn derive_chain_output(
    input: syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let struct_attrs = extract_struct_attr(&input.attrs)?;
    let crate_path = struct_attrs.crate_path.unwrap_or_else(talin);
    let from_input = struct_attrs.from_input;

    let fields = extract_fields(input.data)?
        .into_iter()
        .map(ChainOutputField::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let partial_fields = validate_fields(&fields, from_input.as_ref())?;

    let partial_struct_def =
        partial_struct(&crate_path, &input.ident, &input.attrs, partial_fields);

    let chain_output_impls = match &from_input {
        Some(input_ty) => {
            let chain_output_impl =
                impl_chain_output(&crate_path, &input.ident, &input.generics, true);
            let [partial_output_impl, with_input_impl] = impls_for_partial(
                &crate_path,
                &input.ident,
                &input.generics,
                &fields,
                input_ty,
            )?;
            vec![chain_output_impl, partial_output_impl, with_input_impl]
        }
        None => {
            let chain_output_impl =
                impl_chain_output(&crate_path, &input.ident, &input.generics, false);
            let partial_output_impl =
                impl_for_full(&crate_path, &input.ident, &input.generics, &fields)?;
            vec![chain_output_impl, partial_output_impl]
        }
    };

    let json_schema_impl = struct_attrs
        .derive_schema
        .then(|| impl_json_schema(&crate_path, &input.ident, &input.generics))
        .transpose()?;

    let expanded = quote! {
        #partial_struct_def
        #(#chain_output_impls)*
        #json_schema_impl
    };
    Ok(expanded)
}

fn validate_fields<'a>(
    fields: &'a [ChainOutputField],
    from_input: Option<&syn::Type>,
) -> Result<PartialFields<'a>, Diagnostic> {
    let mut input_field = None;
    let mut response_json_fields = Vec::new();
    let mut response_field = None;

    for field in fields {
        match field.from {
            OutputSource::Input(_) => input_field = input_field.or(Some(&field.field)),
            OutputSource::Response if !is_string_like_type(&field.field.ty) => {
                if response_field.is_some() {
                    return Err(Diagnostic::spanned(
                        field.field.span(),
                        Level::Error,
                        MULTIPLE_RESPONSE_ERR,
                    ));
                }
                response_field = Some(&field.field);
            }
            OutputSource::ResponseJson => {
                response_json_fields.push(&field.field);
            }
            _ => {}
        }
    }

    if let Some(input_field) = input_field
        && from_input.is_none()
    {
        return Err(Diagnostic::spanned(
            input_field.span(),
            Level::Error,
            MISSING_INPUT_ERR,
        ));
    }

    match response_field {
        Some(_) if !response_json_fields.is_empty() => Err(Diagnostic::spanned(
            response_field.span(),
            Level::Error,
            INVALID_RESPONSE_ERR,
        )),
        Some(field) => Ok(PartialFields::Response(field)),
        None if !response_json_fields.is_empty() => {
            Ok(PartialFields::ResponseJson(response_json_fields))
        }
        None => Ok(PartialFields::None),
    }
}

fn get_partial_struct_name(struct_name: &syn::Ident) -> syn::Ident {
    format_ident!("{struct_name}Partial")
}

/// Generates the helper struct used for deserializing fields from response JSON.
fn partial_struct(
    crate_path: &syn::Path,
    struct_name: &syn::Ident,
    struct_attrs: &[syn::Attribute],
    fields: PartialFields<'_>,
) -> syn::ItemStruct {
    let serde_path: syn::Path = parse_quote!(#crate_path::__private::serde);
    let serde_path_str = serde_path.to_token_stream().to_string().replace(" ", "");
    let schemars_path: syn::Path = parse_quote!(#crate_path::__private::schemars);
    let schemars_path_str = schemars_path.to_token_stream().to_string().replace(" ", "");

    let struct_name = get_partial_struct_name(struct_name);
    let struct_attrs = filter_attrs(struct_attrs, &["serde", "schemars", "doc"]);

    let helper_fields = match fields {
        PartialFields::Response(f) => {
            let attrs = filter_attrs(&f.attrs, &["serde", "schemars", "doc"]);
            let ident = &f.ident;
            let ty = if is_cow_str_type(&f.ty) {
                &parse_quote_spanned! { f.ty.span() => String }
            } else {
                &f.ty
            };
            vec![parse_quote! {
                #[serde(flatten)]
                #(#attrs)*
                pub #ident: #ty
            }]
        }
        PartialFields::ResponseJson(fields) => fields
            .iter()
            .map(|f| -> syn::Field {
                let attrs = filter_attrs(&f.attrs, &["serde", "schemars", "doc"]);
                let ident = &f.ident;
                let ty = if is_cow_str_type(&f.ty) {
                    &parse_quote_spanned! { f.ty.span() => String }
                } else {
                    &f.ty
                };

                parse_quote! {
                    #(#attrs)*
                    pub #ident: #ty
                }
            })
            .collect::<Vec<_>>(),
        PartialFields::None => vec![],
    };

    parse_quote! {
        #[derive(#serde_path::Deserialize, #schemars_path::JsonSchema)]
        #[serde(crate = #serde_path_str)]
        #[schemars(crate = #schemars_path_str)]
        #(#struct_attrs)*
        pub struct #struct_name {
            #[serde(skip, default)]
            pub raw: String,
            #(#helper_fields),*
        }
    }
}

fn impl_chain_output(
    crate_path: &syn::Path,
    struct_name: &syn::Ident,
    struct_generics: &syn::Generics,
    use_input: bool,
) -> syn::ItemImpl {
    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();
    let partial_output_ty = if !use_input {
        struct_name
    } else {
        &get_partial_struct_name(struct_name)
    };

    parse_quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::chain::ChainOutput for #struct_name #ty_generics
        #where_clause
        {
            type Partial = #partial_output_ty;
        }
    }
}

fn impls_for_partial(
    crate_path: &syn::Path,
    struct_name: &syn::Ident,
    struct_generics: &syn::Generics,
    fields: &[ChainOutputField],
    from_input: &syn::Type,
) -> Result<[syn::ItemImpl; 2], Diagnostic> {
    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();
    let partial_struct_name = get_partial_struct_name(struct_name);

    let deserialize_partial = deserialize_partial(crate_path, struct_name);
    let initialize_full = initialize_full(struct_name, fields);

    let impl_chain_output: syn::ItemImpl = parse_quote! {
        #[automatically_derived]
        impl #crate_path::chain::PartialOutput for #partial_struct_name {
            fn from_text(text: &str) -> Result<Self, #crate_path::__private::ParseError> {
                #deserialize_partial
                Ok(partial)
            }
        }
    };
    let impl_partial_with_input: syn::ItemImpl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::chain::PartialWithInput<
            #from_input,
            #struct_name #ty_generics
        > for #partial_struct_name
        #where_clause
        {
            fn with_input(self, input: #from_input) -> #struct_name #ty_generics {
                let partial = self;
                #initialize_full
            }
        }
    };

    Ok([impl_chain_output, impl_partial_with_input])
}

fn impl_for_full(
    crate_path: &syn::Path,
    struct_name: &syn::Ident,
    struct_generics: &syn::Generics,
    fields: &[ChainOutputField],
) -> Result<syn::ItemImpl, Diagnostic> {
    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();
    let deserialize_partial = deserialize_partial(crate_path, struct_name);
    let initialize_full = initialize_full(struct_name, fields);

    let impl_chain_output = parse_quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::chain::PartialOutput for #struct_name #ty_generics
        #where_clause
        {
            fn from_text(text: &str) -> Result<Self, #crate_path::__private::ParseError> {
                #deserialize_partial
                let full = #initialize_full;
                Ok(full)
            }
        }
    };

    Ok(impl_chain_output)
}

fn impl_json_schema(
    crate_path: &syn::Path,
    struct_name: &syn::Ident,
    struct_generics: &syn::Generics,
) -> Result<syn::ItemImpl, Diagnostic> {
    let schemars_path: syn::Path = parse_quote!(#crate_path::__private::schemars);

    let partial_struct_name = get_partial_struct_name(struct_name);
    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();

    let expanded = parse_quote! {
        #[automatically_derived]
        impl #impl_generics #schemars_path::JsonSchema for #struct_name #ty_generics
        #where_clause
        {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(stringify!(#struct_name))
            }

            fn json_schema(
                generator: &mut #schemars_path::generate::SchemaGenerator,
            ) -> #schemars_path::Schema {
                let mut schema = #partial_struct_name::json_schema(generator);
                // Providers reject object schemas without `properties`, which is
                // what schemars emits for structs with no fields.
                if let Some(object) = schema.as_object_mut()
                    && object.get("type").and_then(|t| t.as_str()) == Some("object")
                    && !object.contains_key("properties")
                {
                    object.insert(
                        "properties".to_owned(),
                        #crate_path::__private::serde_json::json!({}),
                    );
                }
                schema
            }
        }
    };
    Ok(expanded)
}

fn deserialize_partial(
    crate_path: &syn::Path,
    struct_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let partial_struct_name = get_partial_struct_name(struct_name);
    quote! {
        let jsons = #crate_path::__private::extract_jsons(text)
            .into_iter()
            .filter_map(|text| #crate_path::__private::parse_json::<#partial_struct_name>(text).ok())
            .collect::<Vec<_>>();

        if jsons.len() > 1 {
            #crate_path::__private::log::warn!("Multiple JSON objects found in response text; using the last one matching the schema.");
        }
        let Some(mut partial) = jsons.into_iter().rev().next() else {
            let e = <#crate_path::__private::serde_json::Error as #crate_path::__private::serde::de::Error>::custom("No JSON object found matching the schema.");
            return Err(#crate_path::__private::ParseError::new(text, e));
        };
        partial.raw = text.to_owned();
    }
}

fn initialize_full(
    struct_name: &syn::Ident,
    fields: &[ChainOutputField],
) -> proc_macro2::TokenStream {
    let field_initializers = fields.iter().map(|f| -> syn::FieldValue {
        let ident = f.field.ident.as_ref().unwrap();
        match &f.from {
            OutputSource::Input(expr) => match expr {
                Some(mem) => parse_quote! { #ident: input.#mem.into() },
                None => parse_quote! { #ident: input.#ident.into() },
            },
            OutputSource::ResponseJson => parse_quote! { #ident: partial.#ident.into() },
            OutputSource::Response if is_string_like_type(&f.field.ty) => {
                parse_quote! { #ident: partial.raw.into() }
            }
            OutputSource::Response => parse_quote! { #ident: partial.#ident },
        }
    });

    quote! {
        #struct_name {
            #(#field_initializers,)*
        }
    }
}
