pub async fn session<P: MyAgentProfile>(
    profile: P,
) -> ::talin::agent::AgentSession<P::Llm, MyAgentInputCtor, MyAgentOutputCtor, P::Strategy>
{
    Self::new(profile).await.session()
}
