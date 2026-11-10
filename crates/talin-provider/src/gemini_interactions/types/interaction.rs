use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, PickFirst, serde_as, skip_serializing_none};

use super::{
    AgentConfig, AgentOption, Environment, ErrorMessage, FunctionCallStep, GenerationConfig,
    InteractionsInput, ModelOption, ResponseFormat, ResponseModality, SafetySetting, Step, Tool,
    Usage,
};
use crate::utils::string_enum;

/// The Interaction resource.
///
/// An interaction is the stateful, step-based unit of the Interactions API. It is returned
/// when creating, retrieving, or cancelling an interaction.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Interaction {
    /// Opaque token to resume a long decode when the status is `requires_continuation`. Pass it
    /// back unchanged in the next create request to continue decoding.
    #[cfg_attr(test, schema(value_type = String, format = Byte, nullable = false))]
    pub continuation_token: Option<String>,

    /// The name of the `Agent` used for generating the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub agent: Option<AgentOption>,

    /// Configuration parameters for the agent interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub agent_config: Option<AgentConfig>,

    /// The time at which the response was created, in ISO 8601 format.
    pub created: String,

    /// The environment the interaction runs in.
    #[cfg_attr(test, schema(nullable = false))]
    pub environment: Option<Environment>,

    /// The ID of the environment the interaction ran in, if the request set an environment.
    #[cfg_attr(test, schema(nullable = false))]
    pub environment_id: Option<String>,

    /// Diagnostic faults and platform errors recorded on the interaction (output only).
    #[cfg_attr(test, schema(nullable = false))]
    pub errors: Option<Vec<ErrorMessage>>,

    /// Configuration parameters for the model interaction (input only).
    #[cfg_attr(test, schema(nullable = false))]
    pub generation_config: Option<GenerationConfig>,

    /// A unique identifier for the interaction.
    pub id: String,

    /// The input for the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub input: Option<InteractionsInput>,

    /// User-defined metadata for the request.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::map_schema, nullable = false))]
    pub labels: Option<BTreeMap<String, String>>,

    /// The maximum number of tokens the interaction may use in total.
    #[serde_as(as = "Option<PickFirst<(DisplayFromStr, _)>>")]
    #[cfg_attr(test, schema(value_type = String, format = Int64, nullable = false))]
    pub max_total_tokens: Option<i64>,

    /// The name of the `Model` used for generating the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub model: Option<ModelOption>,

    /// The ID of the previous interaction, if any.
    #[cfg_attr(test, schema(nullable = false))]
    pub previous_interaction_id: Option<String>,

    /// The response format(s) enforced on the generated response.
    #[cfg_attr(test, schema(nullable = false))]
    pub response_format: Option<ResponseFormats>,

    /// The mime type of the response (deprecated; use `response_format`).
    #[deprecated(note = "use `response_format` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub response_mime_type: Option<String>,

    /// The requested modalities of the response.
    #[deprecated(note = "use `response_format` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub response_modalities: Option<Vec<ResponseModality>>,

    /// Safety settings for the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub safety_settings: Option<Vec<SafetySetting>>,

    /// The service tier the interaction is served on.
    #[cfg_attr(test, schema(nullable = false))]
    pub service_tier: Option<ServiceTier>,

    /// The status of the interaction.
    pub status: InteractionStatus,

    /// The steps that make up the interaction.
    pub steps: Vec<Step>,

    /// System instruction for the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub system_instruction: Option<String>,

    /// A list of tool declarations the model may call during the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub tools: Option<Vec<Tool>>,

    /// The time at which the response was last updated, in ISO 8601 format.
    pub updated: String,

    /// Statistics on the interaction request's token usage.
    #[cfg_attr(test, schema(nullable = false))]
    pub usage: Option<Usage>,

    /// Where to deliver notifications when the interaction completes.
    #[cfg_attr(test, schema(nullable = false))]
    pub webhook_config: Option<WebhookConfig>,
}

string_enum! {
    /// The status of an interaction.
    #[serde(rename_all = "snake_case")]
    pub enum InteractionStatus {
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

        /// The interaction failed due to exceeding the budget.
        BudgetExceeded,

        /// The interaction stopped early and can be resumed with a `continuation_token`.
        RequiresContinuation,
    }
}

/// Where to deliver notifications about an interaction.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct WebhookConfig {
    /// The webhook URIs to use for this interaction, in place of the registered webhooks.
    #[cfg_attr(test, schema(nullable = false))]
    pub uris: Option<Vec<String>>,

    /// Metadata returned on each event emitted to the webhooks.
    #[cfg_attr(
        test,
        schema(schema_with = crate::gemini_interactions::spec::schema::object_schema)
    )]
    pub user_metadata: Option<serde_json::Value>,
}

string_enum! {
    /// The service tier an interaction is served on.
    #[serde(rename_all = "snake_case")]
    pub enum ServiceTier {
        /// Served on the flex tier.
        Flex,

        /// Served on the standard tier.
        Standard,

        /// Served on the priority tier.
        Priority,
    }
}

/// One or more [`ResponseFormat`]s.
///
/// The API accepts either a single response format or a list of them.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ResponseFormats {
    /// A single response format.
    One(ResponseFormat),
    /// A list of response formats.
    #[cfg_attr(test, schema(title = "ResponseFormatList"))]
    Many(Vec<ResponseFormat>),
}

impl Interaction {
    /// Returns the concatenated output text of the interaction.
    ///
    /// Concatenates the text content of the last model output step.
    pub fn text(&self) -> Option<String> {
        let step = self.steps.iter().rev().find_map(Step::as_model_output)?;

        let text: String = step
            .iter()
            .filter_map(|content| content.as_text())
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() { None } else { Some(text) }
    }

    /// Returns `true` if the interaction has completed.
    pub fn is_completed(&self) -> bool {
        matches!(self.status, InteractionStatus::Completed)
    }

    /// Returns `true` if the interaction requires action (e.g. a function result) from the
    /// caller.
    pub fn requires_action(&self) -> bool {
        matches!(self.status, InteractionStatus::RequiresAction)
    }

    /// Returns an iterator over the function calls requested across all steps.
    pub fn function_calls(&self) -> impl Iterator<Item = &FunctionCallStep> {
        self.steps.iter().filter_map(|step| match step {
            Step::FunctionCall(call) => Some(call),
            _ => None,
        })
    }
}

impl WebhookConfig {
    /// Creates a webhook configuration delivering to the given URIs.
    pub fn new(uris: Vec<String>) -> Self {
        Self {
            uris: Some(uris),
            user_metadata: None,
        }
    }

    /// Sets the metadata returned on each event.
    pub fn with_user_metadata(mut self, user_metadata: serde_json::Value) -> Self {
        self.user_metadata = Some(user_metadata);
        self
    }
}

impl From<ResponseFormat> for ResponseFormats {
    fn from(value: ResponseFormat) -> Self {
        ResponseFormats::One(value)
    }
}

impl From<Vec<ResponseFormat>> for ResponseFormats {
    fn from(value: Vec<ResponseFormat>) -> Self {
        ResponseFormats::Many(value)
    }
}
