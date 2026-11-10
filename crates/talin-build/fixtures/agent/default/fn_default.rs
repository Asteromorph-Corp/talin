pub async fn default() -> ::talin::agent::Agent<
    ::talin::llm::GenericChat,
    MyAgentInputCtor,
    MyAgentOutputCtor,
    (),
> {
    Self::new(Self::default_model()).await
}
