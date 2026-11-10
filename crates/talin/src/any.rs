//! Models and agents whose [`Llm`](crate::llm::Llm) is chosen at runtime, rather than at compile
//! time.
//!
//! Different model backends produce different concrete agent types (`Agent<OpenAIResponses>` vs.
//! `Agent<ClaudeMessages>`), so they cannot be chosen from the same runtime branch without some
//! form of type erasure.
//! `AnyPreset`, `AnyAgent` and `AnyAgentSession` enums provide that erasure with a single variant
//! per backend. The agent enums also erases the agent's default strategy type with a
//! [`BoxedStrategy`].
//!
//! This simplifies runtime model selection as follows:
//! ```ignore
//! let preset: AnyPreset = match name {
//!     "openai" => Models::openai().into(),
//!     "openai-unstrict" => Models::openai_unstrict().into(),
//!     _ => unreachable!(),
//! };
//! let agent = GuesserAgent::any(preset).await;
//! ```
//!
//! The strategy used for `AnyAgent::run_with_strategy` and
//! `AnyAgentSession::run_turn_with_strategy` must be usable with *all* backends (i.e.
//! [`UniversalStrategy`]).

use thiserror::Error;

use crate::agent::{Agent, AgentError, AgentOutput, AgentSession};
use crate::chain::{GetPrompt, InputCtor, OutputCtor, PartialWithInput, StringCtor};
use crate::llm::{
    ClaudeMessages, GeminiGenerateContent, GeminiInteractions, GenericChat, LlmPreset, OpenAIChat,
    OpenAIResponses, SglangGenerate, UnsupportedContent, XaiResponses,
};
use crate::schemas::Prompt;
use crate::strategy::{BoxedStrategy, Strategy};
use crate::template::TemplateError;

/// Declares the backend list once and derives everything that has to enumerate it:
/// - `AnyPreset`, `AnyAgent`, and `AnyAgentSession` enums with one variant per backend.
/// - `From` conversions from each concrete type into the corresponding enum.
/// - [`AnyAgent`] and [`AnyAgentSession`] dispatching methods that call the corresponding method on
///   the concrete type.
/// - [`UniversalStrategy`] trait and blanket impl for any type that implements every backend's
///   [`Strategy`].
macro_rules! define_any_agent {
    ($($(#[$meta:meta])* $variant:ident($llm:ty)),+ $(,)?) => {
        /// A model preset whose [`Llm`](crate::llm::Llm) is chosen at runtime.
        ///
        /// It is independent of any agent's output type, so a single preset can be cloned and
        /// reused across agents and requests.
        #[derive(Clone)]
        #[non_exhaustive]
        pub enum AnyPreset {
            $($(#[$meta])* $variant(Box<LlmPreset<$llm, BoxedStrategy<$llm>>>),)+
        }

        /// An [`Agent`] whose [`Llm`](crate::llm::Llm) is chosen at runtime.
        #[non_exhaustive]
        pub enum AnyAgent<I = $crate::chain::StrCtor, O = $crate::chain::StringCtor>
        where
            I: InputCtor,
            O: OutputCtor,
        {
            $($(#[$meta])* $variant(Agent<$llm, I, O, BoxedStrategy<$llm>>),)+
        }

        /// An [`AgentSession`] whose [`Llm`](crate::llm::Llm) is chosen at runtime.
        #[non_exhaustive]
        pub enum AnyAgentSession<I = $crate::chain::StrCtor, O = $crate::chain::StringCtor>
        where
            I: InputCtor,
            O: OutputCtor,
        {
            $($(#[$meta])* $variant(AgentSession<$llm, I, O, BoxedStrategy<$llm>>),)+
        }

        $(
            impl<S> From<LlmPreset<$llm, S>> for AnyPreset
            where
                S: Strategy<$llm, StringCtor> + Clone + 'static,
            {
                fn from(preset: LlmPreset<$llm, S>) -> Self {
                    Self::$variant(Box::new(LlmPreset {
                        model: preset.model,
                        strategy: BoxedStrategy::new(preset.strategy),
                        options: preset.options,
                    }))
                }
            }

            impl<I, O, S> From<Agent<$llm, I, O, S>> for AnyAgent<I, O>
            where
                I: InputCtor,
                O: OutputCtor,
                S: Strategy<$llm, O> + Strategy<$llm, StringCtor> + Clone + 'static,
            {
                fn from(agent: Agent<$llm, I, O, S>) -> Self {
                    Self::$variant(agent.map_strategy(BoxedStrategy::new))
                }
            }

            impl<I, O, S> From<AgentSession<$llm, I, O, S>> for AnyAgentSession<I, O>
            where
                I: InputCtor,
                O: OutputCtor,
                S: Strategy<$llm, O> + Strategy<$llm, StringCtor> + Clone + 'static,
            {
                fn from(session: AgentSession<$llm, I, O, S>) -> Self {
                    Self::$variant(session.map_strategy(BoxedStrategy::new))
                }
            }
        )+

        /// A [`Strategy`] that can be used with every built-in backend.
        pub trait UniversalStrategy<O: OutputCtor>:
            $(Strategy<$llm, O, Output = Self::CommonOutput> +)+
        {
            /// The [`Strategy::Output`] shared by every backend.
            type CommonOutput: Send + Sync;
        }

        impl<T, O, X> UniversalStrategy<O> for T
        where
            O: OutputCtor,
            X: Send + Sync,
            T: $(Strategy<$llm, O, Output = X> +)+
        {
            type CommonOutput = X;
        }


        impl<I, O> AnyAgent<I, O>
        where
            I: InputCtor,
            O: OutputCtor,
        {
            /// The name of the backend this agent runs on, e.g. `"ClaudeMessages"`.
            pub fn backend(&self) -> &'static str {
                match self { $(Self::$variant(_) => stringify!($variant),)+ }
            }

            /// Returns the unique identifier of the agent.
            pub fn id(&self) -> &str {
                match self { $(Self::$variant(agent) => agent.id(),)+ }
            }

            /// Begins a fresh [`AnyAgentSession`] using the agent's configuration.
            pub fn session(&self) -> AnyAgentSession<I, O> {
                match self { $(Self::$variant(agent) => agent.session().into(),)+ }
            }

            /// Continues `prev` under this agent, keeping its transcript and compaction policy
            /// while adopting this agent's prompt, tools and options. See
            /// [`AnyAgentSession::continue_with`].
            pub fn continue_session<I1, O1>(
                &self,
                prev: AnyAgentSession<I1, O1>,
            ) -> Result<AnyAgentSession<I, O>, BackendMismatch<I1, O1>>
            where
                I1: InputCtor,
                O1: OutputCtor,
            {
                prev.continue_with(self)
            }

            /// Continues `prev` under this agent, migrating the transcript when the backends
            /// differ. See [`AnyAgentSession::continue_migrated_with`].
            pub fn continue_session_migrated<I1, O1>(
                &self,
                prev: AnyAgentSession<I1, O1>,
            ) -> Result<AnyAgentSession<I, O>, UnsupportedContent>
            where
                I1: InputCtor,
                O1: OutputCtor,
            {
                prev.continue_migrated_with(self)
            }

            /// One-shot convenience: see [`Agent::run`].
            pub async fn run<'input>(
                &self,
                input: I::Target<'input>,
            ) -> Result<O::Target<'input>, AgentError>
            where
                O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
            {
                match self { $(Self::$variant(agent) => agent.run(input).await,)+ }
            }

            /// One-shot run with a custom strategy: see [`Agent::run_with_strategy`].
            pub async fn run_with_strategy<'input, A>(
                &self,
                strategy: A,
                input: I::Target<'input>,
            ) -> Result<AgentOutput<O::Target<'input>, A::CommonOutput>, AgentError>
            where
                A: UniversalStrategy<O>,
                O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
            {
                match self {
                    $(Self::$variant(agent) => agent.run_with_strategy(strategy, input).await,)+
                }
            }
        }

        impl<I, O> GetPrompt<I::Target<'_>> for AnyAgent<I, O>
        where
            I: InputCtor,
            O: OutputCtor,
        {
            fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, TemplateError> {
                match self { $(Self::$variant(agent) => agent.get_prompt(input),)+ }
            }
        }

        impl<I, O> AnyAgentSession<I, O>
        where
            I: InputCtor,
            O: OutputCtor,
        {
            /// The name of the backend this session runs on, e.g. `"ClaudeMessages"`.
            pub fn backend(&self) -> &'static str {
                match self { $(Self::$variant(_) => stringify!($variant),)+ }
            }

            /// Continues the session under `agent`.
            ///
            /// Requires the `agent` to use same backend as the session. Unlike [`AgentSession::continue_with`],
            /// this check happens at runtime, and the method fails if the backends differ.
            /// Use [`continue_migrated_with`](Self::continue_migrated_with) to cross backends.
            ///
            /// See [`AgentSession::continue_with`].
            pub fn continue_with<I2, O2>(
                self,
                agent: &AnyAgent<I2, O2>,
            ) -> Result<AnyAgentSession<I2, O2>, BackendMismatch<I, O>>
            where
                I2: InputCtor,
                O2: OutputCtor,
            {
                match (self, agent) {
                    $((Self::$variant(session), AnyAgent::$variant(agent)) => {
                        Ok(AnyAgentSession::$variant(session.continue_with(agent)))
                    })+
                    #[allow(unreachable_patterns)]
                    (session, agent) => Err(BackendMismatch {
                        agent_backend: agent.backend(),
                        session: Box::new(session),
                    }),
                }
            }

            /// Continues the session under `agent`.
            ///
            /// If the `agent` uses the same backend as the session, this method is identical to
            /// [`continue_with`](Self::continue_with).
            /// When they differ, the transcript is migrated and the compaction policy is cleared.
            ///
            /// See [`AgentSession::continue_migrated_with`].
            pub fn continue_migrated_with<I2, O2>(
                self,
                agent: &AnyAgent<I2, O2>,
            ) -> Result<AnyAgentSession<I2, O2>, UnsupportedContent>
            where
                I2: InputCtor,
                O2: OutputCtor,
            {
                match self.continue_with(agent) {
                    Ok(session) => Ok(session),
                    Err(mismatch) => {
                        let session = mismatch.session;
                        match agent {
                            $(AnyAgent::$variant(agent) => session
                                .migrate_to(agent)
                                .map(AnyAgentSession::$variant),)+
                        }
                    }
                }
            }

            /// Migrates the transcript into a session for `agent`, regardless of backend.
            fn migrate_to<L2, I2, O2, S2>(
                self,
                agent: &Agent<L2, I2, O2, S2>,
            ) -> Result<AgentSession<L2, I2, O2, S2>, UnsupportedContent>
            where
                L2: crate::llm::Llm,
                I2: InputCtor,
                O2: OutputCtor,
                S2: Strategy<L2, O2> + Clone,
            {
                match self {
                    $(Self::$variant(session) => session.continue_migrated_with(agent),)+
                }
            }

            /// Runs one turn to completion with the default strategy: see
            /// [`AgentSession::run_turn`].
            pub async fn run_turn<'input>(
                &mut self,
                input: I::Target<'input>,
            ) -> Result<O::Target<'input>, AgentError>
            where
                O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
            {
                match self { $(Self::$variant(session) => session.run_turn(input).await,)+ }
            }

            /// Runs one turn to completion with a custom strategy: see
            /// [`AgentSession::run_turn_with_strategy`].
            pub async fn run_turn_with_strategy<'input, A>(
                &mut self,
                strategy: A,
                input: I::Target<'input>,
            ) -> Result<AgentOutput<O::Target<'input>, A::CommonOutput>, AgentError>
            where
                A: UniversalStrategy<O>,
                O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
            {
                match self {
                    $(Self::$variant(session) => {
                        session.run_turn_with_strategy(strategy, input).await
                    })+
                }
            }
        }

        impl<I, O> GetPrompt<I::Target<'_>> for AnyAgentSession<I, O>
        where
            I: InputCtor,
            O: OutputCtor,
        {
            fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, TemplateError> {
                match self { $(Self::$variant(session) => session.get_prompt(input),)+ }
            }
        }
    };
}

/// The error returned by [`AnyAgentSession::continue_with`] when the agent runs on a different
/// backend than the session. It gives the untouched session back.
pub struct BackendMismatch<I = crate::chain::StrCtor, O = StringCtor>
where
    I: InputCtor,
    O: OutputCtor,
{
    /// The session that could not be continued.
    pub session: Box<AnyAgentSession<I, O>>,
    agent_backend: &'static str,
}

impl<I: InputCtor, O: OutputCtor> BackendMismatch<I, O> {
    /// The backend of the agent that was supplied.
    pub fn agent_backend(&self) -> &'static str {
        self.agent_backend
    }
}

impl<I: InputCtor, O: OutputCtor> std::fmt::Debug for BackendMismatch<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendMismatch")
            .field("session_backend", &self.session.backend())
            .field("agent_backend", &self.agent_backend)
            .finish()
    }
}

impl<I: InputCtor, O: OutputCtor> std::fmt::Display for BackendMismatch<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot continue a `{}` session with a `{}` agent without migrating; \
             use `continue_migrated_with`",
            self.session.backend(),
            self.agent_backend
        )
    }
}

impl<I: InputCtor, O: OutputCtor> std::error::Error for BackendMismatch<I, O> {}

/// The error returned when a model name does not match any configured model, such as by the
/// generated `Models::any`.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("unknown model `{name}`; available: {available:?}")]
pub struct UnknownModel {
    name: String,
    available: &'static [&'static str],
}

impl UnknownModel {
    /// Creates a new [`UnknownModel`] for `name`, given the names that would have been accepted.
    pub fn new(name: impl Into<String>, available: &'static [&'static str]) -> Self {
        Self {
            name: name.into(),
            available,
        }
    }

    /// The unrecognized name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The names that would have been accepted.
    pub fn available(&self) -> &'static [&'static str] {
        self.available
    }
}

define_any_agent! {
    /// An [`Agent`] backed by [`GenericChat`].
    GenericChat(GenericChat),
    /// An [`Agent`] backed by [`OpenAIChat`].
    OpenAIChat(OpenAIChat),
    /// An [`Agent`] backed by [`OpenAIResponses`].
    OpenAIResponses(OpenAIResponses),
    /// An [`Agent`] backed by [`ClaudeMessages`].
    ClaudeMessages(ClaudeMessages),
    /// An [`Agent`] backed by [`GeminiGenerateContent`].
    GeminiGenerateContent(GeminiGenerateContent),
    /// An [`Agent`] backed by [`GeminiInteractions`].
    GeminiInteractions(GeminiInteractions),
    /// An [`Agent`] backed by [`XaiResponses`].
    XaiResponses(XaiResponses),
    /// An [`Agent`] backed by [`SglangGenerate`].
    SglangGenerate(SglangGenerate),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::StringCtor;
    use crate::schemas::Usage;
    use crate::strategy::{TagStrategy, UsageTrackStrategy};

    fn assert_universal<S: UniversalStrategy<StringCtor>>() {}

    fn common_output<S, O>()
    where
        S: UniversalStrategy<StringCtor, CommonOutput = O>,
    {
    }

    fn assert_conversion<A, B>()
    where
        A: Into<B>,
    {
    }

    #[test]
    fn shipped_strategies_are_universal() {
        assert_universal::<()>();
        assert_universal::<UsageTrackStrategy>();
        assert_universal::<TagStrategy>();
        assert_universal::<(UsageTrackStrategy, TagStrategy)>();
    }

    #[test]
    fn common_output_is_the_strategy_output() {
        common_output::<UsageTrackStrategy, Usage>();
    }

    #[test]
    fn sessions_can_be_continued_by_any_agent() {
        // Compile-time check of the continuation signatures.
        fn _check(session: AnyAgentSession, agent: &AnyAgent) {
            session.continue_with(agent).ok();
        }
        fn _check_migrated(session: AnyAgentSession, agent: &AnyAgent) {
            session.continue_migrated_with(agent).ok();
        }
        fn _check_agent(session: AnyAgentSession, agent: &AnyAgent) {
            agent.continue_session(session).ok();
        }
    }

    #[test]
    fn converts_from_a_concrete_agent() {
        assert_conversion::<Agent<GenericChat>, AnyAgent>();
        assert_conversion::<Agent<OpenAIChat>, AnyAgent>();
        assert_conversion::<Agent<OpenAIResponses>, AnyAgent>();
        assert_conversion::<Agent<ClaudeMessages>, AnyAgent>();
        assert_conversion::<Agent<GeminiGenerateContent>, AnyAgent>();
        assert_conversion::<Agent<GeminiInteractions>, AnyAgent>();
        assert_conversion::<Agent<SglangGenerate>, AnyAgent>();
    }
}
