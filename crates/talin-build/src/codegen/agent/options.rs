use quote::quote;
use syn::parse_quote;

use crate::codegen::common::talin;

/// Builds an `AgentOptions` expression from explicit values, so per-model-type
/// overrides can merge their values over the agent's defaults.
pub(super) fn agent_options_expr(
    max_consecutive_fails: Option<usize>,
    max_iterations: Option<usize>,
    max_continuations: Option<usize>,
) -> syn::Expr {
    let talin = talin();

    let with_options = [
        max_consecutive_fails.map(|value| quote! { .with_max_consecutive_fails(#value) }),
        max_iterations.map(|value| quote! { .with_max_iterations(#value) }),
        max_continuations.map(|value| quote! { .with_max_continuations(#value) }),
    ];

    parse_quote! {
        #talin::agent::AgentOptions::default()
            #(#with_options)*
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::assert_expr_eq;

    #[rstest]
    #[case::normal(Some(5), Some(10), Some(2), "common/options/agent_options.rs")]
    #[case::default(None, None, None, "common/options/agent_options.default.rs")]
    fn test_agent_options_expr(
        #[case] max_consecutive_fails: Option<usize>,
        #[case] max_iterations: Option<usize>,
        #[case] max_continuations: Option<usize>,
        #[case] fixture_path: &str,
    ) {
        let expr = agent_options_expr(max_consecutive_fails, max_iterations, max_continuations);
        assert_expr_eq(&expr, fixture_path);
    }
}
