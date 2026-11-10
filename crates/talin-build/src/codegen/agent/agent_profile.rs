use heck::ToTitleCase;
use quote::quote;
use syn::parse_quote;

use crate::codegen::agent::options::agent_options_expr;
use crate::codegen::agent::tools::{builtin_tool_expr, local_tool_expr, mcp_server_expr};
use crate::codegen::common::talin;
use crate::config::{
    Agent, AgentBuiltinTool, AgentLocalTool, AgentMcpTool, AgentMcpToolGrouped,
    AgentOverrideValues, Attachments, CallOptions, Merge, ModelType, SchemaMode,
};

pub(super) fn trait_agent_profile(
    agent: &Agent,
    profile_symbol: &syn::Ident,
    output_ctor: &syn::Type,
) -> syn::ItemTrait {
    let talin = talin();

    let fn_system_prompt = fn_system_prompt(agent.system_prompt.as_deref());
    let fn_user_prompt = fn_user_prompt(&agent.user_prompt);
    let fn_media = fn_media(&agent.attachments);
    let fn_tools = fn_tools(&agent.local_tools);
    let fn_builtin_tools = fn_builtin_tools(&[]);
    let fn_skills = fn_skills(&agent.skills);
    let fn_mcp_servers = fn_mcp_servers(&agent.mcp_servers);
    let fn_call_options = fn_call_options(&agent.call_options);
    let fn_agent_options = fn_agent_options(
        agent.max_consecutive_fails,
        agent.max_iterations,
        agent.max_continuations,
    );

    parse_quote! {
        pub trait #profile_symbol {
            type Llm: #talin::llm::Llm;
            type Strategy: #talin::strategy::Strategy<Self::Llm, #output_ctor> + Clone;

            fn into_preset(self) -> (Self::Llm, Self::Strategy);

            #fn_system_prompt
            #fn_user_prompt
            #fn_media
            #fn_tools
            #fn_builtin_tools
            #fn_skills
            #fn_mcp_servers
            #fn_call_options
            #fn_agent_options
        }
    }
}

pub(super) fn impl_agent_profile(
    id: &str,
    agent: &Agent,
    profile_symbol: &syn::Ident,
    output_ctor: &syn::Type,
) -> Vec<syn::ItemImpl> {
    ModelType::ALL
        .into_iter()
        .map(|model_type| {
            impl_agent_profile_single(id, agent, profile_symbol, output_ctor, model_type)
        })
        .collect::<Vec<_>>()
}

fn impl_agent_profile_single(
    id: &str,
    agent: &Agent,
    profile_symbol: &syn::Ident,
    output_ctor: &syn::Type,
    model_type: ModelType,
) -> syn::ItemImpl {
    let talin = talin();

    let model_ty = model_type.to_ty();
    let maybe_with_response_format = match &agent.schema_mode {
        SchemaMode::JsonSchema => {
            let rf_expr = response_format_expr(id, output_ctor);
            Some(quote! { .with_response_format(#rf_expr) })
        }
        SchemaMode::JsonObject => Some(quote! {
            .with_response_format(#talin::schemas::ResponseFormat::JsonObject)
        }),
        SchemaMode::Prompt => None,
    };

    // Every method below the associated types and `into_preset` is emitted if this provider differs
    // from the trait default.
    let overrides = override_methods(agent, model_type);

    parse_quote! {
        impl<S> #profile_symbol for #talin::llm::LlmPreset<#model_ty, S>
        where
            S: #talin::strategy::Strategy<#model_ty, #output_ctor> + Clone,
        {
            type Llm = #model_ty;
            type Strategy = S;

            fn into_preset(self) -> (Self::Llm, Self::Strategy) {
                use #talin::llm::Llm;

                let model = self.model
                    .with_default_options(<Self as #profile_symbol>::call_options())
                    .with_default_options(self.options)
                    .with_default_options(#talin::llm::LlmOptions::new() #maybe_with_response_format);

                (model, self.strategy)
            }

            #(#overrides)*
        }
    }
}

/// Builds the per-provider override methods for a single model type.
///
/// A method is emitted only where this provider's value differs from the agent default.
fn override_methods(agent: &Agent, model_type: ModelType) -> Vec<syn::ImplItemFn> {
    let override_values = agent.overrides.get(&model_type);

    let mut methods = Vec::new();

    let builtin_tools = provider_builtin_tools(agent, override_values, model_type);
    if !builtin_tools.is_empty() {
        methods.push(fn_builtin_tools(&builtin_tools));
    }
    if let Some(system_prompt) = override_values.and_then(|o| o.system_prompt.as_deref()) {
        methods.push(fn_system_prompt(Some(system_prompt)));
    }
    if let Some(user_prompt) = override_values.and_then(|o| o.user_prompt.as_deref()) {
        methods.push(fn_user_prompt(user_prompt));
    }
    if let Some(override_values) = override_values.filter(|o| o.overrides_tools()) {
        let local_tools = agent
            .local_tools
            .iter()
            .chain(&override_values.local_tools)
            .cloned()
            .collect::<Vec<_>>();
        methods.push(fn_tools(&local_tools));
    }
    if let Some(override_values) = override_values.filter(|o| o.overrides_mcp_servers()) {
        let mcp_servers = AgentMcpTool::regroup_tools(
            agent.mcp_servers.clone(),
            override_values.mcp_servers.clone(),
        );
        methods.push(fn_mcp_servers(&mcp_servers));
    }
    if let Some(override_values) = override_values.filter(|o| o.overrides_options()) {
        let max_consecutive_fails = override_values
            .max_consecutive_fails
            .or(agent.max_consecutive_fails);
        let max_iterations = override_values.max_iterations.or(agent.max_iterations);
        let max_continuations = override_values
            .max_continuations
            .or(agent.max_continuations);
        methods.push(fn_agent_options(
            max_consecutive_fails,
            max_iterations,
            max_continuations,
        ));
    }
    if let Some(override_call_options) = override_values.and_then(|o| o.call_options.as_ref()) {
        let call_options = override_call_options
            .clone()
            .merge(agent.call_options.clone());
        methods.push(fn_call_options(&call_options));
    }

    methods
}

fn fn_system_prompt(system_prompt: Option<&str>) -> syn::ImplItemFn {
    let body: syn::Expr = match system_prompt {
        Some(system_prompt) => parse_quote!(Some(#system_prompt)),
        None => parse_quote!(None),
    };
    parse_quote! {
        fn system_prompt() -> Option<&'static str> {
            #body
        }
    }
}

fn fn_user_prompt(user_prompt: &str) -> syn::ImplItemFn {
    parse_quote! {
        fn user_prompt() -> &'static str {
            #user_prompt
        }
    }
}

/// Emits the media slot names the agent's user prompt attaches, in kind order.
fn fn_media(attachments: &Attachments) -> syn::ImplItemFn {
    let media = [
        &attachments.images,
        &attachments.audios,
        &attachments.videos,
        &attachments.documents,
    ]
    .into_iter()
    .flatten();

    parse_quote! {
        fn media() -> Vec<String> {
            vec![
                #(String::from(#media),)*
            ]
        }
    }
}

fn fn_tools(local_tools: &[AgentLocalTool]) -> syn::ImplItemFn {
    let talin = talin();

    let local_tools = local_tools.iter().map(local_tool_expr);

    parse_quote! {
        fn tools() -> Vec<#talin::tools::Tool> {
            vec![#(#talin::tools::Tool::from(#local_tools)),*]
        }
    }
}

fn fn_builtin_tools(builtin_tools: &[&AgentBuiltinTool]) -> syn::ImplItemFn {
    let talin = talin();

    let builtin_tools = builtin_tools.iter().copied().map(builtin_tool_expr);
    parse_quote! {
        fn builtin_tools() -> Vec<<Self::Llm as #talin::llm::Llm>::BuiltinTool> {
            vec![#(#builtin_tools),*]
        }
    }
}

fn fn_skills(skills: &[String]) -> syn::ImplItemFn {
    let talin = talin();
    let skill_set = quote! { #talin::skill::SkillSet };

    if skills.is_empty() {
        return parse_quote! {
            fn skills() -> #skill_set {
                #skill_set::new()
            }
        };
    }

    // Where the skills are depends on the machine the program runs on, so they are looked up at
    // runtime, when they are first used.
    let inserts = skills.iter().map(|name| {
        quote! { skills.insert(#talin::skill::FsSkill::in_home(#name)); }
    });
    parse_quote! {
        fn skills() -> #skill_set {
            let mut skills = #skill_set::new();
            #(#inserts)*
            skills
        }
    }
}

fn fn_mcp_servers(mcp_servers: &[AgentMcpToolGrouped]) -> syn::ImplItemFn {
    let talin = talin();

    let mcp_servers = mcp_servers.iter().map(mcp_server_expr);

    parse_quote! {
        fn mcp_servers() -> Vec<#talin::mcp::McpServer> {
            vec![#(#mcp_servers),*]
        }
    }
}

fn fn_call_options(options: &CallOptions) -> syn::ImplItemFn {
    let talin = talin();

    let model_options = options.to_expr();
    parse_quote! {
        fn call_options() -> #talin::llm::LlmOptions {
            #model_options
        }
    }
}

fn fn_agent_options(
    max_consecutive_fails: Option<usize>,
    max_iterations: Option<usize>,
    max_continuations: Option<usize>,
) -> syn::ImplItemFn {
    let talin = talin();

    let agent_options =
        agent_options_expr(max_consecutive_fails, max_iterations, max_continuations);
    parse_quote! {
        fn agent_options() -> #talin::agent::AgentOptions {
            #agent_options
        }
    }
}

/// The `ResponseFormat::JsonSchema` expression derived from the agent's output type.
///
/// Used to seed the model's default options when the agent declares a response format. The schema
/// name defaults to a title-cased agent id when the generated schema carries no `title`.
fn response_format_expr(id: &str, output_ctor: &syn::Type) -> syn::Expr {
    let talin = talin();

    let default_title = format!("{} Output", id.replace('_', " ").to_title_case());
    parse_quote! {
        {
            let schema = #talin::__private::schemars::schema_for!(#talin::__private::Type<
                <#output_ctor as #talin::chain::OutputCtor>::Target<'_>
            >);
            let name = schema
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(#default_title)
                .to_owned();
            let description = schema
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned());

            #talin::schemas::ResponseFormat::JsonSchema(
                #talin::schemas::ResponseFormatJsonSchema {
                    name,
                    description,
                    schema,
                    strict: Some(true),
                }
            )
        }
    }
}

fn provider_builtin_tools<'a>(
    agent: &'a Agent,
    override_values: Option<&'a AgentOverrideValues>,
    model_type: ModelType,
) -> Vec<&'a AgentBuiltinTool> {
    let override_builtins = override_values.into_iter().flat_map(|o| &o.builtin_tools);
    agent
        .builtin_tools
        .iter()
        .chain(override_builtins)
        .filter(|tool| tool.model_type() == model_type)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::test_utils::{
        assert_items_eq_in_trait, attachments, call_options, local_tools, mcp_servers,
    };

    #[rstest]
    fn test_fn_system_prompt() {
        let fn_system_prompt = fn_system_prompt(Some("You are a helpful assistant."));
        assert_items_eq_in_trait(&[fn_system_prompt], "agent/profile/fn_system_prompt.rs");
    }

    #[rstest]
    fn test_fn_user_prompt() {
        let fn_user_prompt = fn_user_prompt("{{ input }}");
        assert_items_eq_in_trait(&[fn_user_prompt], "agent/profile/fn_user_prompt.rs");
    }

    #[rstest]
    fn test_fn_media(attachments: Attachments) {
        let fn_media = fn_media(&attachments);
        assert_items_eq_in_trait(&[fn_media], "agent/profile/fn_media.rs");
    }

    #[rstest]
    fn test_fn_tools(local_tools: Vec<AgentLocalTool>) {
        let fn_tools = fn_tools(&local_tools);
        assert_items_eq_in_trait(&[fn_tools], "agent/profile/fn_tools.rs");
    }

    #[rstest]
    fn test_fn_mcp_servers(mcp_servers: Vec<AgentMcpToolGrouped>) {
        let fn_mcp_servers = fn_mcp_servers(&mcp_servers);
        assert_items_eq_in_trait(&[fn_mcp_servers], "agent/profile/fn_mcp_servers.rs");
    }

    #[rstest]
    fn test_fn_builtin_tools() {
        let tools = [
            AgentBuiltinTool::from_str("claude_messages/web_search").unwrap(),
            AgentBuiltinTool::from_str("claude_messages/code_execution").unwrap(),
        ];
        let tools = tools.iter().collect::<Vec<_>>();
        let fn_builtin_tools = fn_builtin_tools(&tools);
        assert_items_eq_in_trait(&[fn_builtin_tools], "agent/profile/fn_builtin_tools.rs");
    }

    #[rstest]
    fn test_fn_skills() {
        let skills = ["pdf".to_owned(), "xlsx".to_owned()];
        let fn_skills = fn_skills(&skills);
        assert_items_eq_in_trait(&[fn_skills], "agent/profile/fn_skills.rs");
    }

    #[rstest]
    fn test_fn_skills_empty() {
        let fn_skills = fn_skills(&[]);
        assert_items_eq_in_trait(&[fn_skills], "agent/profile/fn_skills.empty.rs");
    }

    #[rstest]
    #[case::normal(call_options(), "agent/profile/fn_call_options.rs")]
    #[case::default(CallOptions::default(), "agent/profile/fn_call_options.default.rs")]
    fn test_fn_call_options(#[case] call_options: CallOptions, #[case] fixture_path: &str) {
        let fn_call_options = fn_call_options(&call_options);
        assert_items_eq_in_trait(&[fn_call_options], fixture_path);
    }

    #[rstest]
    fn test_agent_options() {
        let fn_agent_options = fn_agent_options(Some(5), Some(10), Some(2));
        assert_items_eq_in_trait(&[fn_agent_options], "agent/profile/fn_agent_options.rs");
    }
}
