use quote::quote;
use serde::{Deserialize, Serialize};
use syn::parse_quote;

use crate::codegen::common::talin;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GeminiInteractionsConfig {
    pub pricing: Option<GeminiInteractionsPricing>,
}

impl GeminiInteractionsConfig {
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        let with_pricing = self.pricing.as_ref().map(|p| {
            let expr = p.to_expr();
            quote! { .with_pricing(#expr) }
        });
        [with_pricing].into_iter().flatten().collect()
    }
}

/// The pricing specification of a Gemini Interactions API model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GeminiInteractionsPricing {
    /// USD price per million text input tokens.
    pub input_tokens: f64,
    /// USD price per million cached text input tokens.
    ///
    /// If unspecified, defaults to `input_tokens`.
    pub cached_input_tokens: Option<f64>,
    /// USD price per million image input tokens.
    ///
    /// If unspecified, defaults to `text_input`.
    pub image_input_tokens: Option<f64>,
    /// USD price per million cached image input tokens.
    ///
    /// If unspecified, defaults to `image_input`.
    pub cached_image_input_tokens: Option<f64>,
    /// USD price per million video input tokens.
    ///
    /// If unspecified, defaults to `text_input`.
    pub video_input_tokens: Option<f64>,
    /// USD price per million cached video input tokens.
    ///
    /// If unspecified, defaults to `video_input_tokens`.
    pub cached_video_input_tokens: Option<f64>,
    /// USD price per million audio input tokens.
    ///
    /// If unspecified, defaults to `input_tokens`.
    pub audio_input_tokens: Option<f64>,
    /// USD price per million cached audio input tokens.
    ///
    /// If unspecified, defaults to `audio_input_tokens`.
    pub cached_audio_input_tokens: Option<f64>,
    /// USD price per million output tokens.
    ///
    /// Also used for thinking tokens.
    pub output_tokens: f64,
}

macro_rules! maybe_with {
    ($field:expr, $method:ident) => {
        $field.map(|value| {
            quote! { .$method(#value) }
        })
    };
}

impl GeminiInteractionsPricing {
    fn to_expr(&self) -> syn::Expr {
        let input_tokens = self.input_tokens;
        let cached_input_tokens: syn::Expr = match self.cached_input_tokens {
            Some(price) => parse_quote!(Some(#price)),
            None => parse_quote!(None),
        };
        let output_tokens = self.output_tokens;

        let maybe_with_image_input = maybe_with!(self.image_input_tokens, with_image_input);
        let maybe_with_image_input_cached =
            maybe_with!(self.cached_image_input_tokens, with_image_input_cached);
        let maybe_with_video_input = maybe_with!(self.video_input_tokens, with_video_input);
        let maybe_with_video_input_cached =
            maybe_with!(self.cached_video_input_tokens, with_video_input_cached);
        let maybe_with_audio_input = maybe_with!(self.audio_input_tokens, with_audio_input);
        let maybe_with_audio_input_cached =
            maybe_with!(self.cached_audio_input_tokens, with_audio_input_cached);

        let talin = talin();
        parse_quote! {
            #talin::llm::gemini_interactions::InteractionsPricing::new(
                #input_tokens,
                #cached_input_tokens,
                #output_tokens,
            )
            #maybe_with_image_input
            #maybe_with_image_input_cached
            #maybe_with_video_input
            #maybe_with_video_input_cached
            #maybe_with_audio_input
            #maybe_with_audio_input_cached
        }
    }
}
