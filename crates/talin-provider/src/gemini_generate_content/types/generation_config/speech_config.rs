use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// The speech generation config.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SpeechConfig {
    /// The configuration in case of single-voice output.
    pub voice_config: Option<VoiceConfig>,

    /// Optional. The configuration for the multi-speaker setup.
    /// It is mutually exclusive with the voiceConfig field.
    pub multi_speaker_voice_config: Option<MultiSpeakerVoiceConfig>,

    /// Optional. Language code (in BCP 47 format, e.g. "en-US") for speech synthesis.
    pub language_code: Option<String>,
}

impl SpeechConfig {
    /// Creates a new SpeechConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the single-voice configuration.
    pub fn with_voice_config(mut self, voice_config: VoiceConfig) -> Self {
        self.voice_config = Some(voice_config);
        self
    }

    /// Sets the multi-speaker configuration.
    pub fn with_multi_speaker_voice_config(
        mut self,
        multi_speaker_voice_config: MultiSpeakerVoiceConfig,
    ) -> Self {
        self.multi_speaker_voice_config = Some(multi_speaker_voice_config);
        self
    }

    /// Sets the language code (BCP 47 format).
    pub fn with_language_code(mut self, language_code: impl Into<String>) -> Self {
        self.language_code = Some(language_code.into());
        self
    }
}

/// The configuration for the voice to use.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum VoiceConfig {
    /// Prebuilt voice configuration.
    PrebuiltVoiceConfig(PrebuiltVoiceConfig),
}

/// The configuration for the prebuilt speaker to use.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PrebuiltVoiceConfig {
    /// The name of the preset voice to use.
    pub voice_name: String,
}

impl PrebuiltVoiceConfig {
    /// Creates a new PrebuiltVoiceConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of the preset voice to use.
    pub fn with_voice_name(mut self, voice_name: impl Into<String>) -> Self {
        self.voice_name = voice_name.into();
        self
    }
}

/// The configuration for the multi-speaker setup.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MultiSpeakerVoiceConfig {
    /// Required. All the enabled speaker voices.
    pub speaker_voice_configs: Vec<SpeakerVoiceConfig>,
}

impl MultiSpeakerVoiceConfig {
    /// Creates a new MultiSpeakerVoiceConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the enabled speaker voices.
    pub fn with_speaker_voice_configs(
        mut self,
        speaker_voice_configs: Vec<SpeakerVoiceConfig>,
    ) -> Self {
        self.speaker_voice_configs = speaker_voice_configs;
        self
    }
}

/// The configuration for a single speaker in a multi speaker setup.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SpeakerVoiceConfig {
    /// Required. The name of the speaker to use. Should be the same as in the prompt.
    pub speaker: String,

    /// Required. The configuration for the voice to use.
    pub voice_config: VoiceConfig,
}

impl SpeakerVoiceConfig {
    /// Creates a new SpeakerVoiceConfig.
    pub fn new(speaker: impl Into<String>, voice_config: VoiceConfig) -> Self {
        Self {
            speaker: speaker.into(),
            voice_config,
        }
    }
}
