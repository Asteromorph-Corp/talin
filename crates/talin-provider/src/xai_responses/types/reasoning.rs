use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Reasoning configuration.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ReasoningConfiguration {
    /// Constrains how hard a reasoning model thinks before responding.
    #[cfg_attr(test, schema(value_type = Option<String>))]
    pub effort: Option<ReasoningEffort>,

    /// Only included for compatibility.
    pub generate_summary: Option<String>,

    /// A summary of the model's reasoning process.
    /// Only included for compatibility. The model shall always return `"detailed"`.
    #[cfg_attr(test, schema(value_type = Option<String>))]
    pub summary: Option<ReasoningSummary>,
}

/// Reasoning effort levels.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReasoningEffort {
    /// Disables reasoning completely.
    None,
    /// Uses fewer reasoning tokens.
    Low,
    /// Uses a moderate amount of reasoning tokens.
    Medium,
    /// Uses more reasoning tokens.
    High,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

/// Reasoning summary options.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReasoningSummary {
    /// Auto.
    Auto,
    /// Concise.
    Concise,
    /// Detailed.
    Detailed,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

/// The reasoning done by the model.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Reasoning {
    /// The unique ID of the reasoning content.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// The encrypted reasoning.
    /// Returned when `"reasoning.encrypted_content"` is passed in
    /// [`include`](crate::xai_responses::types::ModelRequest::include).
    pub encrypted_content: Option<String>,

    /// Status of the item.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ReasoningStatus>,

    /// The reasoning text contents.
    #[cfg_attr(test, schema(nullable = false))]
    pub content: Option<Vec<ReasoningText>>,

    /// The summarized reasoning text contents.
    pub summary: Vec<SummaryText>,
}

/// Status of the reasoning item.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReasoningStatus {
    /// Completed.
    Completed,
    /// In progress.
    InProgress,
    /// Incomplete.
    Incomplete,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

/// The reasoning text contents.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReasoningText {
    /// The reasoning text done by the model.
    ReasoningText(ReasoningTextInner),

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

// The closed form of `ReasoningText`, which is what its description is generated from.
/// The reasoning text contents.
#[cfg(test)]
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
#[schema(as = ReasoningText)]
#[serde(tag = "type", rename_all = "snake_case")]
enum KnownReasoningText {
    // Documentation messes with schema generation.
    #[schema(title = "ReasoningTextInner")]
    ReasoningText(#[schema(inline = true)] ReasoningTextInner),
}

#[cfg(test)]
crate::utils::delegate_schema!(ReasoningText => KnownReasoningText);

/// The reasoning text done by the model.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ReasoningTextInner {
    /// Reasoning done by the model.
    pub text: String,
}

/// The summarized reasoning text contents.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SummaryText {
    /// The summarized reasoning text done by the model.
    SummaryText(SummaryTextInner),

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

// The closed form of `SummaryText`, which is what its description is generated from.
/// The summarized reasoning text contents.
#[cfg(test)]
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
#[schema(as = SummaryText)]
#[serde(tag = "type", rename_all = "snake_case")]
enum KnownSummaryText {
    // Documentation messes with schema generation.
    #[schema(title = "SummaryTextInner")]
    SummaryText(#[schema(inline = true)] SummaryTextInner),
}

#[cfg(test)]
crate::utils::delegate_schema!(SummaryText => KnownSummaryText);

/// The summarized reasoning text done by the model.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SummaryTextInner {
    /// Summary of the reasoning done by the model.
    pub text: String,
}

impl ReasoningConfiguration {
    /// Creates an empty [`ReasoningConfiguration`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets how hard a reasoning model thinks before responding.
    pub fn with_effort(mut self, effort: ReasoningEffort) -> Self {
        self.effort = Some(effort);
        self
    }

    /// Sets the `generate_summary` field (only included for compatibility).
    pub fn with_generate_summary(mut self, generate_summary: impl Into<String>) -> Self {
        self.generate_summary = Some(generate_summary.into());
        self
    }

    /// Sets the reasoning summary option.
    pub fn with_summary(mut self, summary: ReasoningSummary) -> Self {
        self.summary = Some(summary);
        self
    }
}

impl Reasoning {
    /// Creates a [`Reasoning`] with the given summary.
    pub fn new(summary: Vec<SummaryText>) -> Self {
        Self {
            id: None,
            encrypted_content: None,
            status: None,
            content: None,
            summary,
        }
    }

    /// Sets the unique ID of the reasoning content.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the encrypted reasoning, as returned when `"reasoning.encrypted_content"` is passed
    /// in [`include`](crate::xai_responses::types::ModelRequest::include).
    pub fn with_encrypted_content(mut self, encrypted_content: impl Into<String>) -> Self {
        self.encrypted_content = Some(encrypted_content.into());
        self
    }

    /// Sets the status of the item.
    pub fn with_status(mut self, status: ReasoningStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the reasoning text contents.
    pub fn with_content(mut self, content: Vec<ReasoningText>) -> Self {
        self.content = Some(content);
        self
    }
}

impl ReasoningText {
    /// Creates a [`ReasoningText`] carrying the given reasoning text.
    pub fn new(text: impl Into<String>) -> Self {
        Self::ReasoningText(ReasoningTextInner::new(text))
    }
}

impl ReasoningTextInner {
    /// Creates a [`ReasoningTextInner`] with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl SummaryText {
    /// Creates a [`SummaryText`] carrying the given summary text.
    pub fn new(text: impl Into<String>) -> Self {
        Self::SummaryText(SummaryTextInner::new(text))
    }
}

impl SummaryTextInner {
    /// Creates a [`SummaryTextInner`] with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}
