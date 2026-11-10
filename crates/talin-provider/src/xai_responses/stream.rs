use futures::StreamExt;

use crate::error::StreamError;
use crate::stream::{Merge, PartialResponse, PreferNew, ResponseStream, or_partial};
use crate::xai_responses::types::{
    CodeInterpreterCall, CodeInterpreterOutput, CustomToolCall, FileSearchCall, FileSearchResult,
    FunctionToolCall, ImageGenerationCall, McpCall, ModelOutput, ModelResponse,
    ModelResponseConfiguration, OutputMessage, OutputMessageContent, OutputTextContent, Reasoning,
    RefusalContent, ShellCall, ShellCallAction, ShellCallActionExec, ShellEnvironment, StreamEvent,
    WebSearchAction, WebSearchCall,
};

/// Assembles a complete [`ModelResponse`] from a stream of [`StreamEvent`]s.
#[allow(clippy::result_large_err)]
pub(super) async fn assemble(
    mut stream: ResponseStream<StreamEvent, serde_json::Value>,
) -> Result<ModelResponse, PartialResponse<ModelResponse, serde_json::Value>> {
    let mut response = match stream.next().await {
        Some(Ok(StreamEvent::Created(event))) => event.response,
        Some(Ok(event)) => match event {
            StreamEvent::InProgress(e) | StreamEvent::Completed(e) => e.response,
            _ => {
                let err = StreamError::invalid_event("Expected response.created event");
                return Err(PartialResponse::new(None, err));
            }
        },
        Some(Err(e)) => {
            return Err(PartialResponse::new(None, e));
        }
        None => return Err(PartialResponse::new(None, StreamError::UnexpectedClose)),
    };

    while let Some(evt) = stream.next().await {
        let evt = or_partial!(evt, response);
        response.merge(evt);
    }
    Ok(response)
}

impl Merge<StreamEvent> for ModelResponse {
    fn merge(&mut self, rhs: StreamEvent) {
        match rhs {
            StreamEvent::Created(e) | StreamEvent::InProgress(e) | StreamEvent::Completed(e) => {
                self.merge(e.response);
            }
            StreamEvent::OutputItemAdded(e) => {
                if e.output_index < self.output.len() {
                    self.output[e.output_index] = e.item;
                } else {
                    self.output.push(e.item);
                }
            }
            StreamEvent::OutputItemDone(e) => {
                if e.output_index < self.output.len() {
                    self.output[e.output_index] = e.item;
                } else {
                    self.output.push(e.item);
                }
            }
            StreamEvent::ContentPartAdded(e) => {
                if let Some(ModelOutput::Message(msg)) = self.output.get_mut(e.output_index) {
                    if e.content_index < msg.content.len() {
                        msg.content[e.content_index] = e.part;
                    } else {
                        msg.content.push(e.part);
                    }
                }
            }
            StreamEvent::ContentPartDone(e) => {
                if let Some(ModelOutput::Message(msg)) = self.output.get_mut(e.output_index) {
                    if e.content_index < msg.content.len() {
                        msg.content[e.content_index] = e.part;
                    } else {
                        msg.content.push(e.part);
                    }
                }
            }
            StreamEvent::OutputTextDelta(e) => {
                if let Some(ModelOutput::Message(msg)) = self.output.get_mut(e.output_index)
                    && let Some(OutputMessageContent::OutputText(text_content)) =
                        msg.content.get_mut(e.content_index)
                {
                    text_content.text.push_str(&e.delta);
                    text_content.logprobs.merge(e.logprobs);
                }
            }
            StreamEvent::OutputTextDone(e) => {
                if let Some(ModelOutput::Message(msg)) = self.output.get_mut(e.output_index)
                    && let Some(OutputMessageContent::OutputText(text_content)) =
                        msg.content.get_mut(e.content_index)
                {
                    text_content.text = e.text;
                }
            }
            // An event this crate does not model carries nothing it knows how to apply.
            StreamEvent::Unknown(_) => {}
        }
    }
}

/// Merges two [`ModelResponse`] values by merging their fields appropriately.
impl Merge<ModelResponse> for ModelResponse {
    fn merge(&mut self, rhs: ModelResponse) {
        self.background = rhs.background;
        self.completed_at.merge(rhs.completed_at);
        self.created_at.merge(rhs.created_at);
        self.error.merge(PreferNew(rhs.error));
        self.frequency_penalty.merge(rhs.frequency_penalty);
        self.id.merge(rhs.id);
        self.incomplete_details
            .merge(PreferNew(rhs.incomplete_details));
        self.instructions.merge(rhs.instructions);
        self.max_output_tokens.merge(rhs.max_output_tokens);
        self.max_tool_calls.merge(rhs.max_tool_calls);
        self.metadata = rhs.metadata;
        self.model = rhs.model;
        // object is constant

        for (idx, chunk_output) in rhs.output.into_iter().enumerate() {
            if idx < self.output.len() {
                self.output[idx].merge(chunk_output);
            } else {
                self.output.push(chunk_output);
            }
        }

        self.parallel_tool_calls = rhs.parallel_tool_calls;
        self.presence_penalty.merge(rhs.presence_penalty);
        self.previous_response_id.merge(rhs.previous_response_id);
        self.prompt_cache_key.merge(rhs.prompt_cache_key);
        self.reasoning.merge(PreferNew(rhs.reasoning));
        self.safety_identifier.merge(rhs.safety_identifier);

        self.service_tier = rhs.service_tier;
        self.status = rhs.status;
        self.store = rhs.store;
        self.temperature.merge(rhs.temperature);
        self.text.merge(rhs.text);

        self.tool_choice = rhs.tool_choice;
        self.tools = rhs.tools;

        self.top_logprobs = rhs.top_logprobs;
        self.top_p.merge(rhs.top_p);
        self.truncation = rhs.truncation;

        self.usage.merge(PreferNew(rhs.usage));
        self.user.merge(rhs.user);
    }
}

/// Merges two [`ModelOutput`] values by merging their inner variants appropriately.
impl Merge<ModelOutput> for ModelOutput {
    fn merge(&mut self, rhs: ModelOutput) {
        match (self, rhs) {
            (ModelOutput::Message(lhs), ModelOutput::Message(rhs)) => lhs.merge(rhs),
            (ModelOutput::FunctionCall(lhs), ModelOutput::FunctionCall(rhs)) => lhs.merge(rhs),
            (ModelOutput::Reasoning(lhs), ModelOutput::Reasoning(rhs)) => lhs.merge(rhs),
            (ModelOutput::WebSearchCall(lhs), ModelOutput::WebSearchCall(rhs)) => lhs.merge(rhs),
            (ModelOutput::FileSearchCall(lhs), ModelOutput::FileSearchCall(rhs)) => lhs.merge(rhs),
            (ModelOutput::CodeInterpreterCall(lhs), ModelOutput::CodeInterpreterCall(rhs)) => {
                lhs.merge(rhs)
            }
            (ModelOutput::McpCall(lhs), ModelOutput::McpCall(rhs)) => lhs.merge(rhs),
            (ModelOutput::CustomToolCall(lhs), ModelOutput::CustomToolCall(rhs)) => lhs.merge(rhs),
            (ModelOutput::ShellCall(lhs), ModelOutput::ShellCall(rhs)) => lhs.merge(rhs),
            (ModelOutput::ImageGenerationCall(lhs), ModelOutput::ImageGenerationCall(rhs)) => {
                lhs.merge(rhs)
            }
            _ => {}
        }
    }
}

/// Merges two [`OutputMessage`] values by merging their fields appropriately.
impl Merge<OutputMessage> for OutputMessage {
    fn merge(&mut self, rhs: OutputMessage) {
        for (idx, item) in rhs.content.into_iter().enumerate() {
            if idx < self.content.len() {
                self.content[idx].merge(item);
            } else {
                self.content.push(item);
            }
        }

        self.id.merge(PreferNew(rhs.id));
        self.role = rhs.role;
        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`OutputMessageContent`] values by merging their inner variants appropriately.
impl Merge<OutputMessageContent> for OutputMessageContent {
    fn merge(&mut self, rhs: OutputMessageContent) {
        match (self, rhs) {
            (
                OutputMessageContent::OutputText(text),
                OutputMessageContent::OutputText(rhs_text),
            ) => text.merge(rhs_text),
            (
                OutputMessageContent::Refusal(refusal),
                OutputMessageContent::Refusal(rhs_refusal),
            ) => refusal.merge(rhs_refusal),
            _ => {}
        }
    }
}

/// Merges two [`OutputTextContent`] values by concatenating text and merging optional fields.
impl Merge<OutputTextContent> for OutputTextContent {
    fn merge(&mut self, rhs: OutputTextContent) {
        self.text.push_str(&rhs.text);
        self.annotations.merge(rhs.annotations);
        self.logprobs.merge(rhs.logprobs);
    }
}

/// Merges two [`RefusalContent`] values by concatenating refusal reasons.
impl Merge<RefusalContent> for RefusalContent {
    fn merge(&mut self, rhs: RefusalContent) {
        self.refusal.push_str(&rhs.refusal);
    }
}

/// Merges two [`FunctionToolCall`] values by concatenating arguments and merging optional fields.
impl Merge<FunctionToolCall> for FunctionToolCall {
    fn merge(&mut self, rhs: FunctionToolCall) {
        self.arguments.push_str(&rhs.arguments);
        self.call_id = rhs.call_id;
        self.id.merge(PreferNew(rhs.id));
        self.name = rhs.name;
        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`Reasoning`] values by merging their fields appropriately.
impl Merge<Reasoning> for Reasoning {
    fn merge(&mut self, rhs: Reasoning) {
        self.id.merge(rhs.id);
        self.encrypted_content.merge(rhs.encrypted_content);
        self.status.merge(PreferNew(rhs.status));
        self.content.merge(rhs.content);
        self.summary.extend(rhs.summary);
    }
}

/// Merges two [`WebSearchCall`] values by merging their fields appropriately.
impl Merge<WebSearchCall> for WebSearchCall {
    fn merge(&mut self, rhs: WebSearchCall) {
        // WebSearchAction merge
        match (&mut self.action, rhs.action) {
            (WebSearchAction::Search(lhs), WebSearchAction::Search(rhs)) => {
                lhs.query.push_str(&rhs.query);
                lhs.sources.merge(rhs.sources);
            }
            (WebSearchAction::OpenPage(lhs), WebSearchAction::OpenPage(rhs)) => {
                lhs.url.push_str(&rhs.url);
            }
            (WebSearchAction::Find(lhs), WebSearchAction::Find(rhs)) => {
                lhs.pattern.push_str(&rhs.pattern);
                lhs.url.push_str(&rhs.url);
            }
            _ => {}
        }

        self.id.merge(PreferNew(rhs.id));
        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`FileSearchCall`] values by merging their fields appropriately.
impl Merge<FileSearchCall> for FileSearchCall {
    fn merge(&mut self, rhs: FileSearchCall) {
        self.id.merge(PreferNew(rhs.id));
        self.queries.extend(rhs.queries);

        // Merge results
        for (idx, item) in rhs.results.into_iter().enumerate() {
            if idx < self.results.len() {
                self.results[idx].merge(item);
            } else {
                self.results.push(item);
            }
        }

        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`FileSearchResult`] values by merging their fields appropriately.
impl Merge<FileSearchResult> for FileSearchResult {
    fn merge(&mut self, rhs: FileSearchResult) {
        self.file_id = rhs.file_id;
        self.filename = rhs.filename;
        self.score.merge(rhs.score);
        self.text.push_str(&rhs.text);
    }
}

/// Merges two [`CodeInterpreterCall`] values by merging their fields appropriately.
impl Merge<CodeInterpreterCall> for CodeInterpreterCall {
    fn merge(&mut self, rhs: CodeInterpreterCall) {
        self.code.merge(rhs.code);
        self.id.merge(PreferNew(rhs.id));
        for (idx, item) in rhs.outputs.into_iter().enumerate() {
            if idx < self.outputs.len() {
                match (&mut self.outputs[idx], item) {
                    (CodeInterpreterOutput::Logs(lhs), CodeInterpreterOutput::Logs(rhs)) => {
                        lhs.logs.push_str(&rhs.logs);
                    }
                    (CodeInterpreterOutput::Image(lhs), CodeInterpreterOutput::Image(rhs)) => {
                        lhs.url.push_str(&rhs.url);
                    }
                    _ => {}
                }
            } else {
                self.outputs.push(item);
            }
        }
        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`McpCall`] values by merging their fields appropriately.
impl Merge<McpCall> for McpCall {
    fn merge(&mut self, rhs: McpCall) {
        self.arguments.push_str(&rhs.arguments);
        self.error.merge(rhs.error);
        self.id.merge(PreferNew(rhs.id));
        self.name = rhs.name;
        self.output.push_str(&rhs.output);
        self.server_label = rhs.server_label;
        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`CustomToolCall`] values by concatenating input and merging optional fields.
impl Merge<CustomToolCall> for CustomToolCall {
    fn merge(&mut self, rhs: CustomToolCall) {
        self.call_id = rhs.call_id;
        self.id = rhs.id;
        self.input.merge(rhs.input);
        self.name = rhs.name;
        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`ShellCall`] values by merging their fields appropriately.
impl Merge<ShellCall> for ShellCall {
    fn merge(&mut self, rhs: ShellCall) {
        self.action.merge(rhs.action);
        self.call_id = rhs.call_id;
        self.environment.merge(rhs.environment);
        self.id.merge(PreferNew(rhs.id));
        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`ShellCallAction`] values by merging their inner variants appropriately.
impl Merge<ShellCallAction> for ShellCallAction {
    fn merge(&mut self, rhs: ShellCallAction) {
        if let (ShellCallAction::Exec(lhs), ShellCallAction::Exec(rhs)) = (self, rhs) {
            lhs.merge(rhs)
        }
    }
}

/// Merges two [`ShellCallActionExec`] values by appending commands and merging optional fields.
impl Merge<ShellCallActionExec> for ShellCallActionExec {
    fn merge(&mut self, rhs: ShellCallActionExec) {
        self.commands.extend(rhs.commands);
        self.max_output_length.merge(rhs.max_output_length);
        self.timeout_ms.merge(rhs.timeout_ms);
    }
}

/// Merges two [`ShellEnvironment`] values by replacing with the rhs value.
impl Merge<ShellEnvironment> for ShellEnvironment {
    fn merge(&mut self, rhs: ShellEnvironment) {
        *self = rhs;
    }
}

/// Merges two [`ImageGenerationCall`] values by merging their fields appropriately.
impl Merge<ImageGenerationCall> for ImageGenerationCall {
    fn merge(&mut self, rhs: ImageGenerationCall) {
        self.id.merge(rhs.id);
        self.prompt.merge(rhs.prompt);
        // The result is a self-contained base64 payload, not a delta, so the latest one wins.
        self.result.merge(rhs.result);
        self.status.merge(PreferNew(rhs.status));
    }
}

/// Merges two [`ModelResponseConfiguration`] values by merging their fields appropriately.
impl Merge<ModelResponseConfiguration> for ModelResponseConfiguration {
    fn merge(&mut self, rhs: ModelResponseConfiguration) {
        self.format.merge(PreferNew(rhs.format));
    }
}
