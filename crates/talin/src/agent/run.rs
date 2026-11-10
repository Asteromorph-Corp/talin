//! Traits shared by the concrete and model-erased agents.
//!
//! [`Agent`] and [`AnyAgent`] (and likewise [`AgentSession`] and [`AnyAgentSession`]) expose the
//! same run methods, but differ in which strategies they accept: `Strategy<L, O>` for a concrete
//! `L`, versus [`UniversalStrategy`] for the erased enum.
//! [`AgentRun`] and [`SessionRun`] abstract over that difference.

use async_trait::async_trait;

use crate::agent::{Agent, AgentError, AgentOutput, AgentSession, AnyAgent, AnyAgentSession};
use crate::chain::{GetPrompt, InputCtor, OutputCtor, PartialWithInput};
use crate::llm::Llm;
use crate::strategy::{Strategy, UniversalStrategy};

/// An agent that can run one-shot turns: [`Agent`] or [`AnyAgent`].
#[async_trait]
pub trait AgentRun<I: InputCtor, O: OutputCtor>:
    for<'a> GetPrompt<I::Target<'a>> + Sync + Sized
{
    /// The session type this agent mints.
    type Session: SessionRun<I, O>;

    /// Begins a fresh session using the agent's configuration.
    fn session(&self) -> Self::Session;

    /// One-shot run with the default strategy.
    async fn run<'input>(&self, input: I::Target<'input>) -> Result<O::Target<'input>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>;

    /// One-shot run with a custom strategy.
    async fn run_with_strategy<'input, A>(
        &self,
        strategy: A,
        input: I::Target<'input>,
    ) -> Result<AgentOutput<O::Target<'input>, A::Output>, AgentError>
    where
        A: StrategyFor<Self::Session, I, O>,
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        self.session().run_turn_with_strategy(strategy, input).await
    }
}

/// A conversation that can run turns: [`AgentSession`] or [`AnyAgentSession`].
#[async_trait]
pub trait SessionRun<I: InputCtor, O: OutputCtor>:
    for<'a> GetPrompt<I::Target<'a>> + Send + Sized
where
    ():,
{
    /// Runs one turn to completion with the default strategy.
    async fn run_turn<'input>(
        &mut self,
        input: I::Target<'input>,
    ) -> Result<O::Target<'input>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>;

    /// Runs one turn to completion with a custom strategy.
    async fn run_turn_with_strategy<'input, A>(
        &mut self,
        strategy: A,
        input: I::Target<'input>,
    ) -> Result<AgentOutput<O::Target<'input>, A::Output>, AgentError>
    where
        A: StrategyFor<Self, I, O>,
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        strategy.run_turn_on(self, input).await
    }
}

#[async_trait]
impl<L, I, O, S> AgentRun<I, O> for Agent<L, I, O, S>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
{
    type Session = AgentSession<L, I, O, S>;

    fn session(&self) -> Self::Session {
        self.session()
    }

    async fn run<'input>(&self, input: I::Target<'input>) -> Result<O::Target<'input>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        self.run(input).await
    }
}

#[async_trait]
impl<I, O> AgentRun<I, O> for AnyAgent<I, O>
where
    I: InputCtor,
    O: OutputCtor,
{
    type Session = AnyAgentSession<I, O>;

    fn session(&self) -> Self::Session {
        self.session()
    }

    async fn run<'input>(&self, input: I::Target<'input>) -> Result<O::Target<'input>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        self.run(input).await
    }
}

#[async_trait]
impl<L, I, O, S> SessionRun<I, O> for AgentSession<L, I, O, S>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
{
    async fn run_turn<'input>(
        &mut self,
        input: I::Target<'input>,
    ) -> Result<O::Target<'input>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        self.run_turn(input).await
    }
}

#[async_trait]
impl<I, O> SessionRun<I, O> for AnyAgentSession<I, O>
where
    I: InputCtor,
    O: OutputCtor,
{
    async fn run_turn<'input>(
        &mut self,
        input: I::Target<'input>,
    ) -> Result<O::Target<'input>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        self.run_turn(input).await
    }
}

/// A strategy that the session type `R` can run a turn with.
///
/// Implemented for:
/// - [`AgentSession<L, ..>`] with [`Strategy<L, O>`]
/// - [`AnyAgentSession`] with [`UniversalStrategy`]
#[async_trait]
pub trait StrategyFor<R: ?Sized, I: InputCtor, O: OutputCtor>: Send {
    /// The strategy's output, i.e. [`Strategy::Output`] / [`UniversalStrategy::CommonOutput`].
    type Output: Send + Sync;

    /// Runs one turn of `session` with this strategy.
    async fn run_turn_on<'input>(
        self,
        session: &mut R,
        input: I::Target<'input>,
    ) -> Result<AgentOutput<O::Target<'input>, Self::Output>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>;
}

#[async_trait]
impl<L, I, O, S, A> StrategyFor<AgentSession<L, I, O, S>, I, O> for A
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
    A: Strategy<L, O>,
{
    type Output = <A as Strategy<L, O>>::Output;

    async fn run_turn_on<'input>(
        self,
        session: &mut AgentSession<L, I, O, S>,
        input: I::Target<'input>,
    ) -> Result<AgentOutput<O::Target<'input>, Self::Output>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        session.run_turn_with_strategy(self, input).await
    }
}

#[async_trait]
impl<I, O, A> StrategyFor<AnyAgentSession<I, O>, I, O> for A
where
    I: InputCtor,
    O: OutputCtor,
    A: UniversalStrategy<O>,
{
    type Output = <A as UniversalStrategy<O>>::CommonOutput;

    async fn run_turn_on<'input>(
        self,
        session: &mut AnyAgentSession<I, O>,
        input: I::Target<'input>,
    ) -> Result<AgentOutput<O::Target<'input>, Self::Output>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        session.run_turn_with_strategy(self, input).await
    }
}
