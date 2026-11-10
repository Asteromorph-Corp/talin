use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{Content, GenerationConfig, Part, Role, SafetySetting, Tool, ToolConfig};

/// Request to generate content from the model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GenerateContentRequest {
    /// Required. The content of the current conversation with the model.
    pub contents: Vec<Content>,

    /// Optional. A list of Tools the Model may use to generate the next response.
    pub tools: Option<Vec<Tool>>,

    /// Optional. Tool configuration for any Tool specified in the request.
    pub tool_config: Option<ToolConfig>,

    /// Optional. A list of unique SafetySetting instances for blocking unsafe content.
    pub safety_settings: Option<Vec<SafetySetting>>,

    /// Optional. Developer set system instruction(s). Currently, text only.
    pub system_instruction: Option<Content>,

    /// Optional. Configuration options for model generation and outputs.
    pub generation_config: Option<GenerationConfig>,

    /// Optional. The name of the content cached to use as context.
    pub cached_content: Option<String>,

    /// Optional. The service tier to use for this request.
    pub service_tier: Option<ServiceTier>,

    /// Optional. If set, overrides the project-level setting for whether to log
    /// the request and response.
    pub store: Option<bool>,
}

/// Service tier used to fulfill a [`GenerateContentRequest`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ServiceTier {
    /// Default service tier, which is standard.
    #[serde(rename = "unspecified")]
    ServiceTierUnspecified,
    /// Standard service tier.
    Standard,
    /// Priority service tier (lower latency, higher quota cost).
    Priority,
    /// Flex service tier (best-effort latency, lower quota cost).
    Flex,
    /// Unknown service tier not represented by this enum.
    #[serde(other)]
    Unknown,
}

impl GenerateContentRequest {
    /// Creates a new request with the given contents.
    pub fn new(contents: Vec<Content>) -> Self {
        Self {
            contents,
            tools: None,
            tool_config: None,
            safety_settings: None,
            system_instruction: None,
            generation_config: None,
            cached_content: None,
            service_tier: None,
            store: None,
        }
    }

    /// Creates a new request from a single user text message.
    pub fn from_text(text: impl Into<String>) -> Self {
        Self::new(vec![Content::user_text(text)])
    }

    /// Sets the tools for this request.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the tool configuration for this request.
    pub fn with_tool_config(mut self, tool_config: ToolConfig) -> Self {
        self.tool_config = Some(tool_config);
        self
    }

    /// Sets the safety settings for this request.
    pub fn with_safety_settings(mut self, safety_settings: Vec<SafetySetting>) -> Self {
        self.safety_settings = Some(safety_settings);
        self
    }

    /// Sets the system instruction for this request.
    pub fn with_system_instruction(mut self, instruction: Content) -> Self {
        self.system_instruction = Some(instruction);
        self
    }

    /// Sets the system instruction from text.
    pub fn with_system_instruction_text(mut self, text: impl Into<String>) -> Self {
        self.system_instruction = Some(Content::system_text(text));
        self
    }

    /// Sets the generation configuration for this request.
    pub fn with_generation_config(mut self, config: GenerationConfig) -> Self {
        self.generation_config = Some(config);
        self
    }

    /// Sets the cached content name for this request.
    pub fn with_cached_content(mut self, cached_content: impl Into<String>) -> Self {
        self.cached_content = Some(cached_content.into());
        self
    }

    /// Sets the service tier for this request.
    pub fn with_service_tier(mut self, tier: ServiceTier) -> Self {
        self.service_tier = Some(tier);
        self
    }

    /// Overrides the project-level logging setting for this request.
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Adds a content to this request.
    pub fn add_content(mut self, content: Content) -> Self {
        self.contents.push(content);
        self
    }

    /// Adds a user message to this request.
    pub fn add_user_text(mut self, text: impl Into<String>) -> Self {
        self.contents.push(Content::user_text(text));
        self
    }

    /// Adds a model message to this request.
    pub fn add_model_text(mut self, text: impl Into<String>) -> Self {
        self.contents.push(Content::model_text(text));
        self
    }
}

impl Content {
    /// Creates a new content with the given role
    pub fn new(role: Role) -> Self {
        Self {
            role: Some(role),
            parts: Some(Vec::new()),
        }
    }

    /// Creates a new content with the given role and parts.
    pub fn new_with_parts(role: Role, parts: Vec<Part>) -> Self {
        Self {
            role: Some(role),
            parts: Some(parts),
        }
    }

    /// Creates a user content
    pub fn user() -> Self {
        Self::new(Role::User)
    }

    /// Creates a user content with the given parts.
    pub fn user_with_parts(parts: Vec<Part>) -> Self {
        Self::new_with_parts(Role::User, parts)
    }

    /// Creates a model content
    pub fn model() -> Self {
        Self::new(Role::Model)
    }

    /// Creates a model content with the given parts.
    pub fn model_with_parts(parts: Vec<Part>) -> Self {
        Self::new_with_parts(Role::Model, parts)
    }

    /// Creates a user content with a single text part.
    pub fn user_text(text: impl Into<String>) -> Self {
        Self::user_with_parts(vec![Part::text(text)])
    }

    /// Creates a model content with a single text part.
    pub fn model_text(text: impl Into<String>) -> Self {
        Self::model_with_parts(vec![Part::text(text)])
    }

    /// Creates a system instruction content (used for system_instruction field).
    pub fn system_text(text: impl Into<String>) -> Self {
        Self {
            role: None,
            parts: Some(vec![Part::text(text)]),
        }
    }

    /// Creates an empty system content (for building with parts).
    pub fn system() -> Self {
        Self {
            role: None,
            parts: Some(Vec::new()),
        }
    }

    /// Adds a part to this content.
    pub fn with_part(mut self, part: Part) -> Self {
        self.parts.get_or_insert_with(Vec::new).push(part);
        self
    }

    /// Adds a text part to this content.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.parts
            .get_or_insert_with(Vec::new)
            .push(Part::text(text));
        self
    }
}

impl Part {
    /// Creates a text part.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            thought: None,
            thought_signature: None,
            part_metadata: None,
            media_resolution: None,
            data: super::Data::Text(text.into()),
            metadata: None,
        }
    }

    /// Creates an inline data part from bytes.
    pub fn inline_data(mime_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            thought: None,
            thought_signature: None,
            part_metadata: None,
            media_resolution: None,
            data: super::Data::InlineData(super::Blob {
                mime_type: mime_type.into(),
                data,
            }),
            metadata: None,
        }
    }

    /// Creates a file data part from a URI.
    pub fn file_data(mime_type: impl Into<String>, file_uri: impl Into<String>) -> Self {
        Self {
            thought: None,
            thought_signature: None,
            part_metadata: None,
            media_resolution: None,
            data: super::Data::FileData(super::FileData {
                mime_type: mime_type.into(),
                file_uri: file_uri.into(),
            }),
            metadata: None,
        }
    }

    /// Creates a function call part.
    pub fn function_call(
        id: Option<impl Into<String>>,
        name: impl Into<String>,
        args: serde_json::Value,
    ) -> Self {
        Self {
            thought: None,
            thought_signature: None,
            part_metadata: None,
            media_resolution: None,
            data: super::Data::FunctionCall(super::FunctionCall {
                id: id.map(|s| s.into()),
                name: name.into(),
                args: Some(args),
            }),
            metadata: None,
        }
    }

    /// Creates a function response part.
    pub fn function_response(
        id: Option<impl Into<String>>,
        name: impl Into<String>,
        response: serde_json::Value,
    ) -> Self {
        Self {
            thought: None,
            thought_signature: None,
            part_metadata: None,
            media_resolution: None,
            data: super::Data::FunctionResponse(super::FunctionResponse {
                id: id.map(|s| s.into()),
                name: name.into(),
                response,
                parts: None,
                will_continue: Some(false),
                scheduling: None,
            }),
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_from_text() {
        let request = GenerateContentRequest::from_text("Hello");
        assert_eq!(request.contents.len(), 1);
        assert_eq!(request.contents[0].role, Some(Role::User));
    }

    #[test]
    fn test_content_user_text() {
        let content = Content::user_text("Hello");
        assert_eq!(content.role, Some(Role::User));
        assert_eq!(content.parts.map(|parts| parts.len()), Some(1));
    }

    #[test]
    fn test_part_text() {
        let part = Part::text("Hello");
        assert!(matches!(part.data, super::super::Data::Text(ref s) if s == "Hello"));
    }

    #[test]
    fn test_service_tier_wire_format() {
        // The API expects lowercase, prefix-free values.
        for (tier, wire) in [
            (ServiceTier::ServiceTierUnspecified, "\"unspecified\""),
            (ServiceTier::Standard, "\"standard\""),
            (ServiceTier::Flex, "\"flex\""),
            (ServiceTier::Priority, "\"priority\""),
        ] {
            assert_eq!(serde_json::to_string(&tier).unwrap(), wire);
            assert_eq!(serde_json::from_str::<ServiceTier>(wire).unwrap(), tier);
        }
    }

    #[test]
    fn test_request_builder_pattern() {
        let request = GenerateContentRequest::from_text("Hello")
            .with_system_instruction_text("You are a helpful assistant")
            .with_generation_config(GenerationConfig::default());

        assert_eq!(request.contents.len(), 1);
        assert!(request.system_instruction.is_some());
        assert!(request.generation_config.is_some());
    }
}
