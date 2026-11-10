use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Config for translation.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TranslationConfig {
    /// Required. The target language for translation.
    /// Supported values are BCP-47 language codes.
    pub target_language_code: String,

    /// Optional. If true, the model will generate audio when the target language
    /// is spoken.
    pub echo_target_language: Option<bool>,
}

impl TranslationConfig {
    /// Creates a new TranslationConfig with the given target language code.
    pub fn new(target_language_code: impl Into<String>) -> Self {
        Self {
            target_language_code: target_language_code.into(),
            echo_target_language: None,
        }
    }

    /// Sets whether the model will generate audio when the target language is spoken.
    pub fn with_echo_target_language(mut self, echo_target_language: bool) -> Self {
        self.echo_target_language = Some(echo_target_language);
        self
    }
}
