use quote::{ToTokens, format_ident};
use syn::parse_quote;
use syn::visit_mut::VisitMut;

use crate::config::StructDef;

/// Builds a plain struct from the definition.
pub(crate) fn struct_definition(symbol: &syn::Ident, struct_: &StructDef) -> syn::ItemStruct {
    build_struct(symbol, struct_, false)
}

/// Builds a struct whose schema is sent to providers.
///
/// Providers reject some schema keywords, e.g. the `minimum`/`maximum` that `schemars` emits for
/// `u8` or `u32`. Fields whose type contains such an integer get
/// `#[schemars(with = "...")]` with the integer replaced by `i32`; (de)serialization is unchanged.
pub(crate) fn schema_struct_definition(
    symbol: &syn::Ident,
    struct_: &StructDef,
) -> syn::ItemStruct {
    build_struct(symbol, struct_, true)
}

struct Widen(bool);

/// Integer types that `schemars` emits range bounds for.
const BOUNDED_INTS: [&str; 10] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i64", "i128",
];

impl syn::visit_mut::VisitMut for Widen {
    fn visit_type_path_mut(&mut self, path: &mut syn::TypePath) {
        if path.qself.is_none()
            && let Some(ident) = path.path.get_ident()
            && BOUNDED_INTS.iter().any(|int| ident == int)
        {
            path.path = syn::parse_quote!(i32);
            self.0 = true;
            return;
        }
        syn::visit_mut::visit_type_path_mut(self, path);
    }
}

fn build_struct(symbol: &syn::Ident, struct_: &StructDef, for_schema: bool) -> syn::ItemStruct {
    let fields = struct_.0.iter().map(|(name, field)| -> syn::Field {
        let field_name = format_ident!("{name}");
        let field_type = &field.type_;
        let doc_attr: Option<syn::Attribute> = field
            .description
            .as_deref()
            .map(|desc| parse_quote! { #[doc = #desc] });

        let with_attr: Option<syn::Attribute> = for_schema
            .then(|| {
                let mut widen = Widen(false);
                let mut ty = field.type_.clone();
                widen.visit_type_mut(&mut ty);
                widen.0.then(|| {
                    let widened = ty.to_token_stream().to_string().replace(' ', "");
                    parse_quote! { #[schemars(with = #widened)] }
                })
            })
            .flatten();

        parse_quote! {
            #doc_attr
            #with_attr
            pub #field_name: #field_type
        }
    });

    parse_quote! {
        pub struct #symbol {
            #(#fields,)*
        }
    }
}
