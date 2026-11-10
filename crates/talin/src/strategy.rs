//! A module for pluggable agent execution strategies.
//!
//! See the [`Strategy` trait documentation](crate::strategy::Strategy) for details.

use std::convert::Infallible;
use std::error::Error;

use async_trait::async_trait;

use crate::chain::OutputCtor;
use crate::llm::{Llm, LlmRequestOptions};
use crate::mcp::McpServerSet;
use crate::schemas::{ContentPart, LlmStep, Message, ToolResult, ToolResultMessage, Usage};
use crate::skill::SkillSet;
use crate::tools::ToolResolver;
use crate::transcript::AgentTranscript;

mod apply_edit_strategy;
mod boxed_strategy;
#[cfg(feature = "buffered-final-answer")]
pub mod buffered_final_answer_strategy;
mod compaction;
mod drop_thought_strategy;
mod final_answer_tool_strategy;
mod record_strategy;
mod remove_strict_strategy;
mod system_to_assistant_strategy;
mod tag_strategy;
mod tool_resolver_strategy;
mod tool_strategy;
pub mod usage_track_strategy;

pub use apply_edit_strategy::{
    ApplyEditsStrategy, ApplyEntryEditStrategy, ApplyEntryProjectionStrategy,
    ApplyProjectionsStrategy, ApplySystemEditStrategy, ApplySystemProjectionStrategy,
};
pub use boxed_strategy::{BoxedStrategy, BoxedStrategyError};
#[cfg(feature = "buffered-final-answer")]
pub use buffered_final_answer_strategy::BufferedFinalAnswerStrategy;
pub use compaction::{
    AutoCompactStrategy, ClaudeMessagesAutoCompactStrategy, ClaudeMessagesCompactStrategy,
    ClaudeMessagesCompactor, CompactionCutoff, CompactionModel, CompactionSelector,
    CompactionStrategy, Compactor, CustomCompactStrategy, InstructionCompactor, MessageCount,
    OpenAIResponsesAutoCompactStrategy, OpenAIResponsesCompactStrategy, OpenAIResponsesCompactor,
    RequestParts, TokenBudget,
};
pub use drop_thought_strategy::DropThoughtStrategy;
pub use final_answer_tool_strategy::FinalAnswerToolStrategy;
pub use record_strategy::RecordStrategy;
pub use remove_strict_strategy::RemoveStrictStrategy;
pub use system_to_assistant_strategy::SystemToAssistantStrategy;
pub use tag_strategy::{SharedTagStrategy, TagStrategy};
pub use tool_resolver_strategy::ToolResolverStrategy;
pub use tool_strategy::ToolStrategy;
pub use usage_track_strategy::{SharedUsage, SharedUsageTrackStrategy, UsageTrackStrategy};

pub use crate::any::UniversalStrategy;

/// A pluggable policy that customizes **how an agent run is executed**.
///
/// A `Strategy` intercepts the major phases of an
/// [`AgentRunner`](crate::agent::AgentRunner) run — injecting tools and options, then inspecting
/// or rewriting the data that flows through each phase — without touching the core loop.
///
/// Every hook defaults to a no-op, so an implementor overrides only the phases it cares about.
/// The hooks fall into three groups:
///
/// - **Injection** — [`additional_tools`], [`additional_skills`], [`additional_mcp_servers`],
///   [`additional_builtin_tools`], [`call_options`].
/// - **Provider-specific (native) hooks** — [`process_materialized`], [`process_request`],
///   [`process_response`]. These operate on the provider's own types, so they can reach fields the
///   provider-agnostic layer erases (cache breakpoints, thinking budgets, …) at the cost of
///   portability.
/// - **Provider-agnostic hooks** — [`process_initial_messages`], [`inspect_usage`],
///   [`process_llm_step`], [`process_tool_result`], [`process_final_answer`], [`process_output`],
///   [`finalize`].
///
/// [`additional_tools`]: Strategy::additional_tools
/// [`additional_skills`]: Strategy::additional_skills
/// [`additional_mcp_servers`]: Strategy::additional_mcp_servers
/// [`additional_builtin_tools`]: Strategy::additional_builtin_tools
/// [`call_options`]: Strategy::call_options
/// [`process_materialized`]: Strategy::process_materialized
/// [`process_request`]: Strategy::process_request
/// [`process_response`]: Strategy::process_response
/// [`process_initial_messages`]: Strategy::process_initial_messages
/// [`inspect_usage`]: Strategy::inspect_usage
/// [`process_llm_step`]: Strategy::process_llm_step
/// [`process_tool_result`]: Strategy::process_tool_result
/// [`process_final_answer`]: Strategy::process_final_answer
/// [`process_output`]: Strategy::process_output
/// [`finalize`]: Strategy::finalize
#[async_trait]
pub trait Strategy<L: Llm, O: OutputCtor>: Send + Sync {
    /// Type produced by [`finalize`](Strategy::finalize), used to return strategy-specific side
    /// artifacts (e.g. tag indices, telemetry, transcripts).
    type Output: Send + Sync;
    /// Error produced by this strategy's hooks.
    type Error: Error + Send + Sync + 'static;

    // --- Injection ---

    /// Extra tools available during the run, returned as a [`ToolResolver`].
    ///
    /// You can implement a [`ToolResolver`] trait for your own type, or return one of the types
    /// with built-in implementations, such as an array slice (`&[Tool]`) or a hashmap reference
    /// `&HashMap<String, Tool>`.
    fn additional_tools<'a>(&'a self) -> impl ToolResolver<'a> + 'a {}

    /// Extra skills available during the run, added to the session's.
    ///
    /// If a skill has the same name as one of the session's, this one is used.
    fn additional_skills(&self) -> Option<SkillSet> {
        None
    }

    /// Extra MCP servers available during the run, added to the session's.
    ///
    /// If a server has the same name as one of the session's, this one is used.
    fn additional_mcp_servers(&self) -> Option<McpServerSet> {
        None
    }

    /// Extra provider-specific built-in tools, appended to the session's.
    fn additional_builtin_tools(&self) -> Vec<L::BuiltinTool> {
        Vec::new()
    }

    /// The call options for the LLM.
    fn call_options(&self) -> L::Options {
        L::Options::from_default()
    }

    // --- Transcript hooks ---

    /// Permanently edit the stored transcript before it is materialized.
    ///
    /// Mutations here persist into the session transcript, so this is the home for *idempotent*
    /// structural edits — dropping a redundant tool call, renaming a call, merging or splitting
    /// messages.
    ///
    /// The edit is applied once per step on the real transcript. Since the edit is permanent, they
    /// must be idempotent, and should not remove information required for later steps.
    async fn edit_transcript(
        &mut self,
        _transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Project a throwaway clone of the transcript at materialize time.
    ///
    /// Mutations here are discarded after the request is built, so this is the home for *ephemeral*
    /// edits that depend on live state — rendering `{{key}}` placeholders against current values,
    /// or conditionally hiding entries.
    ///
    /// The projection is applied once per step on a clone of the transcript, so it can be used to
    /// compute ephemeral state for the request without mutating the real transcript.
    async fn project_transcript(
        &mut self,
        _transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    // --- Provider-specific (native) hooks ---

    /// Rewrite the materialized system and items before the request is built.
    ///
    /// You can use the [`SystemOf`](crate::llm::SystemOf) and [`ItemOf`](crate::llm::ItemOf) type
    /// aliases to access the provider-specific system and item types.
    ///
    /// `Err` retries by materializing again.
    async fn process_materialized(
        &mut self,
        system: L::System,
        items: Vec<L::Item>,
    ) -> Result<(L::System, Vec<L::Item>), Self::Error> {
        Ok((system, items))
    }

    /// Rewrite the provider-specific request before it is sent.
    ///
    /// You can use the [`RequestOf`](crate::llm::RequestOf) type alias to access the
    /// provider-specific request type.
    ///
    /// `Err` retries by building the request again.
    async fn process_request(&mut self, request: L::Request) -> Result<L::Request, Self::Error> {
        Ok(request)
    }

    /// Rewrite the provider-specific response before it is lowered into the transcript.
    ///
    /// You can use the [`ResponseOf`](crate::llm::ResponseOf) type alias to access the
    /// provider-specific response type.
    ///
    /// `Err` retries the request.
    async fn process_response(
        &mut self,
        response: L::Response,
    ) -> Result<L::Response, Self::Error> {
        Ok(response)
    }

    // --- Provider-agnostic hooks ---

    /// Rewrite the initial messages.
    ///
    /// `Err` fails the run immediately.
    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, Self::Error> {
        Ok(messages)
    }

    /// Observe the model usage for a step.
    async fn inspect_usage(&mut self, _usage: &Usage) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Rewrite the model-produced [`LlmStep`].
    ///
    /// `Err` retries the request.
    async fn process_llm_step(&mut self, step: LlmStep) -> Result<LlmStep, Self::Error> {
        Ok(step)
    }

    /// Rewrite a tool's [`ToolResult`].
    ///
    /// You can change a tool call failure into a success, or vice versa.
    ///
    /// `Err` fails only this tool call, while keeping the other calls in the step. The error
    /// message may be returned to the model as feedback.
    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, Self::Error> {
        Ok(message.result)
    }

    /// Rewrite the model's final answer before it is deserialized.
    ///
    /// `Err` retries the request.
    async fn process_final_answer(
        &mut self,
        parts: Vec<ContentPart>,
    ) -> Result<Vec<ContentPart>, Self::Error> {
        Ok(parts)
    }

    /// Rewrite the partial output parsed from the final answer.
    ///
    /// `Err` retries the request.
    async fn process_output(&mut self, output: O::Partial) -> Result<O::Partial, Self::Error> {
        Ok(output)
    }

    /// Consume the strategy and produce its [`Output`](Strategy::Output).
    ///
    /// `Err` fails the run; avoid unless strictly necessary.
    async fn finalize(self) -> Result<Self::Output, Self::Error>;
}

#[async_trait]
impl<L: Llm, O: OutputCtor> Strategy<L, O> for () {
    type Output = ();
    type Error = Infallible;

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

#[async_trait]
impl<S, L, O> Strategy<L, O> for Option<S>
where
    S: Strategy<L, O>,
    L: Llm,
    O: OutputCtor,
{
    type Output = Option<S::Output>;
    type Error = S::Error;

    fn additional_tools<'a>(&'a self) -> impl ToolResolver<'a> + 'a {
        self.as_ref().map(|strategy| strategy.additional_tools())
    }

    fn additional_skills(&self) -> Option<SkillSet> {
        self.as_ref()
            .and_then(|strategy| strategy.additional_skills())
    }

    fn additional_mcp_servers(&self) -> Option<McpServerSet> {
        self.as_ref()
            .and_then(|strategy| strategy.additional_mcp_servers())
    }

    fn additional_builtin_tools(&self) -> Vec<L::BuiltinTool> {
        if let Some(strategy) = self {
            strategy.additional_builtin_tools()
        } else {
            Vec::new()
        }
    }

    fn call_options(&self) -> L::Options {
        if let Some(strategy) = self {
            strategy.call_options()
        } else {
            L::Options::from_default()
        }
    }

    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, Self::Error> {
        if let Some(strategy) = self {
            strategy.process_initial_messages(messages).await
        } else {
            Ok(messages)
        }
    }

    async fn edit_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        if let Some(strategy) = self {
            strategy.edit_transcript(transcript).await?;
        }
        Ok(())
    }

    async fn project_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        if let Some(strategy) = self {
            strategy.project_transcript(transcript).await?;
        }
        Ok(())
    }

    async fn process_materialized(
        &mut self,
        system: L::System,
        items: Vec<L::Item>,
    ) -> Result<(L::System, Vec<L::Item>), Self::Error> {
        if let Some(strategy) = self {
            strategy.process_materialized(system, items).await
        } else {
            Ok((system, items))
        }
    }

    async fn process_request(&mut self, request: L::Request) -> Result<L::Request, Self::Error> {
        if let Some(strategy) = self {
            strategy.process_request(request).await
        } else {
            Ok(request)
        }
    }

    async fn process_response(
        &mut self,
        response: L::Response,
    ) -> Result<L::Response, Self::Error> {
        if let Some(strategy) = self {
            strategy.process_response(response).await
        } else {
            Ok(response)
        }
    }

    async fn inspect_usage(&mut self, _usage: &Usage) -> Result<(), Self::Error> {
        if let Some(strategy) = self {
            strategy.inspect_usage(_usage).await
        } else {
            Ok(())
        }
    }

    async fn process_llm_step(&mut self, output: LlmStep) -> Result<LlmStep, Self::Error> {
        if let Some(strategy) = self {
            strategy.process_llm_step(output).await
        } else {
            Ok(output)
        }
    }

    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, Self::Error> {
        if let Some(strategy) = self {
            strategy.process_tool_result(message).await
        } else {
            Ok(message.result)
        }
    }

    async fn process_final_answer(
        &mut self,
        content: Vec<ContentPart>,
    ) -> Result<Vec<ContentPart>, Self::Error> {
        if let Some(strategy) = self {
            strategy.process_final_answer(content).await
        } else {
            Ok(content)
        }
    }

    async fn process_output(&mut self, output: O::Partial) -> Result<O::Partial, Self::Error> {
        if let Some(strategy) = self {
            strategy.process_output(output).await
        } else {
            Ok(output)
        }
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        if let Some(strategy) = self {
            let output = strategy.finalize().await?;
            Ok(Some(output))
        } else {
            Ok(None)
        }
    }
}

macro_rules! impl_strategy_for_tuple {
    ( $( $S:ident ),+ ; $error:ident ) => {
        /// Error type for a tuple of strategies. Each variant corresponds to the strategy at that
        /// position in the tuple that produced the error.
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        pub enum $error< $( $S ),+ > {
            $(
                #[doc = concat!("Error produced by the strategy at position `", stringify!($S), "`.")]
                $S($S),
            )+
        }

        impl< $( $S: std::fmt::Display ),+ >
            std::fmt::Display for $error< $( $S ),+ >
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( Self::$S(e) => e.fmt(f), )+
                }
            }
        }

        impl< $( $S: std::error::Error + 'static ),+ >
            std::error::Error for $error< $( $S ),+ >
        {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    $( Self::$S(e) => e.source(), )+
                }
            }
        }

        #[::async_trait::async_trait]
        #[allow(non_snake_case)]
        impl<$ ($S,)+ L, O> $crate::strategy::Strategy<L, O> for ( $( $S ),+ )
        where
            $( $S: $crate::strategy::Strategy<L, O>, )+
            L: Llm,
            O: $crate::chain::OutputCtor,
        {
            type Output = ( $( <$S as $crate::strategy::Strategy<L, O>>::Output ),+ );
            type Error = $error<
                $( <$S as $crate::strategy::Strategy<L, O>>::Error ),+
            >;

            fn additional_tools<'a>(&'a self) -> impl $crate::tools::ToolResolver<'a> + 'a {
                let ( $( $S, )+ ) = self;
                ( $( $S.additional_tools(), )+ )
            }

            fn additional_skills(&self) -> Option<$crate::skill::SkillSet> {
                let ( $( $S, )+ ) = self;
                let mut skills: Option<$crate::skill::SkillSet> = None;
                $(
                    if let Some(additional) = $S.additional_skills() {
                        skills.get_or_insert_default().extend(additional);
                    }
                )+
                skills
            }

            fn additional_mcp_servers(&self) -> Option<$crate::mcp::McpServerSet> {
                let ( $( $S, )+ ) = self;
                let mut mcp_servers: Option<$crate::mcp::McpServerSet> = None;
                $(
                    if let Some(additional) = $S.additional_mcp_servers() {
                        mcp_servers.get_or_insert_default().extend(additional);
                    }
                )+
                mcp_servers
            }

            fn additional_builtin_tools(&self) -> Vec<L::BuiltinTool> {
                let ( $( $S, )+ ) = self;
                let mut builtin_tools = Vec::new();
                $(
                    builtin_tools.extend($S.additional_builtin_tools());
                )+
                builtin_tools
            }

            fn call_options(&self) -> L::Options {
                let ( $( $S, )+ ) = self;
                let mut options = L::Options::from_default();
                $(
                    options = options.merge($S.call_options());
                )+
                options
            }

            async fn process_initial_messages(
                &mut self,
                mut messages: Vec<$crate::schemas::Message>,
            ) -> Result<Vec<$crate::schemas::Message>, Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    messages = $S.process_initial_messages(messages).await
                        .map_err($error::$S)?;
                )+
                Ok(messages)
            }

            async fn edit_transcript(
                &mut self,
                transcript: &mut $crate::transcript::AgentTranscript<L::System, L::Item>,
            ) -> Result<(), Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    $S.edit_transcript(transcript).await.map_err($error::$S)?;
                )+
                Ok(())
            }

            async fn project_transcript(
                &mut self,
                transcript: &mut $crate::transcript::AgentTranscript<L::System, L::Item>,
            ) -> Result<(), Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    $S.project_transcript(transcript).await.map_err($error::$S)?;
                )+
                Ok(())
            }

            async fn process_materialized(
                &mut self,
                mut system: L::System,
                mut items: Vec<L::Item>,
            ) -> Result<(L::System, Vec<L::Item>), Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    (system, items) = $S.process_materialized(system, items).await.map_err($error::$S)?;
                )+
                Ok((system, items))
            }

            async fn process_request(
                &mut self,
                mut request: L::Request,
            ) -> Result<L::Request, Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    request = $S.process_request(request).await.map_err($error::$S)?;
                )+
                Ok(request)
            }

            async fn process_response(
                &mut self,
                mut response: L::Response,
            ) -> Result<L::Response, Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    response = $S.process_response(response).await.map_err($error::$S)?;
                )+
                Ok(response)
            }

            async fn inspect_usage(
                &mut self,
                usage: &$crate::schemas::Usage,
            ) -> Result<(), Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    $S.inspect_usage(usage).await.map_err($error::$S)?;
                )+
                Ok(())
            }

            async fn process_llm_step(
                &mut self,
                mut output: $crate::schemas::LlmStep,
            ) -> Result<$crate::schemas::LlmStep, Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    output = $S.process_llm_step(output).await.map_err($error::$S)?;
                )+
                Ok(output)
            }

            async fn process_tool_result(
                &mut self,
                mut message: $crate::schemas::ToolResultMessage,
            ) -> Result<Result<$crate::schemas::ToolOutput, $crate::tools::ToolError>, Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    message.result = $S.process_tool_result($crate::schemas::ToolResultMessage {
                        call_id: message.call_id.clone(),
                        name: message.name.clone(),
                        result: message.result,
                    }).await.map_err($error::$S)?;
                )+
                Ok(message.result)
            }

            async fn process_final_answer(
                &mut self,
                mut parts: ::std::vec::Vec<$crate::schemas::ContentPart>,
            ) -> Result<::std::vec::Vec<$crate::schemas::ContentPart>, Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    parts = $S.process_final_answer(parts).await
                        .map_err($error::$S)?;
                )+
                Ok(parts)
            }

            async fn process_output(
                &mut self,
                mut output: O::Partial,
            ) -> Result<O::Partial, Self::Error> {
                let ( $( $S, )+ ) = self;
                $(
                    output = $S.process_output(output).await.map_err($error::$S)?;
                )+
                Ok(output)
            }

            async fn finalize(self) -> Result<Self::Output, Self::Error> {
                let ( $( $S, )+ ) = self;
                Ok((
                    $(
                        $S.finalize().await.map_err($error::$S)?,
                    )+
                ))
            }
        }
    };
}

impl_strategy_for_tuple!(S0, S1; TupleError2);
impl_strategy_for_tuple!(S0, S1, S2; TupleError3);
impl_strategy_for_tuple!(S0, S1, S2, S3; TupleError4);
impl_strategy_for_tuple!(S0, S1, S2, S3, S4; TupleError5);
impl_strategy_for_tuple!(S0, S1, S2, S3, S4, S5; TupleError6);
impl_strategy_for_tuple!(S0, S1, S2, S3, S4, S5, S6; TupleError7);
impl_strategy_for_tuple!(S0, S1, S2, S3, S4, S5, S6, S7; TupleError8);
impl_strategy_for_tuple!(S0, S1, S2, S3, S4, S5, S6, S7, S8; TupleError9);
impl_strategy_for_tuple!(S0, S1, S2, S3, S4, S5, S6, S7, S8, S9; TupleError10);
