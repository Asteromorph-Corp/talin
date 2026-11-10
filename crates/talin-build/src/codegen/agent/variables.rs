use std::borrow::Cow;
use std::collections::HashSet;

use heck::ToUpperCamelCase;
use quote::format_ident;
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::{Attachments, InputVariable, RuntimeVariable};
use crate::options::{ArtifactOptions, InputFieldOrder};
use crate::ty::{has_lifetime, is_string_like_type, lifetime_a};

pub(super) struct Variables<'a> {
    /// Deduplicated by name, in the configured field order. In appearance order, media
    /// attachments come last.
    vars: Vec<Cow<'a, RuntimeVariable>>,
    /// The idents of the string inputs wrapped in generated newtypes.
    newtyped: Vec<syn::Ident>,
}

impl<'a> Variables<'a> {
    pub(super) fn collect(
        variables: &'a [RuntimeVariable],
        attachments: &Attachments,
        options: &ArtifactOptions,
    ) -> Variables<'a> {
        let talin = talin();
        let lifetime = lifetime_a();

        let media_vars = [
            (&attachments.images, format_ident!("ImageContent")),
            (&attachments.audios, format_ident!("AudioContent")),
            (&attachments.videos, format_ident!("VideoContent")),
            (&attachments.documents, format_ident!("DocumentContent")),
        ]
        .into_iter()
        .flat_map(|(vars, content)| {
            let talin = talin.clone();
            let lifetime = lifetime.clone();
            vars.iter().map(move |var| {
                let ident = format_ident!("{var}");
                let type_ =
                    parse_quote! { std::borrow::Cow<#lifetime, [#talin::schemas::#content]> };
                RuntimeVariable::InputMedia(Box::new(InputVariable { ident, type_ }))
            })
        });

        let mut seen = HashSet::new();
        let mut newtyped = Vec::new();
        let mut vars = Vec::new();
        let borrowed = variables.iter().map(Cow::Borrowed);
        let owned = media_vars.map(Cow::Owned);
        for var in borrowed.chain(owned) {
            if seen.contains(&var.name()) {
                continue;
            }
            seen.insert(var.name());

            if options.use_newtype_inputs
                && let RuntimeVariable::Input(input) = var.as_ref()
                && is_string_like_type(&input.type_)
            {
                let ident: syn::Ident = input.ident.clone();
                let type_ = newtype_ident(&ident);
                let type_: syn::Type = parse_quote! { #type_ };

                newtyped.push(input.ident.clone());
                vars.push(Cow::Owned(RuntimeVariable::Input(Box::new(
                    InputVariable { ident, type_ },
                ))));
            } else {
                vars.push(var);
            }
        }

        if options.input_field_order == InputFieldOrder::Alphabetical {
            vars.sort_by_key(|var| var.name());
        }

        Variables { vars, newtyped }
    }

    /// The newtypes wrapping the string inputs, in no particular order.
    pub(super) fn newtypes(&self) -> Vec<syn::Ident> {
        self.newtyped.iter().map(newtype_ident).collect()
    }

    pub(super) fn is_newtyped(&self, ident: &syn::Ident) -> bool {
        self.newtyped.contains(ident)
    }

    pub(super) fn all(&self) -> Vec<&RuntimeVariable> {
        self.vars.iter().map(AsRef::as_ref).collect()
    }

    pub(super) fn inputs(&self) -> Vec<&InputVariable> {
        self.all()
            .into_iter()
            .filter_map(|var| match var {
                RuntimeVariable::Input(var) => Some(var.as_ref()),
                RuntimeVariable::InputMedia(var) => Some(var.as_ref()),
                _ => None,
            })
            .collect()
    }

    pub(super) fn has_lifetime(&self) -> bool {
        self.all().into_iter().any(|var| {
            let type_ = var.get_type();
            has_lifetime(&type_) || matches!(type_, syn::Type::Reference(_))
        })
    }
}

/// The newtype wrapping the string input `ident`, e.g. `Language` for `language`.
fn newtype_ident(ident: &syn::Ident) -> syn::Ident {
    format_ident!("{}", ident.to_string().to_upper_camel_case())
}
