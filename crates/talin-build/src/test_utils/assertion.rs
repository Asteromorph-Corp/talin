use std::fs;
use std::path::PathBuf;

use quote::ToTokens;
use syn::parse_quote;

use crate::test_utils::normalize_string::normalize_string;

fn get_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

/// Formats a file into the canonical, readable Rust used for fixture comparisons.
///
/// `prettyplease` alone is not canonical: token-level noise that doesn't affect meaning —
/// trailing commas the fixture carries but `quote!`-generated code doesn't, and
/// punctuation spacing such as `> ::` vs `>::` — leaks into its output. We strip the
/// trailing commas, then re-parse and re-print: re-parsing discards insignificant
/// whitespace and spacing, so two equivalent inputs converge on identical output, while
/// the result stays readable, multi-line Rust (a real mismatch is a normal diff, not one
/// collapsed line).
fn prettify_file(file: &syn::File) -> String {
    let printed = prettyplease::unparse(file);
    let normalized = normalize_string(&printed);
    let reparsed: syn::File =
        syn::parse_str(&normalized).expect("normalized output should re-parse as Rust");
    prettyplease::unparse(&reparsed)
}

fn prettify_item(item: &impl ToTokens) -> String {
    let file: syn::File = syn::parse2(item.to_token_stream()).expect("item didn’t parse");
    prettify_file(&file)
}

fn prettify_items_in_trait(items: &[impl ToTokens]) -> String {
    let file: syn::File = parse_quote! {
        pub trait DummyTrait {
            #(#items)*
        }
    };
    prettify_file(&file)
}

/// Wraps a bare expression in a dummy function body so `prettyplease` (which only formats
/// complete files) can render it.
fn prettify_expr(expr: &impl ToTokens) -> String {
    let file: syn::File = parse_quote! {
        fn __dummy() {
            let _ = #expr;
        }
    };
    prettify_file(&file)
}

fn load_fixture(fixture_path: &str) -> String {
    let src = fs::read_to_string(get_fixture_path(fixture_path)).expect("fixture file missing");
    let file: syn::File = syn::parse_str(&src).expect("fixture didn’t parse");
    prettify_file(&file)
}

pub(crate) fn load_fixture_in_trait(fixture_path: &str) -> String {
    let src = fs::read_to_string(get_fixture_path(fixture_path)).expect("fixture file missing");
    let items: proc_macro2::TokenStream = src.parse().expect("fixture didn’t parse");
    let file: syn::File = parse_quote! {
        pub trait DummyTrait {
            #items
        }
    };
    prettify_file(&file)
}

pub(crate) fn assert_item_eq(item: &impl quote::ToTokens, fixture_path: &str) {
    let pretty = prettify_item(item);
    let expected = load_fixture(fixture_path);
    pretty_assertions::assert_eq!(pretty, expected);
}

/// Test helper to compare a generated expression against a fixture.
///
/// Since `prettyplease` can only format complete files, the expression is wrapped in a
/// dummy function body on both sides; the fixture holds only the bare expression.
pub(crate) fn assert_expr_eq(expr: &impl quote::ToTokens, fixture_path: &str) {
    let pretty = prettify_expr(expr);
    let expected = load_fixture(fixture_path);
    pretty_assertions::assert_eq!(pretty, expected);
}

/// Test helper to compare a list of items in a trait context.
///
/// Since `prettyplease` can only format complete files, we wrap the items in a dummy trait.
pub(crate) fn assert_items_eq_in_trait(items: &[impl ToTokens], fixture_path: &str) {
    let pretty = prettify_items_in_trait(items);
    let expected = load_fixture_in_trait(fixture_path);
    pretty_assertions::assert_eq!(pretty, expected);
}
