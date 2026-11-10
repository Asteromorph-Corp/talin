use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod input;

pub use input::InteractionsInput;

use super::{
    AgentConfig, AgentOption, Environment, GenerationConfig, ModelOption, ResponseFormats,
    ResponseModality, SafetySetting, ServiceTier, Tool, WebhookConfig,
};

/// The body of a request to create an interaction.
///
/// An interaction is created either against a [`ModelOption`] (a raw Gemini model) or an
/// [`AgentOption`] (a managed agent).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum CreateInteractionRequest {
    /// Parameters for creating an agent interaction.
    Agent(Box<AgentInteraction>),
    /// Parameters for creating a model interaction.
    Model(Box<ModelInteraction>),
}

/// Parameters for creating a model interaction.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema), schema(as = CreateModelInteractionParams))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ModelInteraction {
    /// The name of the `Model` used for generating the interaction.
    pub model: ModelOption,

    /// The input for the interaction.
    pub input: InteractionsInput,

    /// Configuration parameters for the model interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub generation_config: Option<GenerationConfig>,

    /// System instruction for the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub system_instruction: Option<String>,

    /// A list of tool declarations the model may call during the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub tools: Option<Vec<Tool>>,

    /// The response format(s) enforced on the generated response.
    #[cfg_attr(test, schema(nullable = false))]
    pub response_format: Option<ResponseFormats>,

    /// The mime type of the response (deprecated; use `response_format`).
    #[deprecated(note = "use `response_format` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub response_mime_type: Option<String>,

    /// Safety settings for the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub safety_settings: Option<Vec<SafetySetting>>,

    /// User-defined metadata for the request.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::map_schema, nullable = false))]
    pub labels: Option<BTreeMap<String, String>>,

    /// The environment the interaction runs in.
    #[cfg_attr(test, schema(nullable = false))]
    pub environment: Option<Environment>,

    /// The requested modalities of the response.
    #[deprecated(note = "use `response_format` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub response_modalities: Option<Vec<ResponseModality>>,

    /// The service tier to serve the interaction on.
    #[cfg_attr(test, schema(nullable = false))]
    pub service_tier: Option<ServiceTier>,

    /// Where to deliver notifications when the interaction completes.
    #[cfg_attr(test, schema(nullable = false))]
    pub webhook_config: Option<WebhookConfig>,

    /// Opaque token to resume a long decode. Pass the latest token back unchanged to continue
    /// decoding.
    #[cfg_attr(test, schema(value_type = String, format = Byte, nullable = false))]
    pub continuation_token: Option<String>,

    /// The ID of the previous interaction, if any.
    #[cfg_attr(test, schema(nullable = false))]
    pub previous_interaction_id: Option<String>,

    /// Whether to run the model interaction in the background.
    #[cfg_attr(test, schema(nullable = false))]
    pub background: Option<bool>,

    /// Whether to store the response and request for later retrieval.
    #[cfg_attr(test, schema(nullable = false))]
    pub store: Option<bool>,

    /// Whether the interaction will be streamed.
    #[cfg_attr(test, schema(nullable = false))]
    pub stream: Option<bool>,
}

/// Parameters for creating an agent interaction.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema), schema(as = CreateAgentInteractionParams))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AgentInteraction {
    /// The name of the `Agent` used for generating the interaction.
    pub agent: AgentOption,

    /// The input for the interaction.
    pub input: InteractionsInput,

    /// Configuration parameters for the agent interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub agent_config: Option<AgentConfig>,

    /// System instruction for the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub system_instruction: Option<String>,

    /// A list of tool declarations the model may call during the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub tools: Option<Vec<Tool>>,

    /// The response format(s) enforced on the generated response.
    #[cfg_attr(test, schema(nullable = false))]
    pub response_format: Option<ResponseFormats>,

    /// The mime type of the response (deprecated; use `response_format`).
    #[deprecated(note = "use `response_format` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub response_mime_type: Option<String>,

    /// Safety settings for the interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub safety_settings: Option<Vec<SafetySetting>>,

    /// User-defined metadata for the request.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::map_schema, nullable = false))]
    pub labels: Option<BTreeMap<String, String>>,

    /// The environment the interaction runs in.
    #[cfg_attr(test, schema(nullable = false))]
    pub environment: Option<Environment>,

    /// The requested modalities of the response.
    #[deprecated(note = "use `response_format` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub response_modalities: Option<Vec<ResponseModality>>,

    /// The service tier to serve the interaction on.
    #[cfg_attr(test, schema(nullable = false))]
    pub service_tier: Option<ServiceTier>,

    /// Where to deliver notifications when the interaction completes.
    #[cfg_attr(test, schema(nullable = false))]
    pub webhook_config: Option<WebhookConfig>,

    /// Opaque token to resume a long decode. Pass the latest token back unchanged to continue
    /// decoding.
    #[cfg_attr(test, schema(value_type = String, format = Byte, nullable = false))]
    pub continuation_token: Option<String>,

    /// The ID of the previous interaction, if any.
    #[cfg_attr(test, schema(nullable = false))]
    pub previous_interaction_id: Option<String>,

    /// Whether to run the agent interaction in the background.
    #[cfg_attr(test, schema(nullable = false))]
    pub background: Option<bool>,

    /// Whether to store the response and request for later retrieval.
    #[cfg_attr(test, schema(nullable = false))]
    pub store: Option<bool>,

    /// Whether the interaction will be streamed.
    #[cfg_attr(test, schema(nullable = false))]
    pub stream: Option<bool>,
}

impl CreateInteractionRequest {
    /// Sets whether the interaction should be streamed.
    pub fn with_stream(mut self, stream: bool) -> Self {
        match &mut self {
            CreateInteractionRequest::Model(p) => p.stream = Some(stream),
            CreateInteractionRequest::Agent(p) => p.stream = Some(stream),
        }
        self
    }

    /// Sets the continuation token used to resume a long decode.
    pub fn with_continuation_token(mut self, token: impl Into<String>) -> Self {
        let token = Some(token.into());
        match &mut self {
            CreateInteractionRequest::Model(p) => p.continuation_token = token,
            CreateInteractionRequest::Agent(p) => p.continuation_token = token,
        }
        self
    }

    /// Sets whether the interaction should run in the background.
    pub fn with_background(mut self, background: bool) -> Self {
        match &mut self {
            CreateInteractionRequest::Model(p) => p.background = Some(background),
            CreateInteractionRequest::Agent(p) => p.background = Some(background),
        }
        self
    }
}

impl ModelInteraction {
    /// Creates model interaction parameters from a model and an input.
    pub fn new(model: impl Into<ModelOption>, input: impl Into<InteractionsInput>) -> Self {
        #[allow(deprecated)]
        Self {
            model: model.into(),
            input: input.into(),
            generation_config: None,
            system_instruction: None,
            tools: None,
            response_format: None,
            response_mime_type: None,
            safety_settings: None,
            labels: None,
            environment: None,
            response_modalities: None,
            service_tier: None,
            webhook_config: None,
            continuation_token: None,
            previous_interaction_id: None,
            background: None,
            store: None,
            stream: None,
        }
    }

    /// Sets the generation configuration.
    pub fn with_generation_config(mut self, config: GenerationConfig) -> Self {
        self.generation_config = Some(config);
        self
    }

    /// Sets the system instruction.
    pub fn with_system_instruction(mut self, instruction: impl Into<String>) -> Self {
        self.system_instruction = Some(instruction.into());
        self
    }

    /// Sets the tools available to the model.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the response format(s).
    pub fn with_response_format(mut self, response_format: impl Into<ResponseFormats>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Sets the safety settings.
    pub fn with_safety_settings(mut self, safety_settings: Vec<SafetySetting>) -> Self {
        self.safety_settings = Some(safety_settings);
        self
    }

    /// Sets the previous interaction ID, continuing a stateful conversation.
    pub fn with_previous_interaction_id(mut self, id: impl Into<String>) -> Self {
        self.previous_interaction_id = Some(id.into());
        self
    }

    /// Sets whether to store the interaction for later retrieval.
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Sets whether to run the interaction in the background.
    pub fn with_background(mut self, background: bool) -> Self {
        self.background = Some(background);
        self
    }

    /// Sets the user-defined metadata labels for the request.
    pub fn with_labels(mut self, labels: BTreeMap<String, String>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Sets the environment the interaction runs in.
    pub fn with_environment(mut self, environment: impl Into<Environment>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    /// Sets the service tier to serve the interaction on.
    pub fn with_service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    /// Sets where to deliver notifications when the interaction completes.
    pub fn with_webhook_config(mut self, webhook_config: WebhookConfig) -> Self {
        self.webhook_config = Some(webhook_config);
        self
    }
}

impl AgentInteraction {
    /// Creates agent interaction parameters from an agent and an input.
    pub fn new(agent: impl Into<AgentOption>, input: impl Into<InteractionsInput>) -> Self {
        #[allow(deprecated)]
        Self {
            agent: agent.into(),
            input: input.into(),
            agent_config: None,
            system_instruction: None,
            tools: None,
            response_format: None,
            response_mime_type: None,
            safety_settings: None,
            labels: None,
            environment: None,
            response_modalities: None,
            service_tier: None,
            webhook_config: None,
            continuation_token: None,
            previous_interaction_id: None,
            background: None,
            store: None,
            stream: None,
        }
    }

    /// Sets the agent configuration.
    pub fn with_agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = Some(config);
        self
    }

    /// Sets the system instruction.
    pub fn with_system_instruction(mut self, instruction: impl Into<String>) -> Self {
        self.system_instruction = Some(instruction.into());
        self
    }

    /// Sets the tools available to the agent.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the response format(s).
    pub fn with_response_format(mut self, response_format: impl Into<ResponseFormats>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Sets the safety settings.
    pub fn with_safety_settings(mut self, safety_settings: Vec<SafetySetting>) -> Self {
        self.safety_settings = Some(safety_settings);
        self
    }

    /// Sets the previous interaction ID, continuing a stateful conversation.
    pub fn with_previous_interaction_id(mut self, id: impl Into<String>) -> Self {
        self.previous_interaction_id = Some(id.into());
        self
    }

    /// Sets whether to store the interaction for later retrieval.
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Sets whether to run the interaction in the background.
    pub fn with_background(mut self, background: bool) -> Self {
        self.background = Some(background);
        self
    }

    /// Sets the user-defined metadata labels for the request.
    pub fn with_labels(mut self, labels: BTreeMap<String, String>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Sets the environment the interaction runs in.
    pub fn with_environment(mut self, environment: impl Into<Environment>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    /// Sets the service tier to serve the interaction on.
    pub fn with_service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    /// Sets where to deliver notifications when the interaction completes.
    pub fn with_webhook_config(mut self, webhook_config: WebhookConfig) -> Self {
        self.webhook_config = Some(webhook_config);
        self
    }
}

impl From<ModelInteraction> for CreateInteractionRequest {
    fn from(value: ModelInteraction) -> Self {
        CreateInteractionRequest::Model(Box::new(value))
    }
}

impl From<AgentInteraction> for CreateInteractionRequest {
    fn from(value: AgentInteraction) -> Self {
        CreateInteractionRequest::Agent(Box::new(value))
    }
}
