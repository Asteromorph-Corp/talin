use talin_provider::gemini_interactions::types::{ModalityTokens, ResponseModality, Usage};

use crate::llm::{LlmPricing, LlmUsage};
use crate::schemas::{UsageLine, UsageLines};

const TEXT_INPUT_TOKENS: &str = "Text input tokens";
const TEXT_INPUT_TOKENS_CACHED: &str = "Text input tokens (cached)";
const IMAGE_INPUT_TOKENS: &str = "Image input tokens";
const IMAGE_INPUT_TOKENS_CACHED: &str = "Image input tokens (cached)";
const VIDEO_INPUT_TOKENS: &str = "Video input tokens";
const VIDEO_INPUT_TOKENS_CACHED: &str = "Video input tokens (cached)";
const AUDIO_INPUT_TOKENS: &str = "Audio input tokens";
const AUDIO_INPUT_TOKENS_CACHED: &str = "Audio input tokens (cached)";
const TOOL_USE_INPUT_TOKENS: &str = "Tool use input tokens";
const OUTPUT_TOKENS: &str = "Output tokens";
const THINKING_TOKENS: &str = "Thinking tokens";

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
/// Pricing specification for the Gemini Interactions API.
pub struct InteractionsPricing {
    /// USD price per million text input tokens.
    pub text_input: f64,
    /// USD price per million cached text input tokens.
    ///
    /// If unspecified, defaults to `text_input`.
    pub text_input_cached: Option<f64>,
    /// USD price per million image input tokens.
    ///
    /// If unspecified, defaults to `text_input`.
    pub image_input: Option<f64>,
    /// USD price per million cached image input tokens.
    ///
    /// If unspecified, defaults to `image_input`.
    pub image_input_cached: Option<f64>,
    /// USD price per million video input tokens.
    ///
    /// If unspecified, defaults to `text_input`.
    pub video_input: Option<f64>,
    /// USD price per million cached video input tokens.
    ///
    /// If unspecified, defaults to `video_input`.
    pub video_input_cached: Option<f64>,
    /// USD price per million audio input tokens.
    ///
    /// If unspecified, defaults to `text_input`.
    pub audio_input: Option<f64>,
    /// USD price per million cached audio input tokens.
    ///
    /// If unspecified, defaults to `audio_input`.
    pub audio_input_cached: Option<f64>,
    /// USD price per million output tokens.
    pub output: f64,
}

impl InteractionsPricing {
    /// Creates a new [`InteractionsPricing`] with the given prices.
    pub fn new(text_input: f64, text_input_cached: Option<f64>, output: f64) -> Self {
        InteractionsPricing {
            text_input,
            text_input_cached,
            image_input: None,
            image_input_cached: None,
            video_input: None,
            video_input_cached: None,
            audio_input: None,
            audio_input_cached: None,
            output,
        }
    }

    /// Sets the [`image_input`](Self::image_input) price.
    pub fn with_image_input(mut self, price: f64) -> Self {
        self.image_input = Some(price);
        self
    }

    /// Sets the [`image_input_cached`](Self::image_input_cached) price.
    pub fn with_image_input_cached(mut self, price: f64) -> Self {
        self.image_input_cached = Some(price);
        self
    }

    /// Sets the [`video_input`](Self::video_input) price.
    pub fn with_video_input(mut self, price: f64) -> Self {
        self.video_input = Some(price);
        self
    }

    /// Sets the [`video_input_cached`](Self::video_input_cached) price.
    pub fn with_video_input_cached(mut self, price: f64) -> Self {
        self.video_input_cached = Some(price);
        self
    }

    /// Sets the [`audio_input`](Self::audio_input) price.
    pub fn with_audio_input(mut self, price: f64) -> Self {
        self.audio_input = Some(price);
        self
    }

    /// Sets the [`audio_input_cached`](Self::audio_input_cached) price.
    pub fn with_audio_input_cached(mut self, price: f64) -> Self {
        self.audio_input_cached = Some(price);
        self
    }

    /// Builds the usage lines for `usage`.
    ///
    /// When `pricing` is `Some`, each line carries its resolved unit price; when `None`, prices are
    /// omitted.
    fn usage_lines(usage: Usage, pricing: Option<&Self>) -> UsageLines {
        let (text, image, video, audio) =
            split_modality_tokens(usage.input_tokens_by_modality.as_deref());
        let (text_cached, image_cached, video_cached, audio_cached) =
            split_modality_tokens(usage.cached_tokens_by_modality.as_deref());

        let tool_use = non_negative(usage.total_tool_use_tokens);
        let output = non_negative(usage.total_output_tokens);
        let thinking = non_negative(usage.total_thought_tokens);

        // Resolved unit prices (per million tokens), or `None` when building default lines. Each
        // modality falls back to `text_input`, and its cached variant to the non-cached price.
        let text_input = pricing.map(|p| p.text_input);
        let image_input = pricing.map(|p| p.image_input.unwrap_or(p.text_input));
        let video_input = pricing.map(|p| p.video_input.unwrap_or(p.text_input));
        let audio_input = pricing.map(|p| p.audio_input.unwrap_or(p.text_input));
        let text_cached_price = pricing.map(|p| p.text_input_cached.unwrap_or(p.text_input));
        let image_cached_price = pricing.map(|p| {
            p.image_input_cached
                .unwrap_or(p.image_input.unwrap_or(p.text_input))
        });
        let video_cached_price = pricing.map(|p| {
            p.video_input_cached
                .unwrap_or(p.video_input.unwrap_or(p.text_input))
        });
        let audio_cached_price = pricing.map(|p| {
            p.audio_input_cached
                .unwrap_or(p.audio_input.unwrap_or(p.text_input))
        });
        let output_price = pricing.map(|p| p.output);

        UsageLines::from_iter([
            (
                TEXT_INPUT_TOKENS.into(),
                UsageLine::per_million_tokens(text.saturating_sub(text_cached), text_input),
            ),
            (
                TEXT_INPUT_TOKENS_CACHED.into(),
                UsageLine::per_million_tokens(text_cached, text_cached_price),
            ),
            (
                IMAGE_INPUT_TOKENS.into(),
                UsageLine::per_million_tokens(image.saturating_sub(image_cached), image_input),
            ),
            (
                IMAGE_INPUT_TOKENS_CACHED.into(),
                UsageLine::per_million_tokens(image_cached, image_cached_price),
            ),
            (
                VIDEO_INPUT_TOKENS.into(),
                UsageLine::per_million_tokens(video.saturating_sub(video_cached), video_input),
            ),
            (
                VIDEO_INPUT_TOKENS_CACHED.into(),
                UsageLine::per_million_tokens(video_cached, video_cached_price),
            ),
            (
                AUDIO_INPUT_TOKENS.into(),
                UsageLine::per_million_tokens(audio.saturating_sub(audio_cached), audio_input),
            ),
            (
                AUDIO_INPUT_TOKENS_CACHED.into(),
                UsageLine::per_million_tokens(audio_cached, audio_cached_price),
            ),
            (
                TOOL_USE_INPUT_TOKENS.into(),
                UsageLine::per_million_tokens(tool_use, text_input),
            ),
            (
                OUTPUT_TOKENS.into(),
                UsageLine::per_million_tokens(output, output_price),
            ),
            (
                THINKING_TOKENS.into(),
                UsageLine::per_million_tokens(thinking, output_price),
            ),
        ])
    }
}

impl LlmUsage for Usage {}

impl LlmPricing for InteractionsPricing {
    type Usage = Usage;

    fn default_usage_lines(usage: Self::Usage) -> UsageLines {
        Self::usage_lines(usage, None)
    }

    fn compute_usage_lines(&self, usage: Self::Usage) -> UsageLines {
        Self::usage_lines(usage, Some(self))
    }
}

/// Clamps an optional signed token count to a non-negative `u32`.
fn non_negative(count: Option<i32>) -> u32 {
    count.unwrap_or(0).max(0) as u32
}

/// Splits a per-modality token breakdown into `(text, image, video, audio)` counts.
fn split_modality_tokens(details: Option<&[ModalityTokens]>) -> (u32, u32, u32, u32) {
    let Some(details) = details else {
        return (0, 0, 0, 0);
    };

    let mut text = 0;
    let mut image = 0;
    let mut video = 0;
    let mut audio = 0;

    for entry in details {
        let count = non_negative(entry.tokens);
        match entry.modality {
            Some(ResponseModality::Image) => image += count,
            Some(ResponseModality::Video) => video += count,
            Some(ResponseModality::Audio) => audio += count,
            // Handle text, documents, and future variants as text.
            _ => text += count,
        }
    }

    (text, image, video, audio)
}
