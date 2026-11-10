use syn::parse_quote;

use crate::codegen::common::talin;

pub(super) fn fn_new(
    id: &str,
    profile_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
) -> syn::ItemFn {
    let talin = talin();

    parse_quote! {
        #[allow(clippy::new_ret_no_self)]
        pub async fn new<P: #profile_symbol>(profile: P) -> #talin::agent::Agent<P::Llm, #input_ctor, #output_ctor, P::Strategy> {
            let (model, strategy) = profile.into_preset();

            let system_prompt = P::system_prompt();
            let user_prompt = P::user_prompt()
                .lines()
                .map(|line| line.trim_start_matches("!"))
                .collect::<Vec<_>>().join("\n");
            let media = P::media();
            let skills = P::skills();

            let system_message = system_prompt.map(#talin::template::MessageTemplate::jinja2);
            let user_message = #talin::template::MessageTemplate::jinja2(user_prompt).with_media(media);

            let mut builder = #talin::agent::Agent::<_, #input_ctor, #output_ctor>::builder().id(#id);
            if let Some(system_message) = system_message {
                builder = builder.system_message(system_message);
            }

            builder
                .user_message(user_message)
                .tools(P::tools())
                .skills(skills)
                .mcp_servers(P::mcp_servers())
                .specialize::<P::Llm>()
                .builtin_tools(P::builtin_tools())
                .strategy(strategy)
                .build(model)
                .with_options(P::agent_options())
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{agent_id, assert_item_eq, input_ctor, output_ctor, profile_symbol};

    #[rstest]
    fn test_fn_new(
        agent_id: &str,
        profile_symbol: syn::Ident,
        input_ctor: syn::Type,
        output_ctor: syn::Type,
    ) {
        let fn_new = fn_new(agent_id, &profile_symbol, &input_ctor, &output_ctor);
        assert_item_eq(&fn_new, "agent/fn_new.rs");
    }
}
