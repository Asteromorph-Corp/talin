use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::gemini_interactions::types::{ImageAspectRatio, ImageSize};
use crate::utils::string_enum;

/// Configuration parameters for model interactions.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
// `image_config` is deprecated, and mentioning it in this struct's own code is not a use the
// deprecation is warning about.
#[allow(deprecated)]
pub struct GenerationConfig {
    /// Configuration for image generation.
    #[deprecated(note = "use `response_format` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub image_config: Option<ImageConfig>,

    /// The maximum number of tokens to include in the response.
    #[cfg_attr(test, schema(nullable = false))]
    pub max_output_tokens: Option<i32>,

    /// Seed used in decoding for reproducibility.
    #[cfg_attr(test, schema(nullable = false))]
    pub seed: Option<i32>,

    /// Configuration for speech generation, one entry per speaker.
    #[cfg_attr(test, schema(nullable = false, inline))]
    pub structured_speech_config: Option<SpeechConfigType>,

    /// A list of character sequences that will stop output interaction.
    #[cfg_attr(test, schema(nullable = false))]
    pub stop_sequences: Option<Vec<String>>,

    /// Controls the randomness of the output.
    #[cfg_attr(test, schema(nullable = false))]
    #[deprecated]
    pub temperature: Option<f32>,

    /// The level of thought tokens that the model should generate.
    #[cfg_attr(test, schema(nullable = false))]
    pub thinking_level: Option<ThinkingLevel>,

    /// Whether to include thought summaries in the response.
    #[cfg_attr(test, schema(nullable = false))]
    pub thinking_summaries: Option<ThinkingSummaries>,

    /// The tool choice configuration.
    #[cfg_attr(test, schema(nullable = false))]
    pub tool_choice: Option<ToolChoice>,

    /// The maximum cumulative probability of tokens to consider when sampling.
    #[cfg_attr(test, schema(nullable = false))]
    #[deprecated]
    pub top_p: Option<f32>,

    /// Configuration for speech recognition. Transcription is enabled when it is set.
    #[cfg_attr(test, schema(nullable = false))]
    pub transcription_config: Option<TranscriptionConfig>,

    /// Configuration for video generation.
    #[cfg_attr(test, schema(nullable = false))]
    pub video_config: Option<VideoConfig>,
}

/// Configuration for image generation.
///
/// Deprecated in favour of an image [`ResponseFormat`](super::ResponseFormat).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema), schema(deprecated))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ImageConfig {
    /// The aspect ratio of the generated image.
    #[cfg_attr(test, schema(nullable = false, value_type = Option<String>))]
    pub aspect_ratio: Option<ImageAspectRatio>,

    /// The size of the generated image.
    #[cfg_attr(test, schema(nullable = false, value_type = Option<String>))]
    pub image_size: Option<ImageSize>,
}

/// Speaker or speech configuration.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SpeechConfigType {
    /// Speaker configuration.
    Speaker(SpeakerConfig),
    /// Speech configuration.
    Speech(Vec<SpeechConfig>),
}

/// Configuration for multi-speaker and speech generation.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct SpeakerConfig {
    /// Individual speaker configurations.
    #[cfg_attr(test, schema(nullable = false))]
    pub speakers: Option<Vec<SpeechConfig>>,
}

/// The configuration for speech interaction.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct SpeechConfig {
    /// The language of the speech.
    #[cfg_attr(test, schema(nullable = false))]
    pub language: Option<String>,

    /// The speaker's name, matching the speaker named in the prompt.
    #[cfg_attr(test, schema(nullable = false))]
    pub speaker: Option<String>,

    /// The voice of the speaker.
    #[cfg_attr(test, schema(nullable = false))]
    pub voice: Option<String>,
}

/// Configuration for speech recognition (transcription).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct TranscriptionConfig {
    /// Phrases to bias the recognition model towards.
    #[deprecated(note = "use `custom_vocabulary` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub adaptation_phrases: Option<Vec<String>>,

    /// Phrases to bias the recognition model towards recognizing specific terms.
    #[cfg_attr(test, schema(nullable = false))]
    pub custom_vocabulary: Option<Vec<String>>,

    /// The speaker diarization mode. `speaker` is supported.
    #[cfg_attr(test, schema(nullable = false))]
    pub diarization_mode: Option<String>,

    /// BCP-47 language codes hinting at the languages present in the audio. Left empty, the
    /// language is detected automatically.
    #[cfg_attr(test, schema(nullable = false))]
    pub language_codes: Option<Vec<String>>,

    /// Discriminated transcription mode options or enum.
    #[cfg_attr(test, schema(nullable = false))]
    pub mode: Option<TranscriptionConfigMode>,

    /// The granularity of the timestamps in the transcription. `word` is supported; left
    /// empty, no timestamps are generated.
    #[cfg_attr(test, schema(nullable = false))]
    pub timestamp_granularities: Option<Vec<String>>,
}

/// Configuration for video generation.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct VideoConfig {
    /// The task mode for the generation. Left unset, the model determines the mode from the
    /// prompt and the input media.
    #[cfg_attr(test, schema(nullable = false))]
    pub task: Option<VideoTask>,
}

string_enum! {
    /// The task mode for video generation.
    #[serde(rename_all = "snake_case")]
    pub enum VideoTask {
        /// Generation from text.
        TextToVideo,

        /// Generation from an image.
        ImageToVideo,

        /// Generation from a reference.
        ReferenceToVideo,

        /// Editing of an existing video.
        Edit,

        /// Extends an existing input video.
        Extend,
    }
}

string_enum! {
    /// The level of thought tokens that the model should generate.
    #[serde(rename_all = "snake_case")]
    pub enum ThinkingLevel {
        /// Little to no thinking.
        Minimal,

        /// Low thinking level.
        Low,

        /// Medium thinking level.
        Medium,

        /// High thinking level.
        High,
    }
}

string_enum! {
    /// Whether to include thought summaries in the response.
    #[serde(rename_all = "snake_case")]
    pub enum ThinkingSummaries {
        /// Auto thinking summaries.
        Auto,

        /// No thinking summaries.
        None,
    }
}

/// The tool choice configuration.
///
/// Either a simple [`ToolMode`] or a detailed [`ToolChoiceConfig`].
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ToolChoice {
    #[cfg_attr(test, allow(missing_docs))] // Documentation messes with schema generation.
    #[cfg_attr(not(test), doc = "A simple tool-choice mode.")]
    Mode(ToolMode),
    /// A detailed tool-choice configuration listing the allowed tools.
    Config(ToolChoiceConfig),
}

string_enum! {
    /// The mode governing how the model may use tools.
    #[serde(rename_all = "snake_case")]
    pub enum ToolMode {
        /// Auto tool choice.
        Auto,

        /// Any tool choice.
        Any,

        /// No tool choice.
        None,

        /// Validated tool choice.
        Validated,
    }
}

/// The tool choice configuration containing allowed tools.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ToolChoiceConfig {
    /// The allowed tools.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_tools: Option<AllowedTools>,
}

/// The configuration for allowed tools.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct AllowedTools {
    /// The mode of the tool choice.
    #[cfg_attr(test, schema(nullable = false))]
    pub mode: Option<ToolMode>,

    /// The names of the allowed tools.
    #[cfg_attr(test, schema(nullable = false))]
    pub tools: Option<Vec<String>>,
}

/// Discriminated transcription mode options or enum.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum TranscriptionConfigMode {
    /// The transcription mode with additional options.
    Options(TranscriptionMode),
    #[cfg_attr(test, allow(missing_docs))] // Documentation messes with schema generation.
    #[cfg_attr(not(test), doc = "Plain transcription mode.")]
    Type(#[cfg_attr(test, schema(inline = true))] TranscriptionModeType),
}

/// The transcription mode with additional options.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum TranscriptionMode {
    /// Smart transcription mode with options.
    Smart(SmartTranscriptionMode),
    /// Verbatim transcription mode with options.
    Verbatim(VerbatimTranscriptionMode),
}

/// Additional options for smart transcription mode.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct SmartTranscriptionMode {
    /// The source text to be edited or rewritten via voice instructions.
    ///
    /// When provided, the model performs voice-driven editing and outputs the finalized
    /// replacement text.
    #[cfg_attr(test, schema(nullable = false))]
    pub source_text: Option<String>,
}

/// Additional options for verbatim transcription mode.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct VerbatimTranscriptionMode {
    /// Configures speaker diarization. Supported values: \"speaker\".
    #[cfg_attr(test, schema(nullable = false))]
    pub diarization_mode: Option<String>,
    /// The granularity of timestamps to include in the transcription output. Supported values:
    /// `"word\"` If empty, no timestamps are generated.
    #[cfg_attr(test, schema(nullable = false))]
    pub timestamp_granularities: Option<Vec<String>>,
}

string_enum! {
    /// Plain transcription mode.
    #[serde(rename_all = "snake_case")]
    pub enum TranscriptionModeType {
        /// Smart transcription mode.
        Smart,

        /// Verbatim transcription mode.
        Verbatim,
    }
}

impl GenerationConfig {
    /// Creates a new, empty generation configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of output tokens.
    pub fn with_max_output_tokens(mut self, max_output_tokens: i32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    /// Sets the decoding seed.
    pub fn with_seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets the stop sequences.
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Sets the temperature.
    #[deprecated]
    #[allow(deprecated)]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the thinking level.
    pub fn with_thinking_level(mut self, thinking_level: ThinkingLevel) -> Self {
        self.thinking_level = Some(thinking_level);
        self
    }

    /// Sets the tool choice configuration.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the top-p value.
    #[deprecated]
    #[allow(deprecated)]
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Sets whether to include thought summaries in the response.
    pub fn with_thinking_summaries(mut self, thinking_summaries: ThinkingSummaries) -> Self {
        self.thinking_summaries = Some(thinking_summaries);
        self
    }

    /// Sets the speech generation configuration, one entry per speaker.
    pub fn with_structured_speech_config(mut self, speech_config: SpeechConfigType) -> Self {
        self.structured_speech_config = Some(speech_config);
        self
    }

    /// Sets the speech recognition configuration, enabling transcription.
    pub fn with_transcription_config(mut self, transcription_config: TranscriptionConfig) -> Self {
        self.transcription_config = Some(transcription_config);
        self
    }

    /// Sets the video generation configuration.
    pub fn with_video_config(mut self, video_config: VideoConfig) -> Self {
        self.video_config = Some(video_config);
        self
    }
}

impl ImageConfig {
    /// Creates a new, empty image generation configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the aspect ratio of the generated image.
    pub fn with_aspect_ratio(mut self, aspect_ratio: ImageAspectRatio) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    /// Sets the size of the generated image.
    pub fn with_image_size(mut self, image_size: ImageSize) -> Self {
        self.image_size = Some(image_size);
        self
    }
}

impl SpeakerConfig {
    /// Creates a new, empty speaker configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the speech configurations for the speakers.
    pub fn with_speakers(mut self, speakers: Vec<SpeechConfig>) -> Self {
        self.speakers = Some(speakers);
        self
    }
}

impl SpeechConfig {
    /// Creates a new, empty speech configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the language of the speech.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Sets the speaker the configuration applies to.
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// Sets the voice of the speaker.
    pub fn with_voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = Some(voice.into());
        self
    }
}

impl TranscriptionConfig {
    /// Creates a new, empty transcription configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the phrases to bias recognition towards.
    pub fn with_custom_vocabulary(mut self, custom_vocabulary: Vec<String>) -> Self {
        self.custom_vocabulary = Some(custom_vocabulary);
        self
    }

    /// Sets the speaker diarization mode.
    pub fn with_diarization_mode(mut self, diarization_mode: impl Into<String>) -> Self {
        self.diarization_mode = Some(diarization_mode.into());
        self
    }

    /// Sets the language hints for the audio.
    pub fn with_language_codes(mut self, language_codes: Vec<String>) -> Self {
        self.language_codes = Some(language_codes);
        self
    }

    /// Sets the transcription mode.
    pub fn with_mode(mut self, mode: TranscriptionConfigMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Sets the granularity of the timestamps in the transcription.
    pub fn with_timestamp_granularities(mut self, granularities: Vec<String>) -> Self {
        self.timestamp_granularities = Some(granularities);
        self
    }
}

impl VideoConfig {
    /// Creates a video configuration for the given task mode.
    pub fn new(task: VideoTask) -> Self {
        Self { task: Some(task) }
    }
}

impl ToolChoice {
    /// Creates a tool choice that restricts the model to the given tool names in the given
    /// mode.
    pub fn allowed(mode: ToolMode, tools: Vec<String>) -> Self {
        ToolChoice::Config(ToolChoiceConfig {
            allowed_tools: Some(AllowedTools::new(mode).with_tools(tools)),
        })
    }
}

impl ToolChoiceConfig {
    /// Creates a tool choice configuration from the given allowed tools.
    pub fn new(allowed_tools: AllowedTools) -> Self {
        Self {
            allowed_tools: Some(allowed_tools),
        }
    }
}

impl AllowedTools {
    /// Creates an allowed-tools configuration with the given mode.
    pub fn new(mode: ToolMode) -> Self {
        Self {
            mode: Some(mode),
            tools: None,
        }
    }

    /// Sets the names of the allowed tools.
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = Some(tools);
        self
    }
}

impl From<ToolMode> for ToolChoice {
    fn from(value: ToolMode) -> Self {
        ToolChoice::Mode(value)
    }
}

impl From<ToolChoiceConfig> for ToolChoice {
    fn from(value: ToolChoiceConfig) -> Self {
        ToolChoice::Config(value)
    }
}

impl SmartTranscriptionMode {
    /// Creates a new smart transcription mode configuration.
    pub fn new() -> Self {
        Self { source_text: None }
    }

    /// Sets the source text for the smart transcription mode.
    pub fn with_source_text(mut self, source_text: impl Into<String>) -> Self {
        self.source_text = Some(source_text.into());
        self
    }
}

impl VerbatimTranscriptionMode {
    /// Creates a new verbatim transcription mode configuration.
    pub fn new() -> Self {
        Self {
            diarization_mode: None,
            timestamp_granularities: None,
        }
    }

    /// Sets the speaker diarization mode.
    pub fn with_diarization_mode(mut self, diarization_mode: impl Into<String>) -> Self {
        self.diarization_mode = Some(diarization_mode.into());
        self
    }

    /// Sets the granularity of timestamps in the transcription.
    pub fn with_timestamp_granularities(mut self, granularities: Vec<String>) -> Self {
        self.timestamp_granularities = Some(granularities);
        self
    }
}
