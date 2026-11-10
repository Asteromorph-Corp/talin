use std::collections::HashMap;

use crate::agent::helper::use_strategy;
use crate::agent::inventory::Inventory;
use crate::agent::{Agent, AgentError, AgentOptions, AgentOutput, AgentRunner};
use crate::chain::{GetPrompt, InputCtor, OutputCtor, PartialWithInput};
use crate::llm::{Llm, UnsupportedContent};
use crate::mcp::McpServerSet;
use crate::schemas::{Message, Prompt};
use crate::skill::SkillSet;
use crate::strategy::{CompactionStrategy, Strategy};
use crate::template::{PromptTemplate, TemplateError};
use crate::tools::{Tool, ToolResolver};
use crate::transcript::scope::{Anchor, Last};
use crate::transcript::{AgentTranscript, Entry, Scope};

/// A persistent conversation between an agent and the LLM.
///
/// An `AgentSession` is created by a [`Agent::session`](crate::agent::Agent::session) call, and
/// owns the state of the conversation including the system prompt, transcript, and available
/// tools.
/// You can mutate these fields at any point during the session.
///
/// You can call [`turn`](Self::turn) to start a new turn with a given input and strategy.
/// This formats the prompt template, appends the resulting messages to the transcript, and
/// returns a transient [`AgentRunner`] handle.
/// Then, you can either call [`AgentRunner::run_to_finish`] to advance the turn to completion, or
/// manually step through the LLM-tool loop with [`AgentRunner::step`] for finer-grained control
/// and interleaving of custom logic.
pub struct AgentSession<L, I, O, S>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O>,
{
    /// The LLM to use for the session.
    pub llm: L,
    /// The template used to render input into messages each [`turn`](Self::turn) call.
    pub prompt: PromptTemplate,

    /// The transcript for the current session, containing all messages and items exchanged with
    /// the LLM.
    ///
    /// Use [`AgentTranscript::modify`](AgentTranscript::modify) and the other methods on
    /// [`AgentTranscript`] to edit the transcript mid-session.
    pub transcript: AgentTranscript<L::System, L::Item>,

    /// Optional history compaction policy, run before each generation request.
    /// Original transcript entries and their UUIDs are preserved.
    pub compaction: Option<Box<dyn CompactionStrategy<L>>>,

    /// The map of registered tool names to their implementations.
    pub tools: HashMap<String, Tool>,
    /// The list of built-in tools provided by the LLM driver.
    pub builtin_tools: Vec<L::BuiltinTool>,
    /// The skills the model can load on demand.
    pub skills: SkillSet,
    /// The map of MCP server names to their configurations.
    pub mcp_servers: McpServerSet,
    /// The session inventory of MCP services and clients, and the skill catalog.
    pub(super) inventory: Inventory,

    /// The default strategy composed into every [`turn`](Self::turn) of the session.
    ///
    /// This is intended for side-effecting strategies that modify the agent's behavior without
    /// producing a retrievable output, such as logging or usage tracking — it is re-cloned fresh
    /// for each turn and its [`finalize`](crate::strategy::Strategy::finalize) output is
    /// discarded. Use [`turn`](Self::turn) or [`Agent::run_with_strategy`] to specify a
    /// strategy on a per-run basis, which allows for strategies that produce outputs.
    ///
    /// Kept private: editing it would not affect an in-flight turn (which drives an
    /// already-composed clone), so exposing it would only mislead. Configure the default via
    /// the agent builder.
    pub(super) strategy: S,
    /// The session option.
    pub options: AgentOptions,

    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<L, I, O, S> AgentSession<L, I, O, S>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
{
    /// Swaps the default strategy, keeping everything else.
    pub(crate) fn map_strategy<S1>(self, f: impl FnOnce(S) -> S1) -> AgentSession<L, I, O, S1>
    where
        S1: Strategy<L, O> + Clone,
    {
        AgentSession {
            llm: self.llm,
            prompt: self.prompt,
            transcript: self.transcript,
            compaction: self.compaction,
            tools: self.tools,
            builtin_tools: self.builtin_tools,
            skills: self.skills,
            mcp_servers: self.mcp_servers,
            inventory: self.inventory,
            strategy: f(self.strategy),
            options: self.options,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Begins a new session from the given [`Agent`], cloning its LLM, prompt, tools, strategy
    /// and options.
    pub fn begin_with(agent: &Agent<L, I, O, S>) -> Self {
        Self {
            llm: agent.llm.clone(),
            prompt: agent.prompt.clone(),
            transcript: AgentTranscript::default(),
            compaction: None,
            tools: agent.tools.clone(),
            builtin_tools: agent.builtin_tools.clone(),
            skills: agent.skills.clone(),
            mcp_servers: agent.mcp_servers.clone(),
            inventory: Inventory::default(),
            strategy: agent.strategy.clone(),
            options: agent.options,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Continues the session with a different agent configuration but same LLM provider.
    pub fn continue_with<I2, O2, S2>(
        self,
        agent: &Agent<L, I2, O2, S2>,
    ) -> AgentSession<L, I2, O2, S2>
    where
        I2: InputCtor,
        O2: OutputCtor,
        S2: Strategy<L, O2> + Clone,
    {
        AgentSession {
            llm: agent.llm.clone(),
            prompt: agent.prompt.clone(),
            transcript: self.transcript,
            compaction: self.compaction,
            tools: agent.tools.clone(),
            builtin_tools: agent.builtin_tools.clone(),
            skills: agent.skills.clone(),
            mcp_servers: agent.mcp_servers.clone(),
            inventory: Inventory::default(),
            strategy: agent.strategy.clone(),
            options: agent.options,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Continues the session with a different agent configuration and a different LLM provider,
    /// migrating the transcript along the way. The provider-specific compaction policy is cleared.
    ///
    /// The transcript conversion is not perfect. It drops any provider-specific fields that cannot
    /// be expressed in [`BodyMessage`](crate::schemas::BodyMessage). Furthermore, if the model
    /// requires certain format for message ids or tool call ids, the migrated transcript may not be
    /// fully compatible with the new model. You may need to edit the transcript manually to fix
    /// the inconsistency.
    pub fn continue_migrated_with<L2, I2, O2, S2>(
        self,
        agent: &Agent<L2, I2, O2, S2>,
    ) -> Result<AgentSession<L2, I2, O2, S2>, UnsupportedContent>
    where
        L2: Llm,
        I2: InputCtor,
        O2: OutputCtor,
        S2: Strategy<L2, O2> + Clone,
    {
        Ok(AgentSession {
            llm: agent.llm.clone(),
            prompt: agent.prompt.clone(),
            transcript: self.transcript.migrate::<L2::System, L2::Item>()?,
            compaction: None,
            tools: agent.tools.clone(),
            builtin_tools: agent.builtin_tools.clone(),
            skills: agent.skills.clone(),
            mcp_servers: agent.mcp_servers.clone(),
            inventory: Inventory::default(),
            strategy: agent.strategy.clone(),
            options: agent.options,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Configures history compaction for this session.
    pub fn with_compaction(mut self, compaction: impl CompactionStrategy<L> + 'static) -> Self {
        self.compaction = Some(Box::new(compaction));
        self
    }

    /// Applies the option to the session.
    pub fn with_options(mut self, options: AgentOptions) -> Self {
        self.options = options;
        self
    }

    /// Rolls the transcript back to the given anchor, discarding everything after it.
    pub fn rollback(&mut self, anchor: &dyn Anchor) {
        self.transcript.rollback(anchor);
    }

    /// Returns a [`ToolResolver`] that can be used to resolve tool calls in the session.
    pub fn tool_resolver(&self) -> impl ToolResolver<'_> {
        (&self.tools, &self.inventory)
    }

    /// Renders the prompt template with `input`, appends the resulting body messages to the
    /// transcript, and returns an [`AgentRunner`].
    ///
    /// Any unresolved tool calls from a previous turn are dropped before the new user message is
    /// appended.
    pub async fn turn<'session, 'input, A>(
        &'session mut self,
        strategy: A,
        input: I::Target<'input>,
    ) -> Result<AgentRunner<'session, 'input, L, I, O, S, A>, AgentError>
    where
        A: Strategy<L, O>,
    {
        let mut strategy = (self.strategy.clone(), strategy);

        self.transcript
            .remove_unresolved_tool_calls(&Scope::from(Last::step()))?;

        let messages = self.prompt.format(&input)?.to_messages();
        let messages = use_strategy!(strategy, process_initial_messages(messages))?;
        if log::log_enabled!(log::Level::Debug) {
            for message in &messages {
                log::debug!("\n{message}");
            }
        }
        let (system_contents, body) = Message::partition(messages);

        if !system_contents.is_empty() {
            self.transcript.set_system(system_contents);
        }
        self.transcript.mark_turn_start();
        for message in body {
            self.transcript.push(Entry::try_from(message)?);
        }

        Ok(AgentRunner::new(self, input, strategy))
    }

    /// Runs one turn to completion with the default strategy.
    pub async fn run_turn<'input>(
        &mut self,
        input: I::Target<'input>,
    ) -> Result<O::Target<'input>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        let output = self.run_turn_with_strategy((), input).await?;
        Ok(output.content)
    }

    /// Runs one turn to completion with a custom [`Strategy`].
    pub async fn run_turn_with_strategy<'input, ST>(
        &mut self,
        strategy: ST,
        input: I::Target<'input>,
    ) -> Result<AgentOutput<O::Target<'input>, ST::Output>, AgentError>
    where
        ST: Strategy<L, O>,
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        let output = self.turn(strategy, input).await?.run_to_finish().await?;
        // Discard the default strategy's output (`.0`); only the per-turn strategy's output is
        // surfaced to the caller.
        Ok(AgentOutput::new(output.content, output.extra_content))
    }

    /// Syncs the session's inventory of MCP servers and skills.
    ///
    /// The session manages an internal inventory of MCP clients and skills, which is used to
    /// resolve tool calls and skill invocations.
    ///
    /// The method is called automatically at the start of each turn.
    pub async fn sync_inventory(&mut self) -> Result<(), AgentError> {
        self.inventory
            .sync(
                &self.mcp_servers,
                &self.skills,
                self.llm.supports_mcp(),
                None,
            )
            .await
    }
}

/// Renders the session's *current* prompt template without advancing a turn or mutating the
/// transcript. Mirrors [`Agent`]'s implementation so that the same `get_prompt_immed` extension
/// method works against either type; reflects any edits made to [`AgentSession::prompt`].
impl<L, I, O, S> GetPrompt<I::Target<'_>> for AgentSession<L, I, O, S>
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
