//! Procedural macro implementations for `talin`.

use ast::extract_tool_id;
use proc_macro::TokenStream;
use proc_macro2_diagnostics::Diagnostic;
use quote::ToTokens;
use syn::{DeriveInput, ItemImpl, parse_macro_input};

mod ast;
mod attrs;
mod check_type;
mod macros;

/// A helper macro for implementing a custom tool.
///
/// This macro simplifies the creation of tools by automatically deriving necessary traits and
/// generating JSON schemas for tool inputs.
///
/// # Example
///
/// ```ignore
/// #[tool]
/// impl AdditionTool {
///     type Output = i64; // Optional, default `String`
///
///     // `Self::Input` is generated from toml definition
///     async fn run(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error>> {
///         Ok(input.num1 + input.num2)
///     }
///
///     // Optional: override the `name` / `description` from the toml definition, e.g. to
///     // reflect the tool's configuration.
///     // The toml values, if present, are available as `TOOL_NAME` and `TOOL_DESCRIPTION`.
///     fn description(&self) -> String {
///         format!("{} (searches {})", TOOL_DESCRIPTION, self.domain)
///     }
///
///     // Optional: if this returns false, the tool is left out of requests and calls to it are rejected.
///     // Default: always available.
///     fn available(&self, transcript: &[EntryView<'_>]) -> bool {
///         count_tool_calls(transcript, "addition") < 5
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return item;
    }

    let impl_block = parse_macro_input!(item as ItemImpl);

    macros::tool(impl_block)
        .map(|item| item.to_token_stream())
        .unwrap_or_else(Diagnostic::emit_as_item_tokens)
        .into()
}

/// Derive macro generating an implementation of the `ChainInput` trait.
///
/// `ChainInput` defines how the fields of a struct are converted into a chain's input entries.
/// Each entry is a key-value pair, with the field name serving as the key and the field value as
/// the corresponding value.
///
/// Fields annotated with `#[chain(into = ...)]` attribute contribute to the generated chain input.
/// The `into` option determines how each field is interpreted:
///
/// - `inner` — Treats the field as another `ChainInput` and merges its entries into those of the
///   containing struct.
/// - `text` — Uses the field as a text substitution value for placeholders in the input template,
///   such as `{{key}}`.
/// - `placeholder` — Uses the field to replace a message placeholder in the input template. This is
///   intended for fields that evaluate to a message or a list of messages.
/// - `media` — Uses the field as media input.
///
/// Fields without a `#[chain(into = ...)]` annotation do not contribute input entries.
#[proc_macro_derive(ChainInput, attributes(chain, serde))]
pub fn derive_chain_input(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    macros::derive_chain_input(input)
        .map(|item| item.to_token_stream())
        .unwrap_or_else(Diagnostic::emit_as_item_tokens)
        .into()
}

/// Derive macro generating an implementation of the `ChainOutput` trait.
///
/// `ChainOutput` defines how the fields of a struct are populated from a chain's output.
///
/// Fields may be annotated with `#[chain(from = ...)]` attribute to specify their source.
/// The `from` option determines how each field is populated:
///
/// - `input` — Populate the field from the corresponding field of the chain input, matched by field
///   name. If any field uses `from = "input"`, the struct must be also have the `#[chain(from_input
///   = <Input>)]` to specify the input type.
/// - `response_json` — Populate the field from the chain's JSON output, using the field name as the
///   key in the response object.
/// - `response` — Populate the field from the raw response. The field is expected to be a
///   `String`-like type.
///
/// Fields without a `#[chain(from = ...)]` annotation default to `response_json`.
#[proc_macro_derive(ChainOutput, attributes(chain, serde, schemars))]
pub fn derive_chain_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    macros::derive_chain_output(input)
        .map(|item| item.to_token_stream())
        .unwrap_or_else(Diagnostic::emit_as_item_tokens)
        .into()
}

/// Derive macro generating the constructor type and an implementation of the `Ctor` trait.
///
/// This macro generates a new unit struct that acts as "constructor" for the annotated type. The
/// struct implements `Ctor` with `Target` set to the annotated type. For example:
///
/// ```rust,ignore
/// #[derive(Ctor)]
/// pub struct MyType<'a, 'b> {
///     // ...
/// }
/// ```
///
/// expands to:
/// ```rust,ignore
/// pub struct MyTypeCtor;
/// impl Ctor for MyTypeCtor {
///     type Target<'a> = MyType<'a, 'a>;
/// }
/// ```
///
/// This lets consumers to abstract over an entire family of types (such as `Input<'input>` or
/// `Output<'input>`), rather than over a single instantiation of those types.
/// In particular, it allows input and output types to be related generically across all `'input`
/// lifetimes.
/// This is used by types such as `Agent<I, O>` to define methods of the form `run(<'input>(& self,
/// input: I::Target<'input>) -> O::Target<'input>`.
#[proc_macro_derive(Ctor, attributes(chain))]
pub fn derive_ctor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    macros::derive_ctor(input)
        .map(|item| item.to_token_stream())
        .unwrap_or_else(Diagnostic::emit_as_item_tokens)
        .into()
}

/// Derive macro generating an implementation of the `ConfigSchema` trait for a tool config type.
///
/// The schema tells the code generated for an agent which fields of the config must be supplied
/// in `agents.toml`; a missing required field is a compile error.
#[proc_macro_derive(ConfigSchema)]
pub fn derive_config_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    macros::derive_config_schema(input)
        .map(|item| item.to_token_stream())
        .unwrap_or_else(Diagnostic::emit_as_item_tokens)
        .into()
}
