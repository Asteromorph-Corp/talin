//! A module for AI agents and related components.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::chain::{
    Chain, ChainError, GetPrompt, InputCtor, OutputCtor, PartialWithInput, StrCtor, StringCtor,
};
use crate::llm::{GenericChat, Llm, UnsupportedContent};
use crate::mcp::McpServerSet;
use crate::schemas::{Prompt, WithUsage};
use crate::skill::SkillSet;
use crate::strategy::{Strategy, UsageTrackStrategy};
use crate::template::{PromptTemplate, TemplateError};
use crate::tools::Tool;

mod agent_output;
mod builder;
mod error;
mod helper;
mod inventory;
mod options;
mod run;
mod runner;
mod session;

pub use agent_output::AgentOutput;
pub use builder::{AgentBuilder, SpecializedAgentBuilder};
pub use error::AgentError;
pub use options::AgentOptions;
pub use run::{AgentRun, SessionRun, StrategyFor};
pub use runner::{AgentRunner, PendingStep};
pub use session::AgentSession;

pub use crate::any::{AnyAgent, AnyAgentSession};

/// An LLM agent that iteratively plan / execute tool actions until producing a valid final answer.
///
/// `Agent` is a stateless configuration: it holds the prompt template, LLM, tools, and options
/// used to construct an [`AgentSession`]. To run a turn, call [`Agent::session`] to mint a fresh
/// session and then [`AgentSession::turn`], or call [`Agent::run`] for a one-shot convenience.
///
/// # Type Parameters
/// - `L`: The LLM driver, implementing [`Llm`].
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type (defaults to [`StrCtor`],
///   which constructs `&str`).
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type (defaults to
///   [`StringCtor`], which constructs `String`).
/// - `S`: The default [`strategy`](Strategy) composed into every turn.
pub struct Agent<L, I = StrCtor, O = StringCtor, S = ()>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
{
    /// A unique identifier for the agent. Used for logging.
    pub(super) id: String,
    /// The prompt template for the agent.
    pub(super) prompt: PromptTemplate,
    /// The LLM used by the agent.
    pub(super) llm: L,
    /// A map of registered tool names to their implementations.
    pub(crate) tools: HashMap<String, Tool>,
    /// A list of built-in tools provided by the LLM driver.
    pub(crate) builtin_tools: Vec<L::BuiltinTool>,
    /// The skills the model can load on demand.
    pub(crate) skills: SkillSet,
    /// A map of MCP server names to their configurations.
    pub(crate) mcp_servers: McpServerSet,
    /// The default [`strategy`](Strategy) composed into every turn.
    ///
    /// This is intended for strategies that modify the agent's behavior without producing a direct
    /// output, such as logging or usage tracking. The output of this strategy is ignored.
    pub(super) strategy: S,
    // /// A list of toolboxes used to dynamically provide tools at runtime.
    // pub(crate) toolboxes: cVec<Arc<dyn Toolbox>>,
    /// The default options for sessions spawned from this agent.
    pub(super) options: AgentOptions,
    pub(super) _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> Agent<GenericChat, I, O>
where
    I: InputCtor,
    O: OutputCtor,
{
    /// Creates a [`AgentBuilder`] to configure an [`Agent`].
    ///
    /// This is the same as calling [`AgentBuilder::new()`].
    pub fn builder() -> AgentBuilder<I, O> {
        AgentBuilder::new()
    }
}

impl<L, I, O, S> Agent<L, I, O, S>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
{
    /// Returns the unique identifier of the agent.
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Sets the agent options.
    pub fn with_options(mut self, options: AgentOptions) -> Self {
        self.options = options;
        self
    }

    /// Maps the agent's strategy to a new strategy by applying a function `f`.
    ///
    /// Other configurations remain unchanged.
    pub(crate) fn map_strategy<S1>(self, f: impl FnOnce(S) -> S1) -> Agent<L, I, O, S1>
    where
        S1: Strategy<L, O> + Clone,
    {
        Agent {
            id: self.id,
            prompt: self.prompt,
            llm: self.llm,
            tools: self.tools,
            builtin_tools: self.builtin_tools,
            skills: self.skills,
            mcp_servers: self.mcp_servers,
            strategy: f(self.strategy),
            options: self.options,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Begins a fresh [`AgentSession`] using the agent's configuration.
    pub fn session(&self) -> AgentSession<L, I, O, S> {
        AgentSession::begin_with(self)
    }

    /// Continues a prior session under this agent, carrying over its transcript while adopting this
    /// agent's LLM, prompt, tools, and options.
    pub fn continue_session<I1, O1, S1>(
        &self,
        prev: AgentSession<L, I1, O1, S1>,
    ) -> AgentSession<L, I, O, S>
    where
        I1: InputCtor,
        O1: OutputCtor,
        S1: Strategy<L, O1> + Clone,
    {
        prev.continue_with(self)
    }

    /// Continues a prior session under this agent, migrating its transcript to this agent's LLM
    /// provider while adopting this agent's prompt, tools, and options.
    ///
    /// See [`AgentSession::continue_migrated_with`] for caveats on transcript migration.
    pub fn continue_session_migrated<L1, I1, O1, S1>(
        &self,
        prev: AgentSession<L1, I1, O1, S1>,
    ) -> Result<AgentSession<L, I, O, S>, UnsupportedContent>
    where
        L1: Llm,
        I1: InputCtor,
        O1: OutputCtor,
        S1: Strategy<L1, O1> + Clone,
    {
        prev.continue_migrated_with(self)
    }

    /// One-shot convenience: spawns a session, begins a turn with `input`, and runs to completion
    /// using the default strategy.
    pub async fn run<'input>(
        &self,
        input: I::Target<'input>,
    ) -> Result<O::Target<'input>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        let output = self.run_with_strategy((), input).await?;
        Ok(output.content)
    }

    /// One-shot run with a custom [`Strategy`]. See [`Agent::run`] for the default-strategy form.
    pub async fn run_with_strategy<'input, A>(
        &self,
        strategy: A,
        input: I::Target<'input>,
    ) -> Result<AgentOutput<O::Target<'input>, A::Output>, AgentError>
    where
        A: Strategy<L, O>,
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        let mut session = self.session();
        session.run_turn_with_strategy(strategy, input).await
    }
}

#[async_trait]
impl<L, I, O, S> Chain<I, O> for Agent<L, I, O, S>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
    for<'any> I::Target<'any>: std::fmt::Display,
    for<'any> O::Partial: PartialWithInput<I::Target<'any>, O::Target<'any>>,
{
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let output = self
            .run_with_strategy(UsageTrackStrategy::default(), input)
            .await?
            .into_with_usage();
        Ok(output)
    }
}

impl<L, I, O, S> GetPrompt<I::Target<'_>> for Agent<L, I, O, S>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
{
    fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, TemplateError> {
        self.prompt.format(&input)
    }
}
