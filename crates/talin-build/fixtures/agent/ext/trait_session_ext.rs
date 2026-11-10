#[::talin::__private::async_trait::async_trait]
#[allow(dead_code)]
pub trait MyAgentSessionExt:
    ::talin::agent::SessionRun<MyAgentInputCtor, MyAgentOutputCtor>
{
    #[allow(clippy::too_many_arguments)]
    async fn run_immed<'input>(
        &mut self,
        audio: &'input [::talin::schemas::AudioContent],
        document: &'input [::talin::schemas::DocumentContent],
        image: &'input [::talin::schemas::ImageContent],
        input: &'input str,
        video: &'input [::talin::schemas::VideoContent],
    ) -> Result<
        <MyAgentOutputCtor as ::talin::chain::OutputCtor>::Target<'input>,
        ::talin::agent::AgentError,
    > {
        self.run_turn(MyAgentInput::new(audio, document, image, input, video))
            .await
    }
    #[allow(clippy::too_many_arguments)]
    async fn run_with_strategy_immed<'input, S2>(
        &mut self,
        strategy: S2,
        audio: &'input [::talin::schemas::AudioContent],
        document: &'input [::talin::schemas::DocumentContent],
        image: &'input [::talin::schemas::ImageContent],
        input: &'input str,
        video: &'input [::talin::schemas::VideoContent],
    ) -> Result<
        ::talin::agent::AgentOutput<
            <MyAgentOutputCtor as ::talin::chain::OutputCtor>::Target<'input>,
            <S2 as ::talin::agent::StrategyFor<
                Self,
                MyAgentInputCtor,
                MyAgentOutputCtor,
            >>::Output,
        >,
        ::talin::agent::AgentError,
    >
    where
        S2: ::talin::agent::StrategyFor<Self, MyAgentInputCtor, MyAgentOutputCtor>,
    {
        self.run_turn_with_strategy(
            strategy,
            MyAgentInput::new(audio, document, image, input, video),
        )
        .await
    }
    #[allow(clippy::too_many_arguments)]
    fn get_prompt_immed(
        &self,
        audio: &[::talin::schemas::AudioContent],
        document: &[::talin::schemas::DocumentContent],
        image: &[::talin::schemas::ImageContent],
        input: &str,
        video: &[::talin::schemas::VideoContent],
    ) -> Result<::talin::schemas::Prompt, ::talin::template::TemplateError>
    {
        ::talin::chain::GetPrompt::get_prompt(
            self,
            MyAgentInput::new(audio, document, image, input, video),
        )
    }
}
