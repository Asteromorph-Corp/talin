use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{ErrorMessage, InteractionStatus, ServiceTier, Step, StepDeltaData, Usage};
use crate::utils::tagged_enum;

tagged_enum! {
    /// A server-sent event emitted while streaming an interaction.
    ///
    /// Discriminated by its `event_type` field.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "event_type")]
    pub enum InteractionSseEvent {
        /// Emitted when the interaction is created.
        #[serde(rename = "interaction.created")]
        InteractionCreated(InteractionCreatedEvent),
        /// Emitted at the end of the stream with the completed interaction.
        #[serde(rename = "interaction.completed")]
        InteractionCompleted(InteractionCompletedEvent),
        /// Emitted when the interaction status changes.
        #[serde(rename = "interaction.status_update")]
        StatusUpdate(InteractionStatusUpdate),
        /// Emitted when an error occurs during streaming.
        #[serde(rename = "error")]
        Error(ErrorEvent),
        /// Emitted when a step starts.
        #[serde(rename = "step.start")]
        StepStart(StepStart),
        /// Emitted with an incremental update to a step.
        #[serde(rename = "step.delta")]
        StepDelta(StepDelta),
        /// Emitted when a step stops.
        #[serde(rename = "step.stop")]
        StepStop(StepStop),
    }
}

/// Emitted when the interaction is created.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InteractionCreatedEvent {
    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The partial interaction resource emitted when the stream is created.
    pub interaction: InteractionSseEventInteraction,
}

/// Emitted at the end of the stream with the completed interaction.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InteractionCompletedEvent {
    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The partial completed interaction resource emitted at the end of the stream.
    pub interaction: InteractionSseEventInteraction,
}

/// Emitted when the interaction status changes.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InteractionStatusUpdate {
    /// An opaque token to resume decoding from the latest checkpoint after a disconnected
    /// stream.
    #[cfg_attr(test, schema(value_type = String, format = Byte, nullable = false))]
    pub continuation_token: Option<String>,

    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The ID of the interaction whose status changed.
    pub interaction_id: String,

    /// The new status of the interaction.
    pub status: InteractionStatus,

    /// Model usage statistics up to this checkpoint.
    #[cfg_attr(test, schema(nullable = false))]
    pub usage: Option<Usage>,
}

/// Emitted when an error occurs during streaming.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ErrorEvent {
    /// The error that occurred.
    #[cfg_attr(test, schema(nullable = false))]
    pub error: Option<ErrorMessage>,

    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,
}

/// Emitted when a step starts.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct StepStart {
    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The index of the step.
    pub index: i32,

    /// The step that is starting.
    pub step: Step,
}

/// Emitted with an incremental update to a step.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct StepDelta {
    /// The incremental step data.
    pub delta: StepDeltaData,

    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The index of the step.
    pub index: i32,

    /// Metadata accompanying this step delta.
    #[cfg_attr(test, schema(nullable = false))]
    pub metadata: Option<StepDeltaMetadata>,
}

/// Emitted when a step stops.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct StepStop {
    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The index of the step.
    pub index: i32,

    /// Model usage stats for this specific step.
    #[cfg_attr(test, schema(nullable = false))]
    pub step_usage: Option<Usage>,

    /// Cumulative model usage stats from the start of the session.
    #[cfg_attr(test, schema(nullable = false))]
    pub usage: Option<Usage>,
}

/// A partial interaction resource emitted by interaction lifecycle SSE events.
///
/// Streaming lifecycle payloads may omit fields that are only available on full,
/// non-streaming [`Interaction`](super::Interaction) responses.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InteractionSseEventInteraction {
    /// The agent used to interact.
    #[cfg_attr(test, schema(nullable = false))]
    pub agent: Option<String>,

    /// The time at which the response was created, in ISO 8601 format.
    #[cfg_attr(test, schema(nullable = false, read_only))]
    pub created: Option<String>,

    /// Opaque token to resume a long decode when the status is `incomplete`.
    #[cfg_attr(test, schema(value_type = String, format = Byte, nullable = false, read_only))]
    pub continuation_token: Option<String>,

    /// A unique identifier for the interaction.
    #[cfg_attr(test, schema(read_only))]
    pub id: String,

    /// The model that will complete the prompt.
    #[cfg_attr(test, schema(nullable = false))]
    pub model: Option<String>,

    /// The name of the resource, always `interaction`.
    #[cfg_attr(test, schema(nullable = false, read_only))]
    pub object: Option<String>,

    /// The service tier the interaction is served on.
    #[cfg_attr(test, schema(nullable = false))]
    pub service_tier: Option<ServiceTier>,

    /// The status of the interaction.
    #[cfg_attr(test, schema(schema_with = status_schema))]
    pub status: InteractionStatus,

    /// The steps that make up the interaction, if included in this event.
    #[cfg_attr(test, schema(schema_with = steps_schema))]
    pub steps: Option<Vec<Step>>,

    /// The time at which the response was last updated, in ISO 8601 format.
    #[cfg_attr(test, schema(nullable = false, read_only))]
    pub updated: Option<String>,

    /// Statistics on the interaction request's token usage.
    #[cfg_attr(test, schema(nullable = false, read_only))]
    pub usage: Option<Usage>,
}

/// Optional metadata accompanying a streamed step-delta event.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct StepDeltaMetadata {
    /// Statistics on the interaction request's token usage.
    #[cfg_attr(test, schema(nullable = false))]
    pub total_usage: Option<Usage>,
}

/// The wire envelope for a single interaction SSE event (`{ "data": ... }`).
#[cfg_attr(test, derive(utoipa::ToSchema), schema(as = InteractionSseStreamEvent))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InteractionSseStreamEnvelope {
    /// The event payload.
    pub data: InteractionSseEvent,
}

/// [`InteractionStatus`] without `"budget_exceeded"` or `"queued"` variant.
// The variants are missing for the streaming event status, which is likely an oversight in the
// upstream spec. We use private enum for spec generation only.
#[cfg(test)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
enum StreamInteractionStatus {
    /// The interaction is in progress.
    InProgress,
    /// The interaction requires action/input from the user.
    RequiresAction,
    /// The interaction is completed.
    Completed,
    /// The interaction failed.
    Failed,
    /// The interaction was cancelled.
    Cancelled,
    /// The interaction is completed, but contains incomplete results (e.g. hitting
    /// `max_output_tokens`).
    Incomplete,
}

impl InteractionSseEvent {
    /// Returns the completed interaction payload, if this is a completion event.
    pub fn as_completed(&self) -> Option<&InteractionSseEventInteraction> {
        match self {
            InteractionSseEvent::InteractionCompleted(e) => Some(&e.interaction),
            _ => None,
        }
    }

    /// Returns the incremental text of this event, if it carries a text step delta.
    pub fn text_delta(&self) -> Option<&str> {
        match self {
            InteractionSseEvent::StepDelta(e) => match &e.delta {
                StepDeltaData::Text(t) => Some(&t.text),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Generates the schema for [`InteractionSseEventInteraction::status`].
///
/// [`StreamInteractionStatus`] written inline and marked read-only, which is how the API
/// describes the field. `utoipa` renders a field whose type is a schema of its own as a
/// `$ref`, and a `$ref` carries neither the enumeration nor `readOnly`.
#[cfg(test)]
fn status_schema() -> utoipa::openapi::schema::Schema {
    use utoipa::PartialSchema;
    use utoipa::openapi::RefOr;
    use utoipa::openapi::schema::Schema;

    match StreamInteractionStatus::schema() {
        RefOr::T(Schema::Object(mut object)) => {
            object.read_only = Some(true);
            Schema::Object(object)
        }
        _ => unreachable!("a plain enum derives an object schema"),
    }
}

/// Generates the schema for [`InteractionSseEventInteraction::steps`].
///
/// An array of [`Step`] marked read-only, which is how the API describes the field.
/// `utoipa` drops `read_only` on an array — it applies the feature to the items, which are a
/// `$ref` and cannot carry it — and `utoipa`'s [`Array`](utoipa::openapi::schema::Array) has
/// no `read_only` of its own, so the keyword is written as an extension. Extensions are
/// emitted verbatim, which is also how the variant tags become `const`.
#[cfg(test)]
fn steps_schema() -> utoipa::openapi::schema::Schema {
    use utoipa::openapi::extensions::Extensions;
    use utoipa::openapi::schema::{ArrayItems, Schema};
    use utoipa::openapi::{ArrayBuilder, Ref, RefOr};

    let items = ArrayItems::RefOrSchema(Box::new(RefOr::Ref(Ref::from_schema_name("Step"))));
    Schema::Array(
        ArrayBuilder::new()
            .items(items)
            .extensions(Some(Extensions::from_iter([("readOnly", true)])))
            .build(),
    )
}
