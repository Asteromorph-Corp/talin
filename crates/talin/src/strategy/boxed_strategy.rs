use std::error::Error;
use std::fmt;

use async_trait::async_trait;
use dyn_clone::{DynClone, clone_box};

use crate::chain::{OutputCtor, StringCtor};
use crate::llm::Llm;
use crate::mcp::McpServerSet;
use crate::schemas::{ContentPart, LlmStep, Message, ToolResult, ToolResultMessage, Usage};
use crate::skill::SkillSet;
use crate::strategy::Strategy;
use crate::tools::ToolResolver;
use crate::transcript::AgentTranscript;

/// A type-erased [`Strategy`] for backend `L`.
///
/// The [`Output`](Strategy::Output) of the strategy is discarded.
pub struct BoxedStrategy<L: Llm>(Box<dyn DynStrategy<L>>);

impl<L: Llm + 'static> BoxedStrategy<L> {
    /// Creates a new [`BoxedStrategy`] from a concrete [`Strategy`].
    pub fn new<S: Strategy<L, StringCtor> + Clone + 'static>(strategy: S) -> Self {
        Self(Box::new(strategy))
    }
}

impl<L: Llm> Clone for BoxedStrategy<L> {
    fn clone(&self) -> Self {
        Self(clone_box(&*self.0))
    }
}

#[async_trait]
impl<L: Llm, O: OutputCtor> Strategy<L, O> for BoxedStrategy<L> {
    type Output = ();
    type Error = BoxedStrategyError;

    fn additional_tools<'a>(&'a self) -> impl ToolResolver<'a> + 'a {
        self.0.additional_tools()
    }

    fn additional_skills(&self) -> Option<SkillSet> {
        self.0.additional_skills()
    }

    fn additional_mcp_servers(&self) -> Option<McpServerSet> {
        self.0.additional_mcp_servers()
    }

    fn additional_builtin_tools(&self) -> Vec<L::BuiltinTool> {
        self.0.additional_builtin_tools()
    }

    fn call_options(&self) -> L::Options {
        self.0.call_options()
    }

    async fn edit_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        self.0.edit_transcript(transcript).await
    }

    async fn project_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), Self::Error> {
        self.0.project_transcript(transcript).await
    }

    async fn process_materialized(
        &mut self,
        system: L::System,
        items: Vec<L::Item>,
    ) -> Result<(L::System, Vec<L::Item>), Self::Error> {
        self.0.process_materialized(system, items).await
    }

    async fn process_request(&mut self, request: L::Request) -> Result<L::Request, Self::Error> {
        self.0.process_request(request).await
    }

    async fn process_response(
        &mut self,
        response: L::Response,
    ) -> Result<L::Response, Self::Error> {
        self.0.process_response(response).await
    }

    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, Self::Error> {
        self.0.process_initial_messages(messages).await
    }

    async fn inspect_usage(&mut self, usage: &Usage) -> Result<(), Self::Error> {
        self.0.inspect_usage(usage).await
    }

    async fn process_llm_step(&mut self, step: LlmStep) -> Result<LlmStep, Self::Error> {
        self.0.process_llm_step(step).await
    }

    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, Self::Error> {
        self.0.process_tool_result(message).await
    }

    async fn process_final_answer(
        &mut self,
        parts: Vec<ContentPart>,
    ) -> Result<Vec<ContentPart>, Self::Error> {
        self.0.process_final_answer(parts).await
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        self.0.finalize().await
    }
}

/// The error type for a [`BoxedStrategy`].
#[derive(Debug)]
pub struct BoxedStrategyError(Box<dyn Error + Send + Sync + 'static>);

impl BoxedStrategyError {
    pub(crate) fn new<E: Error + Send + Sync + 'static>(error: E) -> Self {
        Self(Box::new(error))
    }
}

impl fmt::Display for BoxedStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BoxedStrategyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

/// A [`ToolResolver`] behind a box, since `Box<dyn ToolResolver>` does not implement the trait.
struct BoxedResolver<'a>(Box<dyn ToolResolver<'a> + 'a>);

impl<'a> ToolResolver<'a> for BoxedResolver<'a> {
    fn list_tools(&self) -> Vec<crate::tools::ToolRef<'a>> {
        self.0.list_tools()
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&'a dyn crate::tools::FunctionTool> {
        self.0.resolve_tool(tool_name)
    }
}

/// Object-safe mirror of [`Strategy`].
#[async_trait]
trait DynStrategy<L: Llm>: DynClone + Send + Sync {
    fn additional_tools<'a>(&'a self) -> BoxedResolver<'a>;
    fn additional_skills(&self) -> Option<SkillSet>;
    fn additional_mcp_servers(&self) -> Option<McpServerSet>;
    fn additional_builtin_tools(&self) -> Vec<L::BuiltinTool>;
    fn call_options(&self) -> L::Options;
    async fn edit_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), BoxedStrategyError>;
    async fn project_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), BoxedStrategyError>;
    async fn process_materialized(
        &mut self,
        system: L::System,
        items: Vec<L::Item>,
    ) -> Result<(L::System, Vec<L::Item>), BoxedStrategyError>;
    async fn process_request(
        &mut self,
        request: L::Request,
    ) -> Result<L::Request, BoxedStrategyError>;
    async fn process_response(
        &mut self,
        response: L::Response,
    ) -> Result<L::Response, BoxedStrategyError>;
    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, BoxedStrategyError>;
    async fn inspect_usage(&mut self, usage: &Usage) -> Result<(), BoxedStrategyError>;
    async fn process_llm_step(&mut self, step: LlmStep) -> Result<LlmStep, BoxedStrategyError>;
    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, BoxedStrategyError>;
    async fn process_final_answer(
        &mut self,
        parts: Vec<ContentPart>,
    ) -> Result<Vec<ContentPart>, BoxedStrategyError>;
    async fn finalize(self: Box<Self>) -> Result<(), BoxedStrategyError>;
}

#[async_trait]
impl<L, S> DynStrategy<L> for S
where
    L: Llm + 'static,
    S: Strategy<L, StringCtor> + Clone + 'static,
{
    fn additional_tools<'a>(&'a self) -> BoxedResolver<'a> {
        BoxedResolver(Box::new(Strategy::additional_tools(self)))
    }

    fn additional_skills(&self) -> Option<SkillSet> {
        Strategy::additional_skills(self)
    }

    fn additional_mcp_servers(&self) -> Option<McpServerSet> {
        Strategy::additional_mcp_servers(self)
    }

    fn additional_builtin_tools(&self) -> Vec<L::BuiltinTool> {
        Strategy::additional_builtin_tools(self)
    }

    fn call_options(&self) -> L::Options {
        Strategy::call_options(self)
    }

    async fn edit_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), BoxedStrategyError> {
        Strategy::edit_transcript(self, transcript)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn project_transcript(
        &mut self,
        transcript: &mut AgentTranscript<L::System, L::Item>,
    ) -> Result<(), BoxedStrategyError> {
        Strategy::project_transcript(self, transcript)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn process_materialized(
        &mut self,
        system: L::System,
        items: Vec<L::Item>,
    ) -> Result<(L::System, Vec<L::Item>), BoxedStrategyError> {
        Strategy::process_materialized(self, system, items)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn process_request(
        &mut self,
        request: L::Request,
    ) -> Result<L::Request, BoxedStrategyError> {
        Strategy::process_request(self, request)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn process_response(
        &mut self,
        response: L::Response,
    ) -> Result<L::Response, BoxedStrategyError> {
        Strategy::process_response(self, response)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, BoxedStrategyError> {
        Strategy::process_initial_messages(self, messages)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn inspect_usage(&mut self, usage: &Usage) -> Result<(), BoxedStrategyError> {
        Strategy::inspect_usage(self, usage)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn process_llm_step(&mut self, step: LlmStep) -> Result<LlmStep, BoxedStrategyError> {
        Strategy::process_llm_step(self, step)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn process_tool_result(
        &mut self,
        message: ToolResultMessage,
    ) -> Result<ToolResult, BoxedStrategyError> {
        Strategy::process_tool_result(self, message)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn process_final_answer(
        &mut self,
        parts: Vec<ContentPart>,
    ) -> Result<Vec<ContentPart>, BoxedStrategyError> {
        Strategy::process_final_answer(self, parts)
            .await
            .map_err(BoxedStrategyError::new)
    }

    async fn finalize(self: Box<Self>) -> Result<(), BoxedStrategyError> {
        Strategy::finalize(*self)
            .await
            .map(drop)
            .map_err(BoxedStrategyError::new)
    }
}
