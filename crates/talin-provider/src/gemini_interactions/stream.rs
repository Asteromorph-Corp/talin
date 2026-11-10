use futures::StreamExt;

use crate::error::{ApiError, StreamError};
use crate::gemini_interactions::types::{
    Content, ErrorMessage, Interaction, InteractionSseEvent, InteractionSseEventInteraction,
    InteractionSseStreamEnvelope, ModelOption, Step, StepDeltaData,
};
use crate::stream::{PartialResponse, ResponseStream};

macro_rules! or_build_partial {
    ($body:expr) => {
        match $body {
            Ok(val) => val,
            Err(e) => return Err(PartialResponse::new(None, e)),
        }
    };
    ($body:expr, ? $partial:expr, $output_text:expr, $collected_steps:expr) => {
        match $body {
            Ok(val) => val,
            Err(e) => {
                let partial = $partial.map(|partial| {
                    $crate::gemini_interactions::stream::build_interaction(
                        partial,
                        $output_text,
                        $collected_steps,
                    )
                });
                return Err(PartialResponse::new(partial, e));
            }
        }
    };
}

/// Assembles a complete [`Interaction`] from a stream of [`InteractionSseStreamEnvelope`]s.
pub(super) async fn assemble(
    mut stream: ResponseStream<InteractionSseStreamEnvelope, ErrorMessage>,
) -> Result<Interaction, PartialResponse<Interaction, ErrorMessage>> {
    let mut output_text = String::new();
    let mut collected_steps: Vec<Step> = Vec::new();
    let mut latest: Option<InteractionSseEventInteraction> = None;

    while let Some(evt) = stream.next().await {
        let evt = or_build_partial!(evt, ?latest, output_text, collected_steps);
        match evt.data {
            InteractionSseEvent::InteractionCreated(event) => latest = Some(event.interaction),
            InteractionSseEvent::InteractionCompleted(event) => {
                return Ok(build_interaction(
                    event.interaction,
                    output_text,
                    collected_steps,
                ));
            }
            InteractionSseEvent::StepStart(event) => collected_steps.push(event.step),
            InteractionSseEvent::StepDelta(event) => {
                if let StepDeltaData::Text(text) = &event.delta {
                    output_text.push_str(&text.text);
                }
            }
            InteractionSseEvent::Error(event) => {
                let partial = latest.map(|interaction| {
                    build_interaction(interaction, output_text, collected_steps)
                });
                let error = event.error.unwrap_or_default();
                return Err(PartialResponse::new(partial, ApiError::new(error)));
            }
            _ => {}
        }
    }

    // The stream ended without a completion event; fall back to the latest lifecycle payload.
    match latest {
        Some(interaction) => Ok(build_interaction(interaction, output_text, collected_steps)),
        None => Err(PartialResponse::new(None, StreamError::UnexpectedClose)),
    }
}

/// Builds a full [`Interaction`] from a streamed lifecycle payload plus accumulated state.
#[allow(deprecated)]
fn build_interaction(
    partial: InteractionSseEventInteraction,
    output_text: String,
    collected_steps: Vec<Step>,
) -> Interaction {
    let mut steps = match partial.steps {
        Some(steps) if !steps.is_empty() => steps,
        _ => collected_steps,
    };

    // The streamed steps carry no text of their own, so keep the text the deltas delivered.
    let has_text = steps.iter().any(|step| {
        step.as_model_output()
            .is_some_and(|content| content.iter().any(|c| c.as_text().is_some()))
    });
    if !has_text && !output_text.is_empty() {
        steps.push(Step::model_output(vec![Content::text(output_text)]));
    }

    Interaction {
        agent: partial.agent.map(Into::into),
        agent_config: None,
        continuation_token: partial.continuation_token,
        created: partial.created.unwrap_or_default(),
        environment: None,
        environment_id: None,
        errors: None,
        generation_config: None,
        id: partial.id,
        max_total_tokens: None,
        input: None,
        labels: None,
        model: partial.model.map(ModelOption::from),
        previous_interaction_id: None,
        response_format: None,
        response_mime_type: None,
        response_modalities: None,
        safety_settings: None,
        service_tier: partial.service_tier,
        status: partial.status,
        steps,
        system_instruction: None,
        tools: None,
        updated: partial.updated.unwrap_or_default(),
        usage: partial.usage,
        webhook_config: None,
    }
}
