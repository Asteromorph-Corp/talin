use std::fmt;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

use super::{ANTHROPIC_BETA, AUTHORIZATION};
use crate::claude_messages::types::{ApiErrorDetails, MessagesRequest};
use crate::error::Error;
use crate::vertex::VertexTokenProvider;

const DEFAULT_VERTEX_API_BASE: &str = "https://aiplatform.googleapis.com";
const DEFAULT_VERTEX_LOCATION: &str = "global";
const DEFAULT_VERTEX_ANTHROPIC_VERSION: &str = "vertex-2023-10-16";

/// Configuration for Google Cloud's Vertex AI Model Garden endpoint.
///
/// The Vertex endpoint speaks Anthropic's Messages wire format, but differs in three ways which
/// this config handles: the model is addressed in the URL rather than the request body, the request
/// body carries a required `anthropic_version` field, and authentication uses a bearer access token
/// resolved per request via a [token provider](Self::with_token_provider).
#[derive(Clone)]
pub struct VertexConfig {
    /// The api base.
    ///
    /// Defaults to "<https://aiplatform.googleapis.com>". Override this to target a regional
    /// endpoint (e.g. "<https://us-central1-aiplatform.googleapis.com>").
    api_base: String,

    /// The Google Cloud project id that hosts the endpoint.
    project_id: String,

    /// The Google Cloud location of the endpoint.
    ///
    /// Defaults to "global".
    location: String,

    /// Resolves the OAuth access token for `Authorization: Bearer` authentication, per request.
    token_provider: Option<Arc<dyn VertexTokenProvider>>,

    /// The beta version(s) to use.
    ///
    /// Sent via the `anthropic-beta` header, as on the first-party API.
    ///
    /// See [`anthropic-beta`](https://docs.claude.com/en/api/messages#parameter-anthropic-beta)
    anthropic_beta: Option<Vec<String>>,

    /// The `anthropic_version` body field value required by the Vertex endpoint.
    ///
    /// Defaults to "vertex-2023-10-16".
    anthropic_version: String,
}

impl VertexConfig {
    /// Creates a new [`VertexConfig`] for the given Google Cloud project.
    ///
    /// You must supply credentials with [`with_token_provider`](Self::with_token_provider).
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            api_base: DEFAULT_VERTEX_API_BASE.to_string(),
            project_id: project_id.into(),
            location: DEFAULT_VERTEX_LOCATION.to_string(),
            token_provider: None,
            anthropic_beta: None,
            anthropic_version: DEFAULT_VERTEX_ANTHROPIC_VERSION.to_string(),
        }
    }

    /// Sets the [`VertexTokenProvider`] used for Bearer authentication.
    ///
    /// The provider is called once per request and should return the current access token,
    /// refreshing it as needed.
    pub fn with_token_provider(mut self, provider: impl VertexTokenProvider + 'static) -> Self {
        self.token_provider = Some(Arc::new(provider));
        self
    }

    /// Sets the Anthropic beta flag(s) for the Vertex endpoint.
    pub fn with_anthropic_beta(mut self, anthropic_beta: impl IntoIterator<Item = String>) -> Self {
        self.anthropic_beta = Some(anthropic_beta.into_iter().collect());
        self
    }

    /// Checks whether the specific beta flag is enabled.
    pub fn is_beta_enabled(&self, flag: &str) -> bool {
        self.anthropic_beta
            .as_deref()
            .is_some_and(|beta| beta.iter().any(|b| b == flag))
    }

    /// Sets the Google Cloud location of the endpoint.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = location.into();
        self
    }

    /// Sets the API base. Override this to target a regional endpoint.
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    /// Sets the `anthropic_version` body field value.
    pub fn with_anthropic_version(mut self, anthropic_version: impl Into<String>) -> Self {
        self.anthropic_version = anthropic_version.into();
        self
    }

    /// Returns the `url` by concatenating the `api_base` and `path`.
    pub(super) fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base, path)
    }

    /// Returns the [`HeaderMap`], resolving the `Authorization: Bearer` token via the provider.
    pub(super) async fn headers(&self) -> Result<HeaderMap, Error<ApiErrorDetails>> {
        let mut headers = HeaderMap::new();

        if let Some(provider) = &self.token_provider {
            let token = provider
                .access_token()
                .await
                .map_err(|e| Error::Other(format!("failed to get access token: {e}")))?;
            let bearer = format!("Bearer {}", token.token);
            let Ok(mut value) = bearer.parse::<HeaderValue>() else {
                return Err(Error::invalid_header(AUTHORIZATION, "(secret)"));
            };
            value.set_sensitive(true);
            headers.insert(AUTHORIZATION, value);
        }

        if let Some(anthropic_beta) = &self.anthropic_beta {
            let anthropic_beta = anthropic_beta.join(",");
            let Ok(anthropic_beta) = anthropic_beta.parse() else {
                return Err(Error::InvalidHeader(ANTHROPIC_BETA, anthropic_beta));
            };
            headers.insert(ANTHROPIC_BETA, anthropic_beta);
        }

        Ok(headers)
    }

    pub(super) fn messages_path(&self, model: &str, stream: bool) -> String {
        let specifier = if stream {
            "streamRawPredict"
        } else {
            "rawPredict"
        };
        format!(
            "/v1/projects/{}/locations/{}/publishers/anthropic/models/{}:{}",
            self.project_id, self.location, model, specifier,
        )
    }

    /// Reshapes the request body for the Vertex endpoint: drops `model` (it is addressed in the
    /// URL) and injects the required `anthropic_version` field.
    pub(crate) fn messages_body(
        &self,
        request: &MessagesRequest,
    ) -> Result<Value, Error<ApiErrorDetails>> {
        let mut value = serde_json::to_value(request).map_err(|err| {
            Error::JSONDeserialize(err, "failed to serialize request body".into())
        })?;

        if let Some(object) = value.as_object_mut() {
            object.remove("model");
            object.insert(
                "anthropic_version".to_string(),
                Value::String(self.anthropic_version.clone()),
            );
        }

        Ok(value)
    }
}

impl fmt::Debug for VertexConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VertexConfig")
            .field("api_base", &self.api_base)
            .field("project_id", &self.project_id)
            .field("location", &self.location)
            .field(
                "token_provider",
                &self.token_provider.as_ref().map(|_| "<provider>"),
            )
            .field("anthropic_beta", &self.anthropic_beta)
            .field("anthropic_version", &self.anthropic_version)
            .finish()
    }
}
