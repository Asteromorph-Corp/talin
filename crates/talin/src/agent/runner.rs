use std::borrow::Cow;
use std::collections::HashSet;

use crate::agent::helper::use_strategy;
use crate::agent::inventory::Inventory;
use crate::agent::{AgentError, AgentOutput, AgentSession};
use crate::chain::{InputCtor, OutputCtor, PartialOutput, PartialWithInput};
use crate::llm::{Llm, LlmError, LlmExt, LlmRequest, LlmRequestItem, LlmResponse, Truncation};
use crate::mcp::McpServerSet;
use crate::schemas::{
    BodyMessage, ContentPart, LlmStep, ToolCall, ToolErrorMessage, ToolMessage, ToolResultMessage,
};
use crate::skill::SkillSet;
use crate::skill::wire::InjectCatalog;
use crate::strategy::{RequestParts, Strategy};
use crate::tools::{FunctionTool, ToolError, ToolExt, ToolRef, ToolResolver, normalize_tool_name};
use crate::transcript::query::has_tool_calls;
use crate::transcript::scope::Last;
use crate::transcript::{Entry, MessageRecord, ModelRecord, Scope};

/// A transient handle that drives a single turn of an [`AgentSession`] through the LLM-tool loop.
///
/// `SessionRunner` borrows the underlying [`AgentSession`] and owns the per-turn input, strategy,
/// and counters (use counts, consecutive fails, step count).
///
/// Use [`step`](Self::step) to step manually or [`run_to_finish`](Self::run_to_finish) to
/// loop until the model returns a final answer.
///
/// Use [`finalize`](Self::finalize) to convert a final partial output into the turn's final output
/// with fields from the input.
///
/// The runner uses both the agent's default strategy `S` and the per-turn strategy `A`.
pub struct AgentRunner<'session, 'input, L, I, O, S, A>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O>,
    A: Strategy<L, O>,
{
    /// The session that the runner belongs to.
    pub session: &'session mut AgentSession<L, I, O, S>,
    /// The input for the current turn.
    input: I::Target<'input>,
    /// The strategies for the current turn.
    strategy: (S, A),
    /// The turn inventory.
    inventory: Inventory,

    /// The current number of consecutive failures.
    consecutive_fails: usize,
    /// The current number of steps.
    step_count: usize,
    /// When set, [`advance`](Self::advance) applies [`LlmRequest::force_final`] to the outgoing
    /// request.
    force_final: bool,
}

/// A transient handle that drives a single step of an [`AgentRunner`].
///
/// See [`AgentRunner::half_step`] for details.
pub struct PendingStep<'runner, 'session, 'input, L, I, O, S, A>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O>,
    A: Strategy<L, O>,
{
    /// The runner that the step belongs to.
    pub runner: &'runner mut AgentRunner<'session, 'input, L, I, O, S, A>,
    /// The model produced output for the step.
    pub step: LlmStep,
}

impl<'session, 'input, L, I, O, S, A> AgentRunner<'session, 'input, L, I, O, S, A>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
    A: Strategy<L, O>,
{
    pub(crate) fn new(
        session: &'session mut AgentSession<L, I, O, S>,
        input: I::Target<'input>,
        strategy: (S, A),
    ) -> Self {
        Self {
            session,
            input,
            strategy,
            inventory: Inventory::default(),
            consecutive_fails: 0,
            step_count: 0,
            force_final: false,
        }
    }

    /// Drives the session to completion, returning the final output once the model produces a
    /// terminal answer.
    pub async fn run_to_finish(
        mut self,
    ) -> Result<AgentOutput<O::Target<'input>, A::Output>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        loop {
            match self.step().await {
                Ok(Some(partial)) => return self.finalize(partial).await,
                // Step succeeded but did not produce a final answer, continue loop.
                Ok(None) => {}
                Err(e) if !e.is_retryable() => return Err(e),
                Err(e) if self.fail_limit_reached() => {
                    log::error!("Too many consecutive fails, aborting");
                    return Err(e);
                }
                Err(_) => {}
            }
        }
    }

    /// Syncs the runner's inventory of MCP servers and skills.
    ///
    /// The runner manages an internal inventory of MCP clients and skills, which is used to
    /// resolve tool calls and skill invocations.
    ///
    /// The method is called automatically at the start of each step.
    pub async fn sync_inventory(&mut self) -> Result<(), AgentError> {
        self.session.sync_inventory().await?;

        let mcp_servers = self
            .session
            .mcp_servers
            .with_additional(self.strategy.additional_mcp_servers());
        let skills = self
            .session
            .skills
            .with_additional(self.strategy.additional_skills());
        self.inventory
            .sync(
                &mcp_servers,
                &skills,
                self.session.llm.supports_mcp(),
                Some(&self.session.inventory),
            )
            .await
    }

    /// Builds the LLM request for the current session state, honoring the strategy's additional
    /// tools and option overrides.
    async fn build_request(&mut self) -> Result<L::Request, AgentError> {
        // Permanent pass: commit idempotent structural edits to the stored transcript.
        use_strategy!(self.strategy, edit_transcript(&mut self.session.transcript))?;

        // Ephemeral pass: project a throwaway clone against live state, then consume it.
        let mut transcript = self.session.transcript.clone();
        let skills = self.skills().metadata().await?;
        transcript
            .apply_system_edit(&InjectCatalog(&skills))
            .await
            .unwrap_or_else(|e| match e {});
        use_strategy!(self.strategy, project_transcript(&mut transcript))?;

        let (system, items) = self.session.llm.materialize(transcript);
        let (system, items) = use_strategy!(self.strategy, process_materialized(system, items))?;

        let mut request = self
            .request_parts()
            .build(&self.session.llm, (system, items));
        if self.step_count > 1 {
            request = request.clear_tool_choice();
        }
        if self.force_final {
            request = request.force_final();
        }
        Ok(request)
    }

    /// The tools and options requests are built with, besides the conversation.
    fn request_parts(&self) -> RequestParts<L> {
        let resolver = (
            self.strategy.additional_tools(),
            &self.session.tools,
            &self.inventory,
            self.session.inventory.masked_by(&self.inventory),
        );
        let views = self.session.transcript.full_view();
        let listed = dedup_tools(resolver.list_tools());
        let available = listed
            .iter()
            .filter(|tool| match tool {
                ToolRef::Function(function) => function.available(&views),
            })
            .collect::<Vec<_>>();
        // Providers reject a conversation with tool calls but no tool definitions.
        let tools = if available.is_empty() && has_tool_calls(&views) {
            listed.iter().map(ToolExt::get_spec).collect()
        } else {
            available.iter().map(|t| t.get_spec()).collect()
        };
        let builtin_tools = self
            .session
            .builtin_tools
            .clone()
            .into_iter()
            .chain(self.strategy.additional_builtin_tools())
            .collect::<Vec<_>>();
        let mcp_servers = self
            .mcp_servers()
            .route(self.session.llm.supports_mcp())
            .remote;
        let options = self.strategy.call_options();

        RequestParts {
            tools,
            builtin_tools,
            mcp_servers,
            options,
        }
    }

    /// Applies the session's compaction policy to the stored transcript.
    async fn compact(&mut self) -> Result<(), AgentError> {
        let parts = self.request_parts();
        if let Some(compaction) = &mut self.session.compaction {
            let result = compaction
                .compact(&self.session.llm, &mut self.session.transcript, &parts)
                .await?;
            use_strategy!(self.strategy, inspect_usage(&result.usage))?;
        }
        Ok(())
    }

    /// Sends the LLM request.
    async fn send_request(&mut self, request: L::Request) -> Result<L::Response, AgentError> {
        let response: L::Response = self.session.llm.send_request(request).await?;
        let response = use_strategy!(self.strategy, process_response(response))?;
        let usage = self.session.llm.compute_usage(response.get_usage());

        use_strategy!(self.strategy, inspect_usage(&usage))?;
        Ok(response)
    }

    /// Executes a single step of the session, returning a partial output if the model produced a
    /// final answer.
    pub async fn step(&mut self) -> Result<Option<O::Partial>, AgentError> {
        let pending = self.half_step().await?;
        pending.complete().await.inspect_err(|e| self.on_fail(e))
    }

    /// Executes the LLM half of a step, returning a [`PendingStep`].
    ///
    /// Unlike [`step`](Self::step), this does not execute any tool calls, so you can inspect the
    /// [`LlmStep`] and decide how to handle the tool calls:
    /// - You can [`complete`](PendingStep::complete) the step, which resolves the tool calls
    ///   automatically using the tools configured for the agent. This is the default behavior of
    ///   [`step`](Self::step).
    /// - Alternatively, you can resolve the tool calls manually by providing tool results with
    ///   [`complete_with`](PendingStep::complete_with), or [`discard`](PendingStep::discard) the
    ///   step to roll it back and re-run the LLM step.
    ///
    /// Calling any of the methods consumes [`PendingStep`], after which you can continue the turn
    /// with the same [`AgentRunner`].
    pub async fn half_step(
        &mut self,
    ) -> Result<PendingStep<'_, 'session, 'input, L, I, O, S, A>, AgentError> {
        let step = self.step_inner().await.inspect_err(|e| self.on_fail(e))?;
        Ok(PendingStep { runner: self, step })
    }

    async fn step_inner(&mut self) -> Result<LlmStep, AgentError> {
        if self.max_iterations_reached(1) {
            log::warn!("Max iteration reached, forcing final answer");
            self.force_final = true;
        }

        let step = self.advance().await?;
        self.step_count += 1;
        if self.max_iterations_reached(0) && matches!(step, LlmStep::ToolCall(_)) {
            return Err(AgentError::ToolCallAfterMaxIteration);
        }
        Ok(step)
    }

    /// Issues a single LLM call, recording the response and producing the resulting [`LlmStep`].
    async fn advance(&mut self) -> Result<LlmStep, AgentError> {
        self.session
            .transcript
            .remove_unresolved_tool_calls(&Scope::from(Last::step()))?;
        self.sync_inventory().await?;
        self.session.transcript.mark_step_start();

        self.compact().await?;
        let request: L::Request = self.build_request().await?;
        let request = use_strategy!(self.strategy, process_request(request))?;
        let response: L::Response = self.send_request(request).await?;

        let truncation = response.truncation();
        let mut record = self.session.llm.parse_response(response)?;
        if let Some(truncation) = truncation {
            record = self.continue_truncated(record, truncation).await?;
        }

        let step = record.canonical.clone();
        // The canonical side records that this response replaced earlier context.
        if record.native.contains_native_compaction() {
            self.session.transcript.mark_compaction(false);
        }
        self.session
            .transcript
            .push(Entry::from(MessageRecord::from(record)));

        let modified = use_strategy!(self.strategy, process_llm_step(step))?;
        self.session.transcript.modify_step(modified.clone())?;
        Ok(modified)
    }

    /// Asks the model to finish a truncated output.
    ///
    /// The truncated output is only appended to the transcript if the continuation succeeds.
    async fn continue_truncated(
        &mut self,
        mut record: ModelRecord<L::Item>,
        mut truncation: Truncation,
    ) -> Result<ModelRecord<L::Item>, AgentError> {
        let mut count = 1;
        loop {
            if self
                .session
                .options
                .max_continuations
                .is_some_and(|max| count > max)
            {
                return Err(LlmError::Truncated(truncation).into());
            }
            let Some(partial) = record.into_truncated()? else {
                return Err(LlmError::Truncated(truncation).into());
            };

            log::warn!(
                "received incomplete response ({truncation}); \
                 resuming from truncated output (attempt #{count})"
            );

            let request: L::Request = self.build_request().await?;
            let Ok(request) = self
                .session
                .llm
                .build_continuation(request, partial.clone())
                .inspect_err(|e| log::warn!("failed to continue from truncated output: {e}"))
            else {
                return Err(LlmError::Truncated(truncation).into());
            };
            let request = use_strategy!(self.strategy, process_request(request))?;
            let response: L::Response = self.send_request(request).await?;

            let next = response.truncation();
            record = self.session.llm.parse_continuation(response, partial)?;

            let Some(next) = next else {
                return Ok(record);
            };
            truncation = next;
            count += 1;
        }
    }

    /// Executes the tool calls, recording results and errors back into the session.
    async fn handle_tool_calls(&mut self, tool_calls: Vec<ToolCall>) -> bool {
        let mut success = tool_calls.is_empty();
        for call in tool_calls {
            success |= self.handle_tool_call(call).await;
        }
        success
    }

    /// Executes a single tool call.
    async fn handle_tool_call(&mut self, call: ToolCall) -> bool {
        log::debug!("\nTool call:\n{call}");

        let views = self.session.transcript.full_view();
        let tool = self.resolve_tool(&call.name);
        let result = match tool {
            Some(tool) if tool.available(&views) => tool.call(call.arguments).await,
            Some(_) => Err(ToolError::Unavailable),
            None => Err(ToolError::ToolNotFound(call.name.clone())),
        };
        let message = ToolResultMessage::new(call.id.clone(), call.name.clone(), result);
        let result = match self.strategy.process_tool_result(message).await {
            Ok(result) => result,
            Err(e) => Err(ToolError::OutputProcessing(Box::new(e))),
        };

        match &result {
            Ok(output) => log::debug!("\nTool '{}' result:\n{}", call.name, output),
            Err(e) => log::warn!("\nTool '{}' error:\n{}", call.name, e),
        }

        self.push_tool_result(ToolResultMessage::new(call.id, call.name, result))
    }

    /// Parses the LLM's final answer into a partial final output.
    async fn parse_answer(&mut self, parts: Vec<ContentPart>) -> Result<O::Partial, AgentError> {
        log::debug!("\nAgent finished with result:\n{parts:?}");

        let answer = use_strategy!(self.strategy, process_final_answer(parts))?;
        let parsed = O::Partial::from_parts(answer)?;
        let processed: O::Partial = use_strategy!(self.strategy, process_output(parsed))?;

        Ok(processed)
    }

    /// Finalizes the turn output using the partial output produced by the model.
    ///
    /// Provided as a separate method, since it needs to consume the input struct as well as
    /// strategy.
    pub async fn finalize(
        self,
        partial: O::Partial,
    ) -> Result<AgentOutput<O::Target<'input>, A::Output>, AgentError>
    where
        O::Partial: PartialWithInput<I::Target<'input>, O::Target<'input>>,
    {
        let content: O::Target<'input> = partial.with_input(self.input);
        let extra_content = use_strategy!(self.strategy, finalize())?.1;
        Ok(AgentOutput {
            content,
            extra_content,
        })
    }

    /// Returns a [`ToolResolver`] that can be used to resolve tool calls in the session.
    pub fn tool_resolver(&self) -> impl ToolResolver<'_> {
        (
            self.strategy.additional_tools(),
            &self.session.tools,
            &self.inventory,
            self.session.inventory.masked_by(&self.inventory),
        )
    }

    fn mcp_servers(&self) -> Cow<'_, McpServerSet> {
        self.session
            .mcp_servers
            .with_additional(self.strategy.additional_mcp_servers())
    }

    /// The metadata of the skills available on this step: the session's, and the strategy's.
    fn skills(&self) -> Cow<'_, SkillSet> {
        self.session
            .skills
            .with_additional(self.strategy.additional_skills())
    }

    fn resolve_tool(&self, tool_name: &str) -> Option<&dyn FunctionTool> {
        let name = normalize_tool_name(tool_name);
        let resolver = (
            self.strategy.additional_tools(),
            &self.session.tools,
            &self.inventory,
            self.session.inventory.masked_by(&self.inventory),
        );
        resolver.resolve_tool(&name)
    }

    /// Records tool results in the transcript.
    fn push_tool_results(&mut self, results: Vec<ToolResultMessage>) -> bool {
        let mut success = results.is_empty();
        for result in results {
            success |= self.push_tool_result(result);
        }
        success
    }

    /// Records the tool result in the transcript.
    fn push_tool_result(&mut self, message: ToolResultMessage) -> bool {
        let result = message.result.and_then(|output| {
            let tool = ToolMessage::new(message.call_id.clone(), message.name.clone(), output);
            self.session
                .transcript
                .push_message(BodyMessage::Tool(tool))
                .map_err(|e| {
                    log::warn!("failed to record result for tool '{}': {e}", message.name);
                    ToolError::OutputProcessing(Box::new(e))
                })
        });

        match result {
            Ok(_) => true,
            Err(error) => {
                let error = ToolErrorMessage::new(message.call_id, message.name, error);
                self.session.transcript.push_tool_error(error);
                false
            }
        }
    }

    fn on_fail(&mut self, error: &AgentError) {
        self.consecutive_fails += 1;
        log::warn!("{} ({} consecutive fails)", error, self.consecutive_fails);

        match error {
            // The model may require the failure information to produce a valid next step.
            AgentError::AllToolsFailed => {}
            // Rollback transcript in case of any other errors.
            _ => self.session.transcript.rollback(&Last::step()),
        }
    }

    fn max_iterations_reached(&self, after: usize) -> bool {
        self.session
            .options
            .max_iterations
            .is_some_and(|max_iterations| self.step_count + after >= max_iterations)
    }

    fn fail_limit_reached(&self) -> bool {
        self.session
            .options
            .max_consecutive_fails
            .is_some_and(|max_consecutive_fails| self.consecutive_fails >= max_consecutive_fails)
    }
}

impl<L, I, O, S, A> PendingStep<'_, '_, '_, L, I, O, S, A>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    S: Strategy<L, O> + Clone,
    A: Strategy<L, O>,
{
    /// Complete the step by resolving tool calls automatically using the tools configured for the
    /// agent.
    pub async fn complete(self) -> Result<Option<O::Partial>, AgentError> {
        match self.step {
            LlmStep::ToolCall(m) => {
                let success = self.runner.handle_tool_calls(m.tool_calls).await;
                if !success {
                    return Err(AgentError::AllToolsFailed);
                }
                self.runner.consecutive_fails = 0;
                Ok(None)
            }
            LlmStep::Finish(m) => {
                let partial = self.runner.parse_answer(m.content).await?;
                Ok(Some(partial))
            }
        }
    }

    /// Complete the step with custom results for the tool calls, instead of running the tools.
    ///
    /// The results can be errors, e.g. to tell the model that the user declined a call.
    pub fn complete_with(self, results: Vec<ToolResultMessage>) {
        let success = self.runner.push_tool_results(results);
        if success {
            self.runner.consecutive_fails = 0;
        }
    }

    /// Discard the step, rolling back the transcript to before it so the LLM step can be re-run.
    pub fn discard(self) {
        self.runner.session.rollback(&Last::step());
    }
}

/// Removes tools with duplicate names, keeping the first occurrence.
fn dedup_tools(tools: Vec<ToolRef<'_>>) -> Vec<ToolRef<'_>> {
    let mut seen = HashSet::new();
    tools
        .into_iter()
        .filter(|tool| seen.insert(normalize_tool_name(&tool.name())))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::agent::{Agent, AgentOptions};
    use crate::llm::{
        ContinuationError, LlmOptions, LlmPricing, LlmRequestItem, LlmRequestSystem, LlmUsage,
        Truncation,
    };
    use crate::mcp::McpServerHttp;
    use crate::schemas::{
        AssistantMessage, ContentExt, ModelKey, ToolCallMessage, UsageLines, UserMessage,
    };
    use crate::text::prompts::CONTINUE_PARTIAL;
    use crate::tools::ToolSpec;
    use crate::transcript::scope::Scope;
    use crate::transcript::{AgentTranscript, EntryView, ModelRecord, TruncatedRecord};

    /// The names of the tools.
    fn tool_names(tools: &[ToolSpec]) -> Vec<&str> {
        tools.iter().map(ToolSpec::name).collect()
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct ScriptedRequest {
        system: Vec<String>,
        items: Vec<BodyMessage>,
        tools: Vec<ToolSpec>,
        mcp_servers: Vec<McpServerHttp>,
        options: LlmOptions,
    }
    struct ScriptedMessage {
        step: LlmStep,
        truncated: bool,
    }
    struct ScriptedUsage;
    struct ScriptedPricing;

    /// An [`Llm`] that hands out a scripted sequence of steps and records every request the runner
    /// builds, so a turn can be driven end to end without a provider.
    #[derive(Clone)]
    struct ScriptedLlm {
        /// The steps to respond with, each paired with whether it was cut short.
        script: Arc<Mutex<VecDeque<ScriptedMessage>>>,
        /// The list of requests sent.
        requests: Arc<Mutex<Vec<ScriptedRequest>>>,
        /// Whether requests constrain the output to a schema, and so cannot be continued.
        structured: bool,
        /// Whether the provider supports MCP connections natively.
        supports_mcp: bool,
    }

    impl ScriptedLlm {
        fn new(script: impl IntoIterator<Item = ScriptedMessage>) -> Self {
            Self {
                script: Arc::new(Mutex::new(script.into_iter().collect())),
                requests: Arc::new(Mutex::new(Vec::new())),
                structured: false,
                supports_mcp: true,
            }
        }

        fn structured(mut self) -> Self {
            self.structured = true;
            self
        }

        fn mcp_support(mut self, support: bool) -> Self {
            self.supports_mcp = support;
            self
        }

        fn requests(&self) -> Vec<ScriptedRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl Llm for ScriptedLlm {
        type Request = ScriptedRequest;
        type Response = ScriptedMessage;

        type System = Vec<String>;
        type Item = BodyMessage;
        type BuiltinTool = ();
        type Options = LlmOptions;

        type Pricing = ScriptedPricing;
        type Usage = ScriptedUsage;

        fn build_request(
            &self,
            system: Self::System,
            items: Vec<Self::Item>,
            tools: Vec<ToolSpec>,
            _: Vec<Self::BuiltinTool>,
            mcp_servers: Vec<McpServerHttp>,
            options: Self::Options,
        ) -> Self::Request {
            ScriptedRequest {
                system,
                items,
                tools,
                mcp_servers,
                options,
            }
        }

        async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
            self.requests.lock().unwrap().push(request.clone());
            self.script
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| LlmError::other("the script ran out of steps"))
        }

        fn parse_response(
            &self,
            response: Self::Response,
        ) -> Result<ModelRecord<Self::Item>, LlmError> {
            let native = BodyMessage::from(response.step.clone());
            Ok(ModelRecord {
                canonical: response.step,
                native,
            })
        }

        fn build_continuation(
            &self,
            mut request: Self::Request,
            partial: TruncatedRecord<Self::Item>,
        ) -> Result<Self::Request, ContinuationError> {
            // Stand in for a grammar-constrained request, which cannot be continued.
            if self.structured {
                return Err(ContinuationError::StructuredOutput);
            }
            let prompt = BodyMessage::User(UserMessage::new(CONTINUE_PARTIAL));
            request.items.extend([partial.native, prompt]);
            Ok(request)
        }

        fn supports_mcp(&self) -> bool {
            self.supports_mcp
        }

        fn model_key(&self) -> ModelKey {
            ModelKey::new("scripted".to_owned(), "test")
        }

        fn pricing(&self) -> Option<Self::Pricing> {
            None
        }

        fn with_default_options(self, _: impl Into<Self::Options>) -> Self {
            self
        }
    }

    impl LlmRequest for ScriptedRequest {
        fn clear_tool_choice(self) -> Self {
            self
        }

        fn force_final(self) -> Self {
            self
        }
    }

    impl ScriptedMessage {
        fn new<T: Into<LlmStep>>(step: T) -> Self {
            Self {
                step: step.into(),
                truncated: false,
            }
        }

        fn truncated<T: Into<LlmStep>>(step: T) -> Self {
            Self {
                step: step.into(),
                truncated: true,
            }
        }
    }

    impl LlmResponse for ScriptedMessage {
        type Usage = ScriptedUsage;

        fn get_usage(&self) -> Option<Self::Usage> {
            None
        }

        fn truncation(&self) -> Option<Truncation> {
            self.truncated.then_some(Truncation::max_tokens())
        }
    }

    impl LlmUsage for ScriptedUsage {}

    impl LlmPricing for ScriptedPricing {
        type Usage = ScriptedUsage;

        fn default_usage_lines(_: Self::Usage) -> UsageLines {
            UsageLines::default()
        }

        fn compute_usage_lines(&self, _: Self::Usage) -> UsageLines {
            UsageLines::default()
        }
    }

    /// The model messages of the last turn, in order.
    fn model_messages<S: LlmRequestSystem, B: LlmRequestItem>(
        transcript: &AgentTranscript<S, B>,
    ) -> Vec<BodyMessage> {
        transcript
            .view(&Scope::from(Last::turn()))
            .into_iter()
            .filter_map(|entry| match entry {
                EntryView::Message(_, m)
                    if matches!(m, BodyMessage::Assistant(_) | BodyMessage::ToolCall(_)) =>
                {
                    Some(m.clone())
                }
                _ => None,
            })
            .collect()
    }

    fn text(step: &BodyMessage) -> String {
        match step {
            BodyMessage::Assistant(m) => m.content.to_text().unwrap_or_default(),
            _ => panic!("not a final answer: {step:?}"),
        }
    }

    mod continuation {
        use super::*;

        #[tokio::test]
        async fn continues_truncated_answer() {
            let script = [
                ScriptedMessage::truncated(AssistantMessage::new(None, "One, ")),
                ScriptedMessage::new(AssistantMessage::new(None, "two, three.")),
            ];

            let llm = ScriptedLlm::new(script);
            let options = AgentOptions::default().with_max_continuations(1);
            let agent: Agent<_> = Agent::builder().options(options).build(llm.clone());
            let mut session = agent.session();

            // The partial output was preserved and the continuation spliced onto it.
            let answer = session.run_turn("Count to three.").await.unwrap();
            assert_eq!(answer, "One, two, three.");

            // Inspect the continuation request.
            let requests = llm.requests();
            assert_eq!(requests.len(), 2, "expected one continuation request");
            let [.., partial, BodyMessage::User(prompt)] = requests[1].items.as_slice() else {
                panic!(
                    "continuation request should end with partial output and continuation prompt"
                )
            };
            assert_eq!(text(partial), "One, ");
            assert_eq!(prompt.content.to_text().as_deref(), Some(CONTINUE_PARTIAL));

            // The halves were merged into a single model message; partial output and continuation
            // prompt did not enter the transcript.
            let messages = model_messages(&session.transcript);
            assert_eq!(messages.len(), 1);
            assert_eq!(text(&messages[0]), "One, two, three.");
            let users = session
                .transcript
                .view(&Scope::from(Last::turn()))
                .into_iter()
                .filter(|entry| matches!(entry, EntryView::Message(_, BodyMessage::User(_))))
                .count();
            assert_eq!(users, 1, "the continuation prompt outlived the merge");
        }

        #[tokio::test]
        async fn drops_truncated_tool_call() {
            let script = [
                ScriptedMessage::truncated(ToolCallMessage::from_parts(
                    None,
                    vec![ContentPart::Text("Let me look ".to_owned())],
                    vec![ToolCall::new(Some("call_1".to_owned()), "search", None)],
                )),
                ScriptedMessage::new(AssistantMessage::new(None, "it up: 42.")),
            ];

            let llm = ScriptedLlm::new(script);
            let options = AgentOptions::default().with_max_continuations(1);
            let agent: Agent<_> = Agent::builder().options(options).build(llm.clone());
            let mut session = agent.session();

            let answer = session.run_turn("What is the answer?").await.unwrap();
            assert_eq!(answer, "Let me look it up: 42.");

            // The tool call had incomplete arguments, so it was never sent back to the model — an
            // unresolved call is rejected by providers, and its arguments cannot be trusted.
            let requests = llm.requests();
            let items = &requests[1].items;
            assert!(
                !items
                    .iter()
                    .any(|item| matches!(item, BodyMessage::ToolCall(_))),
                "the unfinished tool call was sent back to the model"
            );
        }

        #[tokio::test]
        async fn fails_on_truncated_structured_output() {
            let script = [ScriptedMessage::truncated(AssistantMessage::new(
                None,
                "{\"count\": ",
            ))];

            // Continuations are allowed, so the refusal comes from the structured request itself,
            // not the continuation limit.
            let llm = ScriptedLlm::new(script).structured();
            let options = AgentOptions::default()
                .with_max_continuations(1)
                .with_max_consecutive_fails(1);
            let agent: Agent<_> = Agent::builder().options(options).build(llm.clone());
            let mut session = agent.session();

            let error = session.run_turn("Count to three.").await.unwrap_err();
            assert!(
                matches!(error, AgentError::Model(LlmError::Truncated(_))),
                "expected truncation error, got: {error}"
            );

            // The refusal happens before any continuation request is sent, and the partial output
            // is left out of the transcript.
            assert_eq!(llm.requests().len(), 1, "a continuation request was sent");
            assert!(model_messages(&session.transcript).is_empty());
        }

        #[tokio::test]
        async fn fails_on_truncated_if_continuations_disabled() {
            let script = [ScriptedMessage::truncated(AssistantMessage::new(
                None, "One, ",
            ))];

            let llm = ScriptedLlm::new(script);
            let options = AgentOptions::default()
                .with_max_continuations(0)
                .with_max_consecutive_fails(1);
            let agent: Agent<_> = Agent::builder().options(options).build(llm.clone());
            let mut session = agent.session();

            let error = session.run_turn("Count to three.").await.unwrap_err();
            assert!(
                matches!(error, AgentError::Model(LlmError::Truncated(_))),
                "expected truncation error, got: {error}"
            );

            // The refusal happens before any continuation request is sent, and the partial output
            // is left out of the transcript.
            assert_eq!(llm.requests().len(), 1, "a continuation request was sent");
            assert!(model_messages(&session.transcript).is_empty());
        }
    }

    mod availability {
        use async_trait::async_trait;
        use schemars::JsonSchema;
        use serde::Deserialize;

        use super::*;
        use crate::tools::{Function, Tool};
        use crate::transcript::EntryView;
        use crate::transcript::query::{count_successful_tool_calls, count_tool_calls};

        #[derive(Deserialize, JsonSchema)]
        struct NoInput {}

        /// A tool that can be called at most `limit` times in the transcript.
        struct Capped {
            name: &'static str,
            limit: usize,
        }

        #[async_trait]
        impl Function for Capped {
            type Input = NoInput;
            type Output = String;

            fn name(&self) -> String {
                self.name.to_owned()
            }

            fn description(&self) -> String {
                "A capped tool.".to_owned()
            }

            fn available(&self, transcript: &[EntryView<'_>]) -> bool {
                count_tool_calls(transcript, self.name) < self.limit
            }

            async fn call(
                &self,
                _: Self::Input,
            ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
                Ok("ok".to_owned())
            }
        }

        fn call(id: &str, name: &str) -> ToolCall {
            ToolCall::new(Some(id.to_owned()), name, Some(serde_json::json!({})))
        }

        #[tokio::test]
        async fn drops_unavailable_tool_and_rejects_calls() {
            let script = [
                // Three calls in one step against a cap of two: the third is rejected.
                ScriptedMessage::new(ToolCallMessage::new(
                    None,
                    vec![
                        call("1", "capped"),
                        call("2", "capped"),
                        call("3", "capped"),
                    ],
                )),
                ScriptedMessage::new(AssistantMessage::new(None, "done")),
            ];
            let llm = ScriptedLlm::new(script);
            let agent: Agent<_> = Agent::builder()
                .tools([
                    Tool::from(Capped {
                        name: "capped",
                        limit: 2,
                    }),
                    Tool::from(Capped {
                        name: "other",
                        limit: 100,
                    }),
                ])
                .build(llm.clone());
            let mut session = agent.session();
            session.run_turn("Go.").await.unwrap();

            let requests = llm.requests();
            assert_eq!(requests.len(), 2);
            assert!(tool_names(&requests[0].tools).contains(&"capped"));
            assert!(tool_names(&requests[0].tools).contains(&"other"));
            // After the cap is reached, the tool is left out; the history is untouched.
            assert_eq!(tool_names(&requests[1].tools), ["other"]);

            let rejected = session
                .transcript
                .full_view()
                .into_iter()
                .filter(|v| matches!(v, EntryView::ToolError(..)))
                .count();
            assert_eq!(rejected, 1, "only the third call is rejected");
        }

        /// A tool that fails on its first invocation and is capped on successful calls only.
        struct Fragile {
            attempts: std::sync::atomic::AtomicUsize,
        }

        #[async_trait]
        impl Function for Fragile {
            type Input = NoInput;
            type Output = String;

            fn name(&self) -> String {
                "fragile".to_owned()
            }

            fn description(&self) -> String {
                "Fails once.".to_owned()
            }

            fn available(&self, transcript: &[EntryView<'_>]) -> bool {
                count_successful_tool_calls(transcript, "fragile") < 1
            }

            async fn call(
                &self,
                _: Self::Input,
            ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
                let attempt = self
                    .attempts
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempt == 0 {
                    return Err("first attempt fails".into());
                }
                Ok("ok".to_owned())
            }
        }

        #[tokio::test]
        async fn caps_successful_calls_only() {
            let script = [
                // The failed first call does not use up the cap, the second succeeds and
                // exhausts it, so the third is rejected.
                ScriptedMessage::new(ToolCallMessage::new(
                    None,
                    vec![
                        call("1", "fragile"),
                        call("2", "fragile"),
                        call("3", "fragile"),
                    ],
                )),
                ScriptedMessage::new(AssistantMessage::new(None, "done")),
            ];
            let llm = ScriptedLlm::new(script);
            let agent: Agent<_> = Agent::builder()
                .tools([
                    Tool::from(Fragile {
                        attempts: Default::default(),
                    }),
                    Tool::from(Capped {
                        name: "other",
                        limit: 100,
                    }),
                ])
                .build(llm.clone());
            let mut session = agent.session();
            session.run_turn("Go.").await.unwrap();

            let views = session.transcript.full_view();
            assert_eq!(count_successful_tool_calls(&views, "fragile"), 1);
            assert_eq!(count_tool_calls(&views, "fragile"), 3);
            assert_eq!(tool_names(&llm.requests()[1].tools), ["other"]);
        }

        #[tokio::test]
        async fn keeps_tools_when_none_would_remain() {
            let script = [
                ScriptedMessage::new(ToolCallMessage::new(None, vec![call("1", "capped")])),
                ScriptedMessage::new(AssistantMessage::new(None, "done")),
            ];
            let llm = ScriptedLlm::new(script);
            let agent: Agent<_> = Agent::builder()
                .tools([Tool::from(Capped {
                    name: "capped",
                    limit: 1,
                })])
                .build(llm.clone());
            let mut session = agent.session();
            session.run_turn("Go.").await.unwrap();

            let requests = llm.requests();
            assert_eq!(tool_names(&requests[1].tools), ["capped"]);
        }
    }

    mod skills {
        use indoc::indoc;

        use super::*;
        use crate::skill::{MemorySkill, Skill, SkillSet};

        #[tokio::test]
        async fn skills_injected_and_loaded() {
            let load_skill = ToolCall::new(
                Some("call_1".to_owned()),
                "load_skill",
                Some(serde_json::json!({ "name": "pdf" })),
            );
            let script = [
                ScriptedMessage::new(ToolCallMessage::new(None, vec![load_skill])),
                ScriptedMessage::new(AssistantMessage::new(None, "done")),
            ];

            let skill = MemorySkill::from_markdown(indoc! {"
            ---
            name: pdf
            description: Handle PDF files
            ---
            Unique pdf instructions."
            })
            .unwrap();
            let skill: Arc<dyn Skill> = Arc::new(skill);
            let skills = SkillSet::from_iter([skill]);

            let llm = ScriptedLlm::new(script);
            let agent: Agent<_> = Agent::builder().skills(skills).build(llm.clone());
            let mut session = agent.session();

            let answer = session.run_turn("Use a skill.").await.unwrap();
            assert_eq!(answer, "done");

            // The catalog is in the system prompt and the loader tools are offered every step.
            let requests = llm.requests();
            assert_eq!(requests.len(), 2);
            for request in &requests {
                assert!(
                    request
                        .system
                        .iter()
                        .any(|s| s.contains("<name>pdf</name>"))
                );
                assert!(tool_names(&request.tools).contains(&"load_skill"));
                assert!(tool_names(&request.tools).contains(&"read_skill_file"));
            }

            // The instructions enters the context only through the tool result.
            assert!(
                !requests[0]
                    .system
                    .iter()
                    .any(|s| s.contains("Unique pdf instructions.")),
                "instructions leaked into the system prompt"
            );
            let result = requests[1]
                .items
                .iter()
                .find_map(|m| match m {
                    BodyMessage::Tool(t) => Some(format!("{:?}", t.output)),
                    _ => None,
                })
                .expect("the tool result was sent back");
            assert!(result.contains("Unique pdf instructions."), "{result}");

            // The catalog is a projection, so it does not accumulate in the stored system prompt.
            let (transcript_system, _) = session.transcript.clone().materialize();
            assert!(
                !transcript_system
                    .iter()
                    .any(|s| s.contains("<available_skills>")),
                "the catalog was persisted into the transcript"
            );
        }
    }

    mod mcp {
        use super::*;
        use crate::mcp::McpServer;
        use crate::mcp::test_server::spawn_mock_server;

        /// A provider without native MCP support never sees the servers. A local client is started
        /// for each of them and its tools are offered like any other tool.
        #[tokio::test]
        async fn handles_mcp_servers_locally_for_providers_without_mcp_support() {
            let url = spawn_mock_server().await;
            let llm = ScriptedLlm::new([ScriptedMessage::new(AssistantMessage::new(None, "done"))])
                .mcp_support(false);
            let agent: Agent<_> = Agent::builder()
                .mcp_servers([McpServer::http(url, "mock")])
                .build(llm.clone());
            let mut session = agent.session();

            session.run_turn("hi").await.unwrap();

            let request = llm.requests().pop().unwrap();
            assert!(
                request.mcp_servers.is_empty(),
                "the server was sent to the provider"
            );
            let names = tool_names(&request.tools);
            assert!(names.contains(&"echo"), "{names:?}");
            assert!(names.contains(&"greet"), "{names:?}");
        }

        /// A provider with native MCP support gets the servers with every request, separately from
        /// the tools, and no local client is started for them.
        #[tokio::test]
        async fn handles_mcp_servers_remotely_for_providers_with_mcp_support() {
            let llm = ScriptedLlm::new([ScriptedMessage::new(AssistantMessage::new(None, "done"))])
                .mcp_support(true);
            let agent: Agent<_> = Agent::builder()
                .mcp_servers([McpServer::http("http://localhost:1/mcp", "remote")])
                .build(llm.clone());
            let mut session = agent.session();

            session.run_turn("hi").await.unwrap();

            let request = llm.requests().pop().unwrap();
            assert!(request.tools.is_empty(), "a server is not a tool");
            assert_eq!(request.mcp_servers.len(), 1);
            assert_eq!(request.mcp_servers[0].name(), "remote");
        }
    }

    mod strategy {
        use super::*;
        use crate::mcp::McpServer;
        use crate::skill::{MemorySkill, Skill, SkillMetadata};

        /// A [`Strategy`] that sets the sampling temperature of the run.
        #[derive(Clone)]
        struct Temperature(f32);

        #[async_trait::async_trait]
        impl<O: OutputCtor> Strategy<ScriptedLlm, O> for Temperature {
            type Output = ();
            type Error = std::convert::Infallible;

            fn call_options(&self) -> LlmOptions {
                LlmOptions::new().with_temperature(self.0)
            }

            async fn finalize(self) -> Result<Self::Output, Self::Error> {
                Ok(())
            }
        }

        /// The strategy's call options reach the provider with the request.
        #[tokio::test]
        async fn strategy_call_options_applied() {
            let llm = ScriptedLlm::new([ScriptedMessage::new(AssistantMessage::new(None, "done"))]);
            let agent: Agent<_> = Agent::builder().build(llm.clone());
            let mut session = agent.session();
            session
                .run_turn_with_strategy(Temperature(0.2), "hi")
                .await
                .unwrap();

            let request = llm.requests().pop().unwrap();
            assert_eq!(request.options.temperature, Some(0.2));
        }

        /// A [`Strategy`] that adds MCP servers to the run.
        #[derive(Clone)]
        struct AddMcpServers(Vec<McpServer>);

        #[async_trait::async_trait]
        impl<L: Llm, O: OutputCtor> Strategy<L, O> for AddMcpServers {
            type Output = ();
            type Error = std::convert::Infallible;

            fn additional_mcp_servers(&self) -> Option<McpServerSet> {
                Some(self.0.iter().cloned().collect())
            }

            async fn finalize(self) -> Result<Self::Output, Self::Error> {
                Ok(())
            }
        }

        /// A strategy's MCP servers are passed to the provider for its turn only, and win over the
        /// session's servers of the same name.
        #[tokio::test]
        async fn strategy_adds_mcp_servers() {
            let script = [
                ScriptedMessage::new(AssistantMessage::new(None, "one")),
                ScriptedMessage::new(AssistantMessage::new(None, "two")),
            ];
            let default_servers = [
                McpServer::http("http://localhost:1/mcp", "shared"),
                McpServer::http("http://localhost:1/mcp", "session"),
            ];
            let strategy_servers = vec![
                McpServer::from(
                    McpServerHttp::new("http://localhost:2/mcp", "shared")
                        .with_allowed_tools(vec!["only".to_owned()]),
                ),
                McpServer::http("http://localhost:2/mcp", "strategy"),
            ];

            let llm = ScriptedLlm::new(script);
            let agent: Agent<_> = Agent::builder()
                .mcp_servers(default_servers)
                .build(llm.clone());
            let mut session = agent.session();

            let strategy = AddMcpServers(strategy_servers);
            session
                .run_turn_with_strategy(strategy, "Hi.")
                .await
                .unwrap();

            let mut servers = llm.requests().pop().unwrap().mcp_servers;
            servers.sort_by(|a, b| a.name().cmp(b.name()));
            let names = servers.iter().map(|s| s.name()).collect::<Vec<_>>();
            assert_eq!(names, ["session", "shared", "strategy"]);
            assert_eq!(
                servers[1].allowed_tools(),
                Some(["only".to_owned()].as_slice()),
                "the strategy's server replaces the session's"
            );

            // The strategy's servers are not kept by the session.
            session.run_turn("Hi.").await.unwrap();
            let servers = llm.requests().pop().unwrap().mcp_servers;
            let mut names = servers.iter().map(|s| s.name()).collect::<Vec<_>>();
            names.sort();
            assert_eq!(names, ["session", "shared"]);
            assert_eq!(
                servers
                    .iter()
                    .find(|s| s.name() == "shared")
                    .unwrap()
                    .allowed_tools(),
                None
            );
        }

        /// A [`Strategy`] that adds skills to the run.
        #[derive(Clone)]
        struct AddSkills(SkillSet);

        #[async_trait::async_trait]
        impl<L: Llm, O: OutputCtor> Strategy<L, O> for AddSkills {
            type Output = ();
            type Error = std::convert::Infallible;

            fn additional_skills(&self) -> Option<SkillSet> {
                Some(self.0.clone())
            }

            async fn finalize(self) -> Result<Self::Output, Self::Error> {
                Ok(())
            }
        }

        #[tokio::test]
        async fn strategy_adds_skills() {
            let skill = MemorySkill::new(SkillMetadata::new("pdf", "pdf skill").unwrap(), "");
            let skill: Arc<dyn Skill> = Arc::new(skill);

            let script = [
                ScriptedMessage::new(AssistantMessage::new(None, "one")),
                ScriptedMessage::new(AssistantMessage::new(None, "two")),
            ];

            let llm = ScriptedLlm::new(script);
            let agent: Agent<_> = Agent::builder().build(llm.clone());
            let mut session = agent.session();

            // The session has no skills, but the strategy does.
            let skills = SkillSet::from_iter([skill]);
            let strategy = AddSkills(skills);

            // Strategy skills offered for the first turn.
            let one = session
                .run_turn_with_strategy(strategy, "Hi.")
                .await
                .unwrap();
            assert_eq!(one.content, "one");
            let request = llm.requests().pop().unwrap();
            assert!(tool_names(&request.tools).contains(&"load_skill"));

            // No skills offered for the second turn.
            assert!(session.skills.is_empty());
            let two = session.run_turn("Hi.").await.unwrap();
            assert_eq!(two, "two");
            let request = llm.requests().pop().unwrap();
            assert!(!tool_names(&request.tools).contains(&"load_skill"));
        }
    }
}
