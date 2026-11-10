use std::collections::BTreeMap;

use proc_macro2::Span;
use quote::format_ident;
use syn::parse_quote;

use crate::config::{AgentLocalToolConfig, AgentLocalToolConfigNested};

/// Generate the block expression that constructs and populates an initializer
/// struct instance from a TOML value.
///
/// The returned expression looks like:
/// ```rust,ignore
/// {
///     let mut __x = InitSymbol::default();
///     talin::__private::navigate!(__x.field).set(value);
///     // …more statements…
///     __x
/// }
/// ```
///
/// The `navigate!` macro and helper traits (`VecPushElem`, `MapInsertElem`,
/// `SetOption`, `PeelOption`) must be in scope at the call site — they are
/// re-exported through the talin crate's `__private` module and the
/// generated file's `use super::*;` import brings them into scope.
pub(super) fn tool_init_params(
    init_symbol: &syn::Type,
    params: &AgentLocalToolConfig,
) -> syn::Expr {
    let root_var: syn::Ident = format_ident!("__x");
    let mut counter: usize = 0;

    let stmts = Visitor::owned(&root_var, &mut counter).visit(params);

    if stmts.is_empty() {
        parse_quote! { #init_symbol::default() }
    } else {
        parse_quote! {
            {
                let mut #root_var = #init_symbol::default();
                #(#stmts)*
                #root_var
            }
        }
    }
}

// ── navigate! macro call builder ──────────────────────────────────────────────

/// A navigation root for emitting `navigate!` macro calls.
///
/// - `Owned` — the root is a plain `let mut __x` binding.
/// - `Deref` — the root is a `&mut T` (e.g. `__e` from `push_elem` / `insert_elem`), so we emit
///   `*__e` at the start of the macro call.
enum NavRoot<'a> {
    Owned(&'a syn::Ident),
    Deref(&'a syn::Ident),
}

// ── Core recursive codegen ────────────────────────────────────────────────────

/// Shared mutable state threaded through the recursive visitor.
struct Visitor<'a> {
    root: NavRoot<'a>,
    path: Vec<syn::Member>,
    /// Monotonically-increasing counter for fresh temporary variable names.
    counter: &'a mut usize,
}

impl<'a> Visitor<'a> {
    fn owned(root: &'a syn::Ident, counter: &'a mut usize) -> Self {
        Self {
            root: NavRoot::Owned(root),
            path: Vec::new(),
            counter,
        }
    }

    fn deref(root: &'a syn::Ident, counter: &'a mut usize) -> Self {
        Self {
            root: NavRoot::Deref(root),
            path: Vec::new(),
            counter,
        }
    }

    fn fresh_var(&mut self, prefix: &str) -> syn::Ident {
        let n = *self.counter;
        *self.counter += 1;
        format_ident!("__{prefix}{n}")
    }

    /// Build the `navigate!(…)` invocation for `root + path` as a `syn::Expr`.
    fn navigate(&self) -> syn::Expr {
        let root: syn::Expr = match self.root {
            NavRoot::Owned(r) => parse_quote!(#r),
            NavRoot::Deref(r) => parse_quote!(*#r),
        };
        let path = &self.path;

        if path.is_empty() {
            parse_quote! { talin::__private::navigate!(#root) }
        } else {
            parse_quote! { talin::__private::navigate!(#root . #(#path).*) }
        }
    }

    fn visit(&mut self, value: &AgentLocalToolConfig) -> Vec<syn::Stmt> {
        let nav = self.navigate();

        match value {
            // ── Scalar leaf ───────────────────────────────────────────────────────
            AgentLocalToolConfig::String(s) => vec![parse_quote! { #nav.set_from(#s); }],
            AgentLocalToolConfig::Integer(i) => {
                let i = syn::LitInt::new(&i.to_string(), Span::call_site());
                vec![parse_quote! { #nav.set(#i); }]
            }
            AgentLocalToolConfig::Float(f) => {
                let f = syn::LitFloat::new(&f.to_string(), Span::call_site());
                vec![parse_quote! { #nav.set(#f); }]
            }
            AgentLocalToolConfig::Boolean(b) => vec![parse_quote! { #nav.set_from(#b); }],
            AgentLocalToolConfig::Env(env) => vec![parse_quote! { #nav.set_from_env(#env); }],
            // ── Nested fields ─────────────────────────────────────────────────────
            AgentLocalToolConfig::Array(arr) => vec![self.visit_array(arr)],
            AgentLocalToolConfig::EnumVariant {
                enum_,
                variant,
                fields,
            } => self.visit_enum(enum_, variant, fields),
            AgentLocalToolConfig::Nested(AgentLocalToolConfigNested::Struct(fields)) => {
                self.visit_struct(fields)
            }
            AgentLocalToolConfig::Nested(AgentLocalToolConfigNested::Tuple(fields)) => {
                self.visit_tuple(fields)
            }
        }
    }

    /// Emit a `{ let mut __vN = navigate!(…); { let __eM = __vN.push_elem(); … } … }` block.
    fn visit_array(&mut self, arr: &[AgentLocalToolConfig]) -> syn::Stmt {
        let nav: syn::Expr = self.navigate();
        let vec_var = self.fresh_var("v");

        let inner_stmts = arr.iter().map(|elem| -> syn::Expr {
            let elem_var = self.fresh_var("e");
            let elem_stmts = Visitor::deref(&elem_var, self.counter).visit(elem);

            parse_quote! {
                {
                    let mut #elem_var = #vec_var.push_elem();
                    #(#elem_stmts)*
                }
            }
        });

        parse_quote! {
            {
                let mut #vec_var = #nav;
                #(#inner_stmts)*
            }
        }
    }

    /// Emit code for an enum variant assignment.
    ///
    /// Struct-like variant:
    /// ```text
    /// navigate!(…).set(Enum::Variant { a: Default::default(), b: Default::default() });
    /// if let Enum::Variant { a, b } = navigate!(…).peel() {
    ///     navigate!(*a).set(10);
    ///     navigate!(*b).set("hello");
    /// }
    /// ```
    ///
    /// Tuple-like variant (keys `$0`, `$1`, …):
    /// ```text
    /// navigate!(…).set(Enum::Variant(Default::default(), Default::default()));
    /// if let Enum::Variant(_0, _1) = navigate!(…).peel() {
    ///     navigate!(*_0).set(10);
    ///     navigate!(*_1).set("hello");
    /// }
    /// ```
    ///
    /// Unit variant (no extra keys):
    /// ```text
    /// navigate!(…).set(Enum::Variant);
    /// ```
    fn visit_enum(
        &mut self,
        enum_name: &str,
        variant_name: &str,
        fields: &Option<AgentLocalToolConfigNested>,
    ) -> Vec<syn::Stmt> {
        let enum_name = format_ident!("{enum_name}");
        let variant_name = format_ident!("{variant_name}");

        let nav: syn::Expr = self.navigate();

        match fields {
            // ── Struct-like variant ───────────────────────────────────────────────
            Some(AgentLocalToolConfigNested::Struct(fields)) => {
                let field_idents: Vec<syn::Ident> =
                    fields.keys().map(|k| format_ident!("{k}")).collect();

                let mut arm_stmts = Vec::new();
                for (key, val) in fields {
                    let field_ident = format_ident!("{key}");
                    arm_stmts.extend(Visitor::deref(&field_ident, self.counter).visit(val));
                }

                vec![
                    parse_quote! {
                        #nav.set(#enum_name::#variant_name {
                            #(#field_idents: Default::default(),)*
                        });
                    },
                    parse_quote! {
                        if let #enum_name::#variant_name { #(#field_idents,)* } = #nav.peel() {
                            #(#arm_stmts)*
                        }
                    },
                ]
            }

            // ── Tuple-like variant ────────────────────────────────────────────────
            Some(AgentLocalToolConfigNested::Tuple(fields)) => {
                let len = fields.len();

                // Bindings `_0`, `_1`, … (leading underscore avoids unused warnings
                // for slots with no populated value).
                let slot_idents = (0..len).map(|i| format_ident!("_{i}"));

                let default: syn::Expr = parse_quote! { Default::default() };
                let defaults = std::iter::repeat_n(default, len);

                let arm_stmts = fields
                    .iter()
                    .enumerate()
                    .flat_map(|(i, slot)| {
                        let slot_ident = format_ident!("_{i}");
                        let slot = slot.as_ref()?;
                        Some(Visitor::deref(&slot_ident, self.counter).visit(slot))
                    })
                    .flatten();

                vec![
                    parse_quote! { #nav.set(#enum_name::#variant_name(#(#defaults),*)); },
                    parse_quote! {
                        if let #enum_name::#variant_name(#(#slot_idents),*) = #nav.peel() {
                            #(#arm_stmts)*
                        }
                    },
                ]
            }

            // ── Unit variant ─────────────────────────────────────────────────────
            None => vec![parse_quote! { #nav.set(#enum_name::#variant_name); }],
        }
    }

    /// Emit code for a plain tuple value (no enum discriminant).
    ///
    /// Each `$N` slot extends the current path with a tuple index (`.0`, `.1`, …)
    /// and recurses — the containing struct's `Default` already leaves missing
    /// slots defaulted, so we only emit assignments for slots that have values.
    fn visit_tuple(&mut self, fields: &[Option<AgentLocalToolConfig>]) -> Vec<syn::Stmt> {
        fields
            .iter()
            .enumerate()
            .flat_map(|(i, slot)| {
                let slot = slot.as_ref()?;
                self.path.push(syn::Member::Unnamed(syn::Index::from(i)));
                let stmts = self.visit(slot);
                self.path.pop();
                Some(stmts)
            })
            .flatten()
            .collect()
    }

    /// Emit struct-field navigation: each key extends the current path.
    fn visit_struct(&mut self, fields: &BTreeMap<String, AgentLocalToolConfig>) -> Vec<syn::Stmt> {
        fields
            .iter()
            .flat_map(|(key, val)| {
                let key_ident = format_ident!("{key}");
                self.path.push(syn::Member::Named(key_ident));
                let stmts = self.visit(val);
                self.path.pop();
                stmts
            })
            .collect()
    }

    // /// Emit HashMap-entry navigation:
    // ///
    // /// Currently unused, not sure if it will be useful
    // ///
    // /// ```text
    // /// {
    // ///     let mut __mN = navigate!(root . path);
    // ///     {
    // ///         let mut __eM = __mN.insert_elem("key".into());
    // ///         navigate!(*__eM . field).set(value);
    // ///         …
    // ///     }
    // /// }
    // /// ```
    // #[allow(dead_code)]
    // fn visit_map(
    //     &mut self,
    //     table: &toml::map::Map<String, toml::Value>,
    //     ctx: &mut Visitor<'_>,
    // ) -> Result<syn::Stmt, ToolInitParamsError> {
    //     let nav: syn::Expr = self.navigate();
    //     let map_var = ctx.fresh_var("m");

    //     let mut entry_stmts: Vec<syn::Stmt> = Vec::new();
    //     for (key, val) in table {
    //         if key.starts_with('$') {
    //             continue;
    //         }
    //         let entry_var = ctx.fresh_var("e");
    //         // The value under a map key may itself be a struct or array.
    //         let inner_stmts = Visitor::deref(&entry_var, ctx.counter).visit(val)?;
    //         entry_stmts.push(parse_quote! {
    //             {
    //                 let mut #entry_var = #map_var.insert_elem(#key.into());
    //                 #(#inner_stmts)*
    //             }
    //         });
    //     }

    //     Ok(parse_quote! {
    //         {
    //             let mut #map_var = #nav;
    //             #(#entry_stmts)*
    //         }
    //     })
    // }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use quote::ToTokens;
    use rstest::rstest;

    use super::*;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn prettify(expr: &syn::Expr) -> String {
        let file: syn::File = parse_quote! {
            fn __test() -> () {
                let _ = #expr;
            }
        };
        prettyplease::unparse(&file)
    }

    // ── build_navigate ────────────────────────────────────────────────────────

    #[rstest]
    #[case::single_field("__x", ["foo"], "talin :: __private :: navigate ! (__x . foo)")]
    #[case::multi_field("__x", ["a", "b", "c"], "talin :: __private :: navigate ! (__x . a . b . c)")]
    fn test_navigate_owned<const N: usize>(
        #[case] root: &str,
        #[case] path: [&str; N],
        #[case] expected: &str,
    ) {
        let mut counter = 0;
        let root = format_ident!("{root}");

        let mut visitor = Visitor::owned(&root, &mut counter);
        visitor.path = path
            .iter()
            .map(|s| syn::Member::Named(format_ident!("{s}")))
            .collect();

        let expr = visitor.navigate();
        assert_eq!(expr.to_token_stream().to_string(), expected);
    }

    #[rstest]
    #[case::with_field("__e0", ["inner"], "talin :: __private :: navigate ! (* __e0 . inner)")]
    #[case::empty_path("__e0", [], "talin :: __private :: navigate ! (* __e0)")]
    fn test_build_navigate_deref<const N: usize>(
        #[case] root: &str,
        #[case] path: [&str; N],
        #[case] expected: &str,
    ) {
        let mut counter = 0;
        let root = format_ident!("{root}");

        let mut visitor = Visitor::deref(&root, &mut counter);
        visitor.path = path
            .iter()
            .map(|s| syn::Member::Named(format_ident!("{s}")))
            .collect();

        let expr = visitor.navigate();
        assert_eq!(expr.to_token_stream().to_string(), expected);
    }

    // ── tool_init_params — pretty-printed output ──────────────────────────────

    // #[rstest]
    // #[case("MyInit", "x = 1\ny = \"hi\"")]
    #[rstest]
    #[case::flat_object(
        "MyInit",
        indoc! {
            r#"
            x = 1
            y = "hi""#
        },
        indoc! {
            r#"
            {
                let mut __x = MyInit::default();
                talin::__private::navigate!(__x.x).set(1);
                talin::__private::navigate!(__x.y).set_from("hi");
                __x
            }
            "#
        },
    )]
    #[case::nested_object(
        "MyInit",
        indoc! {
            r#"
            name = "origin"
            point = { x = 10, y = 20 }"#
        },
        indoc! {
            r#"
            {
                let mut __x = MyInit::default();
                talin::__private::navigate!(__x.name).set_from("origin");
                talin::__private::navigate!(__x.point.x).set(10);
                talin::__private::navigate!(__x.point.y).set(20);
                __x
            }
            "#
        },
    )]
    #[case::nested_object_deep(
        "MapInit",
        indoc! {r#"
            [delta.key_a]
            s = "hello"
            t = 5"#
        },
        indoc! {r#"
            {
                let mut __x = MapInit::default();
                talin::__private::navigate!(__x.delta.key_a.s).set_from("hello");
                talin::__private::navigate!(__x.delta.key_a.t).set(5);
                __x
            }
        "#},
    )]
    #[case::tuple(
        "TupInit",
        indoc! {r#"
            [coord]
            "$0" = 10
            "$1" = 20
        "#},
        indoc! {r#"
            {
                let mut __x = TupInit::default();
                talin::__private::navigate!(__x.coord.0).set(10);
                talin::__private::navigate!(__x.coord.1).set(20);
                __x
            }
        "#},
    )]
    #[case::array(
        "ArrInit",
        "items = [1, 2, 3]",
        indoc! {r#"
            {
                let mut __x = ArrInit::default();
                {
                    let mut __v0 = talin::__private::navigate!(__x.items);
                    {
                        let mut __e1 = __v0.push_elem();
                        talin::__private::navigate!(* __e1).set(1);
                    }
                    {
                        let mut __e2 = __v0.push_elem();
                        talin::__private::navigate!(* __e2).set(2);
                    }
                    {
                        let mut __e3 = __v0.push_elem();
                        talin::__private::navigate!(* __e3).set(3);
                    }
                }
                __x
            }
        "#},
    )]
    #[case::arrays_nested(
        "MapInit",
        indoc! {r#"
            [theta]
            scores = [10, 20, 30]
            "#
        },
        indoc! {r#"
            {
                let mut __x = MapInit::default();
                {
                    let mut __v0 = talin::__private::navigate!(__x.theta.scores);
                    {
                        let mut __e1 = __v0.push_elem();
                        talin::__private::navigate!(* __e1).set(10);
                    }
                    {
                        let mut __e2 = __v0.push_elem();
                        talin::__private::navigate!(* __e2).set(20);
                    }
                    {
                        let mut __e3 = __v0.push_elem();
                        talin::__private::navigate!(* __e3).set(30);
                    }
                }
                __x
            }
        "#},
    )]
    #[case::enum_unit(
        "EnumInit",
        indoc! {r#"
            [field]
            "$enum" = "MyEnum"
            "$variant" = "UnitVar""#
        },
        indoc! {r#"
            {
                let mut __x = EnumInit::default();
                talin::__private::navigate!(__x.field).set(MyEnum::UnitVar);
                __x
            }
        "#},
    )]
    #[case::enum_tuple(
        "EnumInit",
        indoc! {r#"
            [field]
            "$enum" = "MyEnum"
            "$variant" = "TupleVar"
            "$0" = 10
            "$1" = "hello""#
        },
        indoc! {r#"
            {
                let mut __x = EnumInit::default();
                talin::__private::navigate!(__x.field).set(MyEnum::TupleVar(Default::default(), Default::default()));
                if let MyEnum::TupleVar(_0, _1) = talin::__private::navigate!(__x.field).peel() {
                    talin::__private::navigate!(*_0).set(10);
                    talin::__private::navigate!(*_1).set_from("hello");
                }
                __x
            }
        "#},
    )]
    #[case::enum_struct(
        "EnumInit",
        indoc! {r#"
            [field]
            "$enum" = "MyEnum"
            "$variant" = "StructVar"
            a = 1
            b = "two""#
        },
        indoc! {r#"
            {
                let mut __x = EnumInit::default();
                talin::__private::navigate!(__x.field).set(MyEnum::StructVar {
                    a: Default::default(),
                    b: Default::default(),
                });
                if let MyEnum::StructVar { a, b } = talin::__private::navigate!(__x.field).peel() {
                    talin::__private::navigate!(*a).set(1);
                    talin::__private::navigate!(*b).set_from("two");
                }
                __x
            }
        "#},
    )]
    fn test_init_params(#[case] symbol: &str, #[case] toml_str: &str, #[case] expected: &str) {
        let ident: syn::Ident = format_ident!("{symbol}");
        let ident = parse_quote!(#ident);
        let value: AgentLocalToolConfig = toml::from_str(toml_str).unwrap();
        let expr = tool_init_params(&ident, &value);

        let prettified = prettify(&expr);
        let expected = prettify(&syn::parse_str(expected).unwrap());
        assert_eq!(prettified, expected);
    }

    // ── tool_init_params — error cases ────────────────────────────────────────

    #[rstest]
    #[case::enum_without_variant(
        indoc! {r#"
            [field]
            "$enum" = "MyEnum"
        "#},
    )]
    #[case::variant_without_enum(
        indoc! {r#"
            [field]
            "$variant" = "Foo"
        "#},
    )]
    #[case::tuple_idx_mixed_with_regular(
        indoc! {r#"
            [field]
            "$0" = 1
            a = 2
        "#},
    )]
    #[case::invalid_nested(
        indoc! {r#"
            [outer.inner]
            "$enum" = "E"
        "#},
    )]
    #[case::non_string_enum(
        indoc! {r#"
            [field]
            "$enum" = 42
            "$variant" = "Foo"
        "#},
    )]
    #[case::non_string_variant(
        indoc! {r#"
            [field]
            "$enum" = "MyEnum"
            "$variant" = 42
        "#},
    )]
    fn test_init_params_errors(#[case] toml_str: &str) {
        toml::from_str::<AgentLocalToolConfig>(toml_str).unwrap_err();
    }
}
