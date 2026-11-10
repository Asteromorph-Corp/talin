#[allow(clippy::new_ret_no_self)]
pub async fn new<P: MyAgentProfile>(
    profile: P,
) -> ::talin::agent::Agent<P::Llm, MyAgentInputCtor, MyAgentOutputCtor, P::Strategy> {
    let (model, strategy) = profile.into_preset();
    let system_prompt = P::system_prompt();
    let user_prompt = P::user_prompt()
        .lines()
        .map(|line| line.trim_start_matches("!"))
        .collect::<Vec<_>>()
        .join("\n");
    let media = P::media();
    let skills = P::skills();
    let system_message = system_prompt.map(::talin::template::MessageTemplate::jinja2);
    let user_message =
        ::talin::template::MessageTemplate::jinja2(user_prompt).with_media(media);
    let mut builder =
        ::talin::agent::Agent::<_, MyAgentInputCtor, MyAgentOutputCtor>::builder()
            .id("my_agent");
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
