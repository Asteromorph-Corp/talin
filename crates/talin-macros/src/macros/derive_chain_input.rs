use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{Type, parse_quote};

use crate::ast::extract_fields;
use crate::attrs::{InputKind, extract_field_attr, extract_struct_attr};
use crate::check_type::{
    CONTENT_PART, extract_array_slice_inner_type, extract_cow_array_inner_type,
    extract_option_inner_type, extract_vec_inner_type, is_cow_message_slice_type, is_cow_str_type,
    is_media_type, is_message_slice_type, is_str_type, is_string_type, is_vec_message_type,
};
use crate::macros::crate_path::talin;

struct ChainInputField {
    field: syn::Field,
    into: InputKind,
}

impl TryFrom<syn::Field> for ChainInputField {
    type Error = Diagnostic;

    fn try_from(field: syn::Field) -> Result<Self, Self::Error> {
        let attrs = extract_field_attr(&field.attrs)?;
        let into = attrs.into.unwrap_or_default();
        Ok(Self { field, into })
    }
}

pub(crate) fn derive_chain_input(input: syn::DeriveInput) -> Result<syn::ItemImpl, Diagnostic> {
    let struct_name = &input.ident;
    let fields = extract_fields(input.data)?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let struct_attrs = extract_struct_attr(&input.attrs)?;
    let crate_path = struct_attrs.crate_path.unwrap_or_else(talin);

    let field_specs = fields
        .into_iter()
        .map(ChainInputField::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    let fn_text_replacements = fn_text_replacements(&crate_path, &field_specs);
    let fn_placeholder_replacements = fn_placeholder_replacements(&crate_path, &field_specs)?;
    let fn_media_replacements = fn_media_replacements(&crate_path, &field_specs)?;

    let expanded = parse_quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::chain::ChainInput for #struct_name #ty_generics
        #where_clause
        {
            #fn_text_replacements
            #fn_placeholder_replacements
            #fn_media_replacements
        }
    };
    Ok(expanded)
}

fn fn_text_replacements(crate_path: &syn::Path, fields: &[ChainInputField]) -> syn::ItemFn {
    let text_fields = fields.iter().filter(|f| f.into == InputKind::Text);
    let inner_fields = fields.iter().filter(|f| f.into == InputKind::Inner);

    let entries = text_fields.map(|f| generate_text_replacement(&f.field));
    let extensions = inner_fields.map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.text_replacements()); }
    });

    parse_quote! {
        fn text_replacements(&self) -> #crate_path::chain::TextReplacements {
            let mut map = std::collections::HashMap::from([
                #(#entries),*
            ]);
            #(#extensions)*;
            map
        }
    }
}

fn fn_placeholder_replacements(
    crate_path: &syn::Path,
    fields: &[ChainInputField],
) -> Result<syn::ItemFn, Diagnostic> {
    let placeholder_fields = fields.iter().filter(|f| f.into == InputKind::Placeholder);
    let inner_fields = fields.iter().filter(|f| f.into == InputKind::Inner);

    let entries = placeholder_fields
        .map(|f| generate_placeholder_replacement(&f.field))
        .collect::<Result<Vec<_>, _>>()?;
    let extensions = inner_fields.map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.placeholder_replacements()); }
    });

    Ok(parse_quote! {
        fn placeholder_replacements(&self) -> #crate_path::chain::PlaceholderReplacements {
            let mut map = std::collections::HashMap::from([
                #(#entries),*
            ]);
            #(#extensions)*;
            map
        }
    })
}

fn fn_media_replacements(
    crate_path: &syn::Path,
    fields: &[ChainInputField],
) -> Result<syn::ItemFn, Diagnostic> {
    let media_fields = fields.iter().filter(|f| f.into == InputKind::Media);
    let inner_fields = fields.iter().filter(|f| f.into == InputKind::Inner);

    let entries = media_fields
        .map(|f| generate_media_replacement(crate_path, &f.field))
        .collect::<Result<Vec<_>, _>>()?;
    let extensions = inner_fields.map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.media_replacements()); }
    });

    Ok(parse_quote! {
        fn media_replacements(&self) -> #crate_path::chain::MediaReplacements {
            let mut map = #crate_path::chain::MediaReplacements::from([
                #(#entries),*
            ]);
            #(#extensions)*;
            map
        }
    })
}

fn generate_text_replacement(field: &syn::Field) -> syn::Expr {
    let key = &field.ident;
    let value = generate_text_replacement_conversion(field);

    parse_quote! { (stringify!(#key), #value) }
}

fn generate_placeholder_replacement(
    field: &syn::Field,
) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let key = &field.ident;
    let value = generate_placeholder_replacement_conversion(field)?;

    Ok(quote! { (stringify!(#key), #value) })
}

fn generate_media_replacement(
    crate_path: &syn::Path,
    field: &syn::Field,
) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let key = &field.ident;
    let value = generate_media_replacement_conversion(crate_path, field)?;

    Ok(quote! { (stringify!(#key), #value) })
}

fn generate_text_replacement_conversion(field: &syn::Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();

    if let Some(ty) = extract_option_inner_type(&field.ty) {
        if is_str_type(ty) || is_string_type(ty) || is_cow_str_type(ty) {
            let maybe_as_deref = (!is_str_type(ty)).then_some(quote! { .as_deref() });
            return quote! { std::borrow::Cow::Borrowed(self.#ident #maybe_as_deref .unwrap_or("")) };
        }
        if let Some(inner_ty) = extract_vec_inner_type(ty)
            .or_else(|| extract_cow_array_inner_type(ty))
            .or_else(|| extract_array_slice_inner_type(ty))
        {
            let maybe_as_deref = extract_array_slice_inner_type(ty).map(|_| quote! { .as_deref() });
            let map_to_str = map_to_str(inner_ty);
            return quote! {
                std::borrow::Cow::Owned(
                    self.#ident
                        #maybe_as_deref
                        .unwrap_or(&[])
                        .iter()
                        #map_to_str
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
        }
        return quote! { std::borrow::Cow::Owned(self.#ident.map_or_else(|| String::new(), |s| s.to_string())) };
    }

    if is_str_type(&field.ty) {
        let maybe_as_ref = (!is_str_type(&field.ty)).then_some(quote! { .as_ref() });
        return quote! { std::borrow::Cow::Borrowed(self.#ident #maybe_as_ref) };
    }
    if let Some(inner_ty) = extract_vec_inner_type(&field.ty)
        .or_else(|| extract_cow_array_inner_type(&field.ty))
        .or_else(|| extract_array_slice_inner_type(&field.ty))
    {
        let map_to_str = map_to_str(inner_ty);
        return quote! {
            std::borrow::Cow::Owned(
                self.#ident
                    .iter()
                    #map_to_str
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
    }
    quote! { std::borrow::Cow::Owned(self.#ident.to_string()) }
}

fn generate_placeholder_replacement_conversion(
    field: &syn::Field,
) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let ident = field.ident.as_ref().unwrap();

    if let Some(ty) = extract_option_inner_type(&field.ty) {
        if is_message_slice_type(ty) {
            return Ok(quote! { std::borrow::Cow::Borrowed(self.#ident.unwrap_or(&[])) });
        } else if is_vec_message_type(ty) || is_cow_message_slice_type(ty) {
            return Ok(
                quote! { std::borrow::Cow::Borrowed(self.#ident.as_deref().unwrap_or(&[])) },
            );
        } else {
            return Err(unsupported_placeholder(ty));
        }
    }

    if is_message_slice_type(&field.ty) {
        Ok(quote! { std::borrow::Cow::Borrowed(self.#ident) })
    } else if is_vec_message_type(&field.ty) || is_cow_str_type(&field.ty) {
        Ok(quote! { std::borrow::Cow::Borrowed(&self.#ident) })
    } else if is_cow_message_slice_type(&field.ty) {
        Ok(quote! { std::borrow::Cow::Borrowed(self.#ident.as_ref()) })
    } else {
        Err(unsupported_placeholder(&field.ty))
    }
}

fn unsupported_placeholder(ty: &syn::Type) -> Diagnostic {
    Diagnostic::spanned(
        ty.span(),
        Level::Error,
        format!(
            "Unsupported type for placeholder replacement: {}",
            ty.to_token_stream()
        ),
    )
}

fn unsupported_media(ty: &syn::Type) -> Diagnostic {
    Diagnostic::spanned(
        ty.span(),
        Level::Error,
        format!("Unsupported type for media: {}", ty.to_token_stream()),
    )
}

fn generate_media_replacement_conversion(
    crate_path: &syn::Path,
    field: &syn::Field,
) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let ident = field.ident.as_ref().unwrap();

    let (ty, optional) =
        extract_option_inner_type(&field.ty).map_or((&field.ty, false), |ty| (ty, true));
    let elem_ty = extract_array_slice_inner_type(ty)
        .or_else(|| extract_vec_inner_type(ty))
        .or_else(|| extract_cow_array_inner_type(ty))
        .ok_or_else(|| unsupported_media(ty))?;

    let Type::Path(type_path) = elem_ty else {
        return Err(unsupported_media(elem_ty));
    };
    let seg = type_path
        .path
        .segments
        .last()
        .ok_or_else(|| unsupported_media(elem_ty))?;

    if !is_media_type(&seg.ident) {
        return Err(unsupported_media(elem_ty));
    }

    let parts = if optional {
        quote! {
            self.#ident.as_ref()
                .map(AsRef::<[#elem_ty]>::as_ref)
                .unwrap_or(&[])
        }
    } else {
        quote! {
            AsRef::<[#elem_ty]>::as_ref(&self.#ident)
        }
    };

    if seg.ident == CONTENT_PART {
        Ok(quote! {
            std::borrow::Cow::Borrowed(#parts)
        })
    } else {
        Ok(quote! {
            std::borrow::Cow::<[#crate_path::schemas::ContentPart]>::Owned(
                (#parts)
                    .iter()
                    .cloned()
                    .map(#crate_path::schemas::ContentPart::from)
                    .collect()
            )
        })
    }
}

fn map_to_str(ty: &syn::Type) -> proc_macro2::TokenStream {
    if is_str_type(ty) {
        quote! { .map(|s| *s)}
    } else if is_string_type(ty) {
        quote! { .map(|s| s.as_str()) }
    } else if is_cow_str_type(ty) {
        quote! { .map(|s| s.as_ref()) }
    } else {
        quote! { .map(|s| s.to_string()) }
    }
}
