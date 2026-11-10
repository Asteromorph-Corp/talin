#[deprecated(note = "Use `default` instead")]
pub async fn async_default() -> ::talin::agent::Agent<
    ::talin::llm::GenericChat,
    MyAgentInputCtor,
    MyAgentOutputCtor,
    (),
> {
    Self::default().await
}
