use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::{string_enum, tagged_enum};

/// The environment an interaction runs in.
///
/// Either a configuration for an environment to create or update, or the ID of one that
/// already exists.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Environment {
    /// A configuration for the environment.
    Config(EnvironmentConfig),
    /// The ID of an existing environment.
    Id(String),
}

tagged_enum! {
    /// Configuration for a custom environment.
    ///
    /// Discriminated by its `type` field.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum EnvironmentConfig {
        /// An environment hosted by the API.
        Remote(RemoteEnvironment),
    }
}

/// An environment hosted by the API.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct RemoteEnvironment {
    /// The ID of an environment to update instead of creating a new one.
    #[cfg_attr(test, schema(nullable = false))]
    pub environment_id: Option<String>,

    /// Environment variables to set in the environment.
    #[cfg_attr(test, schema(nullable = false))]
    pub env: Option<EnvironmentVariables>,

    /// Network configuration for the environment.
    #[cfg_attr(test, schema(nullable = false))]
    pub network: Option<EnvironmentNetwork>,

    /// The sources mounted into the environment.
    #[cfg_attr(test, schema(nullable = false))]
    pub sources: Option<Vec<Source>>,
}

/// Environment variables to set in an environment.
///
/// The API accepts either a mapping of variable names to their definitions or a string.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EnvironmentVariables {
    /// A mapping of variable names to their definitions.
    Map(BTreeMap<String, EnvVar>),
    /// A string.
    Raw(String),
}

/// An environment variable to set in the execution environment.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct EnvVar {
    /// The ID of a server-managed `Credential` resource to take the value from.
    #[cfg_attr(test, schema(nullable = false))]
    pub credential: Option<String>,

    /// The value of a plain environment variable.
    #[cfg_attr(test, schema(nullable = false))]
    pub value: Option<String>,
}

/// Outbound networking configuration for an environment.
///
/// Either an allowlist restricting which domains the environment may reach, or `"disabled"`,
/// which turns all network access off. Omitted entirely, all outbound traffic is allowed.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EnvironmentNetwork {
    /// The domains the environment may reach.
    Egress(EnvironmentNetworkEgressAllowlist),
    /// No network access.
    Disabled(NetworkDisabled),
}

/// The domains an environment may reach.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct EnvironmentNetworkEgressAllowlist {
    /// The allowed outbound domains, or `"disabled"` to block all network egress.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowlist: Option<EgressAllowlist>,
}

/// The value of an egress allowlist: a list of rules, or `"disabled"`.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EgressAllowlist {
    /// The domains the environment may reach. Only requests to listed domains are permitted.
    Rules(Vec<EgressRule>),
    /// No network access.
    Disabled(NetworkDisabled),
}

/// Turns all network access off.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NetworkDisabled {
    /// No network access.
    Disabled,
}

/// A single domain allowlist rule, with optional header injection.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct EgressRule {
    /// The domain to allow outbound requests to. Supports wildcards, for example
    /// `*.googleapis.com`; `*` allows every domain.
    pub domain: String,

    /// The ID of a server-managed `Credential` resource to use for this domain.
    #[cfg_attr(test, schema(nullable = false))]
    pub credential: Option<String>,

    /// Headers the egress proxy injects on outbound requests matching this domain.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::map_schema, nullable = false))]
    pub transform: Option<BTreeMap<String, String>>,
}

/// A source mounted into an environment.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct Source {
    /// The inline content, when the source type is `inline`.
    #[cfg_attr(test, schema(nullable = false))]
    pub content: Option<String>,

    /// The encoding of the inline content, for example `base64`.
    #[cfg_attr(test, schema(nullable = false))]
    pub encoding: Option<String>,

    /// The source to mount: a GCS path, a GitHub path, and so on.
    #[cfg_attr(test, schema(nullable = false))]
    pub source: Option<String>,

    /// Where the source should appear in the environment.
    #[cfg_attr(test, schema(nullable = false))]
    pub target: Option<String>,

    /// The type of the source.
    #[serde(rename = "type")]
    #[cfg_attr(test, schema(nullable = false))]
    pub source_type: Option<SourceType>,
}

string_enum! {
    /// The type of a source mounted into an environment.
    #[serde(rename_all = "snake_case")]
    pub enum SourceType {
        /// A Cloud Storage path.
        Gcs,

        /// Content given inline.
        Inline,

        /// A source repository.
        Repository,
    }
}

impl RemoteEnvironment {
    /// Creates a new, empty remote environment configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the ID of the environment to update.
    pub fn with_environment_id(mut self, environment_id: impl Into<String>) -> Self {
        self.environment_id = Some(environment_id.into());
        self
    }

    /// Sets the environment variables.
    pub fn with_env(mut self, env: impl Into<EnvironmentVariables>) -> Self {
        self.env = Some(env.into());
        self
    }

    /// Sets the network configuration.
    pub fn with_network(mut self, network: EnvironmentNetwork) -> Self {
        self.network = Some(network);
        self
    }

    /// Sets the sources mounted into the environment.
    pub fn with_sources(mut self, sources: Vec<Source>) -> Self {
        self.sources = Some(sources);
        self
    }
}

impl EnvVar {
    /// Creates an environment variable with a plain value.
    pub fn value(value: impl Into<String>) -> Self {
        Self {
            credential: None,
            value: Some(value.into()),
        }
    }

    /// Creates an environment variable taking its value from a server-managed credential.
    pub fn credential(credential: impl Into<String>) -> Self {
        Self {
            credential: Some(credential.into()),
            value: None,
        }
    }
}

impl From<BTreeMap<String, EnvVar>> for EnvironmentVariables {
    fn from(value: BTreeMap<String, EnvVar>) -> Self {
        EnvironmentVariables::Map(value)
    }
}

impl From<String> for EnvironmentVariables {
    fn from(value: String) -> Self {
        EnvironmentVariables::Raw(value)
    }
}

impl EnvironmentNetworkEgressAllowlist {
    /// Creates an allowlist of the given entries.
    pub fn new(allowlist: Vec<EgressRule>) -> Self {
        Self {
            allowlist: Some(EgressAllowlist::Rules(allowlist)),
        }
    }
}

impl EgressRule {
    /// Creates an egress rule for the given domain.
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            credential: None,
            transform: None,
        }
    }

    /// Sets the ID of the server-managed `Credential` resource to use for this domain.
    pub fn with_credential(mut self, credential: impl Into<String>) -> Self {
        self.credential = Some(credential.into());
        self
    }

    /// Sets the headers injected on requests to this domain.
    pub fn with_transform(mut self, transform: BTreeMap<String, String>) -> Self {
        self.transform = Some(transform);
        self
    }
}

impl Source {
    /// Creates a new, empty source.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the source to mount.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Sets the inline content and its encoding.
    pub fn with_content(mut self, content: impl Into<String>, encoding: Option<String>) -> Self {
        self.content = Some(content.into());
        self.encoding = encoding;
        self
    }

    /// Sets where the source should appear in the environment.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Sets the type of the source.
    pub fn with_source_type(mut self, source_type: SourceType) -> Self {
        self.source_type = Some(source_type);
        self
    }
}

impl From<EnvironmentConfig> for Environment {
    fn from(value: EnvironmentConfig) -> Self {
        Environment::Config(value)
    }
}

impl From<String> for Environment {
    fn from(value: String) -> Self {
        Environment::Id(value)
    }
}

impl From<RemoteEnvironment> for EnvironmentConfig {
    fn from(value: RemoteEnvironment) -> Self {
        EnvironmentConfig::Remote(value)
    }
}

impl From<EnvironmentNetworkEgressAllowlist> for EnvironmentNetwork {
    fn from(value: EnvironmentNetworkEgressAllowlist) -> Self {
        EnvironmentNetwork::Egress(value)
    }
}
