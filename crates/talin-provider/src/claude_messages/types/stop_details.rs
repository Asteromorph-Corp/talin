use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::{string_enum, tagged_enum};

tagged_enum! {
    /// Structured information about why the model stopped.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum RefusalStopDetails {
        /// Details for a refusal stop.
        Refusal(RefusalStopDetailsInner),
    }
}

/// Structured information about a refusal stop.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct RefusalStopDetailsInner {
    /// The policy category that triggered the refusal.
    #[cfg_attr(test, schema(required = true))]
    pub category: Option<RefusalCategory>,

    /// Human-readable explanation of the refusal.
    #[cfg_attr(test, schema(required = true))]
    pub explanation: Option<String>,
}

string_enum! {
    /// The policy category that triggered a refusal.
    #[serde(rename_all = "snake_case")]
    pub enum RefusalCategory {
        /// `"cyber"`
        Cyber,

        /// `"bio"`
        Bio,

        /// `"frontier_llm"`
        FrontierLlm,

        /// `"reasoning_extraction"`
        ReasoningExtraction,

        /// `"general_harms"`
        GeneralHarms,
    }
}

impl RefusalStopDetailsInner {
    /// Creates a new [`RefusalStopDetailsInner`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`category`](Self::category) that triggered the refusal.
    pub fn with_category(mut self, category: RefusalCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Sets the [`explanation`](Self::explanation) of the refusal.
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }
}
