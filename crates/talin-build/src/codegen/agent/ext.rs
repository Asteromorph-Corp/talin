use syn::parse_quote;

use crate::codegen::agent::Variables;
use crate::codegen::common::talin;
use crate::ty::{extract_array_like_type, is_string_like_type, lifetime_input, rename_lifetimes};

/// Selects which kind of runner the extension methods are generated for.
///
/// An agent is stateless: its run methods take `&self` and go through `AgentRun`.
/// A session owns a mutable transcript, so its run methods take `&mut self` and go through
/// `SessionRun`.
#[derive(Clone, Copy)]
enum Runner {
    Agent,
    Session,
}

impl Runner {
    /// The receiver for the turn-running methods. `get_prompt_immed` does not mutate and always
    /// takes `&self`, regardless of runner.
    fn receiver(self) -> syn::Receiver {
        match self {
            Runner::Agent => parse_quote! { &self },
            Runner::Session => parse_quote! { &mut self },
        }
    }

    /// The library trait the extension trait builds on.
    fn run_trait(self) -> syn::Ident {
        match self {
            Runner::Agent => parse_quote! { AgentRun },
            Runner::Session => parse_quote! { SessionRun },
        }
    }

    /// The method called to run a turn with the default strategy.
    fn run(self) -> syn::Ident {
        match self {
            Runner::Agent => parse_quote! { run },
            Runner::Session => parse_quote! { run_turn },
        }
    }

    /// The method called to run a turn with an explicit strategy.
    fn run_with_strategy(self) -> syn::Ident {
        match self {
            Runner::Agent => parse_quote! { run_with_strategy },
            Runner::Session => parse_quote! { run_turn_with_strategy },
        }
    }

    /// The bound a caller-supplied strategy must satisfy: it has to be runnable on the *session*
    /// type, which is `Self` for sessions and `<Self as AgentRun>::Session` for agents.
    fn strategy_bound(
        self,
        input_ctor: &syn::Type,
        output_ctor: &syn::Type,
    ) -> syn::TypeParamBound {
        let talin = talin();
        let run_trait = self.run_trait();
        let session_ty: syn::Type = match self {
            Runner::Agent => parse_quote! {
                <Self as #talin::agent::#run_trait<#input_ctor, #output_ctor>>::Session
            },
            Runner::Session => parse_quote! { Self },
        };
        parse_quote! {
            #talin::agent::StrategyFor<#session_ty, #input_ctor, #output_ctor>
        }
    }
}

pub(super) fn trait_agent_ext(
    ext_symbol: &syn::Ident,
    input_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
    variables: &Variables<'_>,
) -> syn::ItemTrait {
    ext_trait(
        ext_symbol,
        input_symbol,
        input_ctor,
        output_ctor,
        variables,
        Runner::Agent,
    )
}

pub(super) fn trait_session_ext(
    ext_symbol: &syn::Ident,
    input_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
    variables: &Variables<'_>,
) -> syn::ItemTrait {
    ext_trait(
        ext_symbol,
        input_symbol,
        input_ctor,
        output_ctor,
        variables,
        Runner::Session,
    )
}

pub(super) fn impl_agent_ext(
    ext_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
) -> syn::ItemImpl {
    ext_impl(ext_symbol, input_ctor, output_ctor, Runner::Agent)
}

pub(super) fn impl_session_ext(
    ext_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
) -> syn::ItemImpl {
    ext_impl(ext_symbol, input_ctor, output_ctor, Runner::Session)
}

/// Builds the extension trait, with the `*_immed` methods as provided methods.
///
/// The trait uses the library's `AgentRun`/`SessionRun`.
fn ext_trait(
    ext_symbol: &syn::Ident,
    input_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
    variables: &Variables<'_>,
    runner: Runner,
) -> syn::ItemTrait {
    let talin = talin();
    let run_trait = runner.run_trait();
    let methods = ext_methods(input_symbol, input_ctor, output_ctor, variables, runner);

    parse_quote! {
        #[#talin::__private::async_trait::async_trait]
        #[allow(dead_code)]
        pub trait #ext_symbol:
            #talin::agent::#run_trait<#input_ctor, #output_ctor>
        {
            #(#methods)*
        }
    }
}

/// Blanket-implements the extension trait for every runner of this agent.
fn ext_impl(
    ext_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
    runner: Runner,
) -> syn::ItemImpl {
    let talin = talin();
    let run_trait = runner.run_trait();
    parse_quote! {
        #[#talin::__private::async_trait::async_trait]
        #[automatically_derived]
        impl<R> #ext_symbol for R
        where
            R: #talin::agent::#run_trait<#input_ctor, #output_ctor>,
        {
        }
    }
}

fn to_arg_type(ty: &syn::Type, lifetime: Option<&syn::Lifetime>) -> syn::Type {
    if is_string_like_type(ty) {
        parse_quote! { &#lifetime str }
    } else if let Some(elem) = extract_array_like_type(ty) {
        let elem = rename_lifetimes(elem.clone(), lifetime);
        parse_quote! { &#lifetime [#elem] }
    } else if let syn::Type::Reference(reference) = ty {
        let inner = rename_lifetimes((*reference.elem).clone(), lifetime);
        parse_quote! { &#lifetime #inner }
    } else {
        rename_lifetimes(ty.clone(), lifetime)
    }
}

/// The input fields of an agent as method arguments: their identifiers, the typed arguments
/// borrowing from the call lifetime `'input`, and the same arguments with elided lifetimes (for
/// methods that do not declare one).
struct InputArgs {
    idents: Vec<syn::Ident>,
    args: Vec<syn::Field>,
    args_elided: Vec<syn::Field>,
}

impl InputArgs {
    fn collect(variables: &Variables<'_>, lifetime: &syn::Lifetime) -> Self {
        let input_variables = variables.inputs();
        let idents = input_variables
            .iter()
            .map(|var| var.ident.clone())
            .collect();
        let args = input_variables
            .iter()
            .map(|var| -> syn::Field {
                let ident = &var.ident;
                let type_ = to_arg_type(&var.type_, Some(lifetime));
                parse_quote! { #ident: #type_ }
            })
            .collect();
        let args_elided = input_variables
            .iter()
            .map(|var| -> syn::Field {
                let ident = &var.ident;
                let type_ = to_arg_type(&var.type_, None);
                parse_quote! { #ident: #type_ }
            })
            .collect();
        Self {
            idents,
            args,
            args_elided,
        }
    }
}

fn ext_methods(
    input_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
    variables: &Variables<'_>,
    runner: Runner,
) -> Vec<syn::TraitItemFn> {
    let talin = talin();
    let lifetime = lifetime_input();
    let InputArgs {
        idents: input_idents,
        args: input_args,
        args_elided: input_args_elided,
    } = InputArgs::collect(variables, &lifetime);

    let recv = runner.receiver();
    let run = runner.run();
    let run_with_strategy = runner.run_with_strategy();
    let strategy_bound = runner.strategy_bound(input_ctor, output_ctor);
    vec![
        parse_quote! {
            #[allow(clippy::too_many_arguments)]
            async fn run_immed<#lifetime>(
                #recv,
                #(#input_args),*
            ) -> Result<
                <#output_ctor as #talin::chain::OutputCtor>::Target<#lifetime>,
                #talin::agent::AgentError,
            > {
                self.#run(#input_symbol::new(#(#input_idents),*)).await
            }
        },
        parse_quote! {
            #[allow(clippy::too_many_arguments)]
            async fn run_with_strategy_immed<#lifetime, S2>(
                #recv,
                strategy: S2,
                #(#input_args),*
            ) -> Result<
                #talin::agent::AgentOutput<
                    <#output_ctor as #talin::chain::OutputCtor>::Target<#lifetime>,
                    <S2 as #strategy_bound>::Output,
                >,
                #talin::agent::AgentError,
            >
            where
                S2: #strategy_bound,
            {
                self.#run_with_strategy(strategy, #input_symbol::new(#(#input_idents),*)).await
            }
        },
        parse_quote! {
            #[allow(clippy::too_many_arguments)]
            fn get_prompt_immed(
                &self,
                #(#input_args_elided),*
            ) -> Result<
                #talin::schemas::Prompt,
                #talin::template::TemplateError
            > {
                #talin::chain::GetPrompt::get_prompt(
                    self,
                    #input_symbol::new(#(#input_idents),*),
                )
            }
        },
    ]
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::config::{Attachments, RuntimeVariable};
    use crate::options::ArtifactOptions;
    use crate::test_utils::{
        assert_item_eq, attachments, ext_symbol, input_ctor, input_symbol, output_ctor,
        session_ext_symbol, variables,
    };

    #[rstest]
    fn test_trait_agent_ext(
        ext_symbol: syn::Ident,
        input_symbol: syn::Ident,
        input_ctor: syn::Type,
        output_ctor: syn::Type,
        variables: Vec<RuntimeVariable>,
        attachments: Attachments,
    ) {
        let variables = Variables::collect(&variables, &attachments, &ArtifactOptions::default());
        let trait_ext = trait_agent_ext(
            &ext_symbol,
            &input_symbol,
            &input_ctor,
            &output_ctor,
            &variables,
        );
        assert_item_eq(&trait_ext, "agent/ext/trait_agent_ext.rs");
    }

    #[rstest]
    fn test_impl_agent_ext(ext_symbol: syn::Ident, input_ctor: syn::Type, output_ctor: syn::Type) {
        let impl_ext = impl_agent_ext(&ext_symbol, &input_ctor, &output_ctor);
        assert_item_eq(&impl_ext, "agent/ext/impl_agent_ext.rs");
    }

    #[rstest]
    fn test_trait_session_ext(
        session_ext_symbol: syn::Ident,
        input_symbol: syn::Ident,
        input_ctor: syn::Type,
        output_ctor: syn::Type,
        variables: Vec<RuntimeVariable>,
        attachments: Attachments,
    ) {
        let variables = Variables::collect(&variables, &attachments, &ArtifactOptions::default());
        let trait_ext = trait_session_ext(
            &session_ext_symbol,
            &input_symbol,
            &input_ctor,
            &output_ctor,
            &variables,
        );
        assert_item_eq(&trait_ext, "agent/ext/trait_session_ext.rs");
    }

    #[rstest]
    fn test_impl_session_ext(
        session_ext_symbol: syn::Ident,
        input_ctor: syn::Type,
        output_ctor: syn::Type,
    ) {
        let impl_ext = impl_session_ext(&session_ext_symbol, &input_ctor, &output_ctor);
        assert_item_eq(&impl_ext, "agent/ext/impl_session_ext.rs");
    }

    /// With a named call lifetime, borrowed forms use it and any nested lifetime (a slice
    /// element or an owned generic) is unified onto it.
    #[test]
    fn test_to_arg_type_named_lifetime() {
        let lt: syn::Lifetime = parse_quote! { 'input };

        let ty: syn::Type = parse_quote! { std::borrow::Cow<'input, str> };
        let expected: syn::Type = parse_quote! { &'input str };
        assert_eq!(to_arg_type(&ty, Some(&lt)), expected);

        let ty: syn::Type = parse_quote! { std::borrow::Cow<'input, [Foo<'input>]> };
        let expected: syn::Type = parse_quote! { &'input [Foo<'input>] };
        assert_eq!(to_arg_type(&ty, Some(&lt)), expected);

        let ty: syn::Type = parse_quote! { Foo<'input> };
        let expected: syn::Type = parse_quote! { Foo<'input> };
        assert_eq!(to_arg_type(&ty, Some(&lt)), expected);
    }

    /// Without a call lifetime (the elided `get_prompt_immed` signature), nested lifetimes
    /// collapse to `'_` so the argument never references a lifetime the signature does not
    /// declare — the case that previously produced uncompilable code.
    #[test]
    fn test_to_arg_type_elided_lifetime() {
        let ty: syn::Type = parse_quote! { std::borrow::Cow<'input, str> };
        let expected: syn::Type = parse_quote! { &str };
        assert_eq!(to_arg_type(&ty, None), expected);

        let ty: syn::Type = parse_quote! { std::borrow::Cow<'input, [Foo<'input>]> };
        let expected: syn::Type = parse_quote! { &[Foo<'_>] };
        assert_eq!(to_arg_type(&ty, None), expected);

        let ty: syn::Type = parse_quote! { Foo<'input> };
        let expected: syn::Type = parse_quote! { Foo<'_> };
        assert_eq!(to_arg_type(&ty, None), expected);
    }

    /// With newtype inputs, `*_immed` take the newtype and the constructor accepts `Into`.
    #[rstest]
    fn test_newtype_inputs(
        ext_symbol: syn::Ident,
        input_symbol: syn::Ident,
        input_ctor: syn::Type,
        output_ctor: syn::Type,
    ) {
        use quote::ToTokens;

        let vars = [RuntimeVariable::Input(Box::new(
            crate::config::InputVariable {
                ident: quote::format_ident!("language"),
                type_: parse_quote!(std::borrow::Cow<'a, str>),
            },
        ))];
        let options = ArtifactOptions::default().use_newtype_inputs(true);
        let variables = Variables::collect(&vars, &Attachments::default(), &options);
        let tokens = trait_agent_ext(
            &ext_symbol,
            &input_symbol,
            &input_ctor,
            &output_ctor,
            &variables,
        )
        .to_token_stream()
        .to_string();
        assert!(tokens.contains("language : Language"), "{tokens}");
        assert!(!tokens.contains("Into"), "{tokens}");
        assert!(!tokens.contains("& str"), "{tokens}");
    }
}
