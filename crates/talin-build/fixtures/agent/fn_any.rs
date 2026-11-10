pub async fn any(
    preset: impl ::std::convert::Into<::talin::llm::AnyPreset>,
) -> ::talin::agent::AnyAgent<MyAgentInputCtor, MyAgentOutputCtor> {
    match preset.into() {
        ::talin::llm::AnyPreset::GenericChat(preset) => {
            ::talin::agent::AnyAgent::GenericChat(Self::new(*preset).await)
        }
        ::talin::llm::AnyPreset::OpenAIChat(preset) => {
            ::talin::agent::AnyAgent::OpenAIChat(Self::new(*preset).await)
        }
        ::talin::llm::AnyPreset::OpenAIResponses(preset) => {
            ::talin::agent::AnyAgent::OpenAIResponses(Self::new(*preset).await)
        }
        ::talin::llm::AnyPreset::ClaudeMessages(preset) => {
            ::talin::agent::AnyAgent::ClaudeMessages(Self::new(*preset).await)
        }
        ::talin::llm::AnyPreset::GeminiGenerateContent(preset) => {
            ::talin::agent::AnyAgent::GeminiGenerateContent(Self::new(*preset).await)
        }
        ::talin::llm::AnyPreset::GeminiInteractions(preset) => {
            ::talin::agent::AnyAgent::GeminiInteractions(Self::new(*preset).await)
        }
        ::talin::llm::AnyPreset::XaiResponses(preset) => {
            ::talin::agent::AnyAgent::XaiResponses(Self::new(*preset).await)
        }
        ::talin::llm::AnyPreset::SglangGenerate(preset) => {
            ::talin::agent::AnyAgent::SglangGenerate(Self::new(*preset).await)
        }
        _ => unreachable!("`AnyPreset` has a variant that `fn_any` does not handle"),
    }
}
