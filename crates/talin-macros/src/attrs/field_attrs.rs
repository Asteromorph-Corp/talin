use proc_macro2_diagnostics::{Diagnostic, Level};

#[derive(Default)]
pub(crate) struct FieldAttrs {
    pub from: Option<OutputSource>,
    pub into: Option<InputKind>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) enum OutputSource {
    Input(Option<syn::Member>),
    Response,
    #[default]
    ResponseJson,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputKind {
    #[default]
    None,
    Inner,
    Text,
    Placeholder,
    Media,
}

impl FieldAttrs {
    pub(super) fn aggregate(mut acc: Self, attr: &syn::Attribute) -> Result<Self, Diagnostic> {
        if !attr.path().is_ident("chain") {
            return Ok(acc);
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                if acc.from.is_some() {
                    return Err(syn::Error::new_spanned(
                        meta.path,
                        "`from` specified multiple times",
                    ));
                }

                let value = meta.value()?;
                let expr: syn::Expr = value.parse()?;

                match expr {
                    syn::Expr::Path(path) => {
                        if path.path.is_ident("response") {
                            acc.from = Some(OutputSource::Response)
                        }
                        else if path.path.is_ident("response_json") {
                            acc.from = Some(OutputSource::ResponseJson)
                        } else if path.path.is_ident("input") {
                            acc.from = Some(OutputSource::Input(None))
                        } else {

                            return Err(syn::Error::new_spanned(
                                path,
                                "Invalid value for `#[chain(from = ...)]`, expected `input.*`, `response`, or `response_json`",
                            ));
                        }
                    }
                    syn::Expr::Field(field) => {
                        if let syn::Expr::Path(ref base_path) = *field.base && base_path.path.is_ident("input") {
                            acc.from = Some(OutputSource::Input(Some(field.member)));
                        } else {
                            return Err(syn::Error::new_spanned(
                                field,
                                "Invalid value for `#[chain(from = ...)]`, expected `input.*`, `response`, or `response_json`",
                            ));
                        }
                    },
                    _ => return Err(syn::Error::new_spanned(
                        expr,
                        "Invalid value for `#[chain(from = ...)]`, expected `input.*`, `response`, or `response_json`",
                    )),
                }
            } else if meta.path.is_ident("into") {
                if acc.into.is_some() {
                    return Err(syn::Error::new_spanned(
                        meta.path,
                        "`into` specified multiple times",
                    ));
                }

                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                acc.into = match lit.value().as_str() {
                    "inner" => Some(InputKind::Inner),
                    "text" => Some(InputKind::Text),
                    "placeholder" => Some(InputKind::Placeholder),
                    "media" | "images" | "audio" | "video" | "documents" => {
                        Some(InputKind::Media)
                    }
                    _ => return Err(syn::Error::new_spanned(
                        lit,
                        "Invalid value for `#[chain(into = ...)]`, expected `inner`, `text`, `placeholder`, or `media`",
                    )),
                };
            } else {
                return Err(syn::Error::new_spanned(
                    meta.path,
                    "Unknown key in `#[chain(...)]`",
                ));
            }

            Ok(())
        })
        .map_err(|e|
            Diagnostic::spanned(
                e.span(),
                Level::Error,
                format!("Failed to parse chain attribute: {e}"
            )
        ))?;

        Ok(acc)
    }
}
