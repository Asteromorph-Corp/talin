use quote::quote;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::InlineOrEnv;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "lowercase", remote = "Self")]
pub(crate) enum ClaudeMessagesConfig {
    Anthropic(AnthropicClaudeMessagesConfig),
    Vertex(VertexClaudeMessagesConfig),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AnthropicClaudeMessagesConfig {
    pub pricing: Option<ClaudeMessagesPricing>,
    pub anthropic_version: Option<String>,
    pub anthropic_beta: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VertexClaudeMessagesConfig {
    pub project_id: InlineOrEnv<String>,
    pub location: Option<InlineOrEnv<String>>,
    pub pricing: Option<ClaudeMessagesPricing>,
    pub anthropic_version: Option<String>,
    pub anthropic_beta: Option<Vec<String>>,
}

impl ClaudeMessagesConfig {
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        match self {
            ClaudeMessagesConfig::Anthropic(config) => config.builder_options(),
            ClaudeMessagesConfig::Vertex(config) => config.builder_options(),
        }
    }
}

impl AnthropicClaudeMessagesConfig {
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        let with_pricing = self.pricing.as_ref().map(|p| {
            let expr = p.to_expr();
            quote! { .with_pricing(#expr) }
        });
        let with_anthropic_version = self.anthropic_version.as_ref().map(|version| {
            quote! { .with_anthropic_version(#version) }
        });
        let with_anthropic_beta = self.anthropic_beta.as_ref().map(|betas| {
            quote! { .with_anthropic_betas([#(String::from(#betas)),*]) }
        });

        [with_pricing, with_anthropic_version, with_anthropic_beta]
            .into_iter()
            .flatten()
            .collect()
    }
}

impl VertexClaudeMessagesConfig {
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        let with_pricing = self.pricing.as_ref().map(|pricing| {
            let expr = pricing.to_expr();
            quote! { .with_pricing(#expr) }
        });

        let talin = talin();
        let project_id = self.project_id.to_expr();
        let as_vertex = quote! {
            .vertex(#project_id)
            .with_token_provider(
                #talin::llm::vertex::LazyVertexCredentials::default()
            )
        };

        let with_location = self.location.as_ref().map(|loc| {
            let expr = loc.to_expr();
            quote! { .with_location(#expr) }
        });

        [with_pricing, Some(as_vertex), with_location]
            .into_iter()
            .flatten()
            .collect()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClaudeMessagesPricing {
    /// USD price per million input tokens.
    pub input_tokens: f64,
    /// USD price for cache writes with 5m ttl (per million tokens).
    ///
    /// If unspecified, defaults to 1.25x `base_input_tokens`.
    pub cache_writes_5m: Option<f64>,
    /// USD price for cache writes with 1h ttl (per million tokens).
    ///
    /// If unspecified, defaults to 2x `base_input_tokens`
    pub cache_writes_1h: Option<f64>,
    /// USD price for cache hits and refreshes (per million tokens).
    ///
    /// If unspecified, defaults to 0.1x `base_input_tokens`
    pub cache_hits_refreshes: Option<f64>,
    /// USD price per million output tokens.
    pub output_tokens: f64,
    /// USD price per web search request.
    pub web_search: Option<f64>,
}

impl ClaudeMessagesPricing {
    fn to_expr(&self) -> syn::Expr {
        let input_tokens = self.input_tokens;
        let output_tokens = self.output_tokens;
        let maybe_with_cache_writes_5m = self.cache_writes_5m.map(|cache_writes_5m| {
            quote! { .with_cache_writes_5m(#cache_writes_5m) }
        });
        let maybe_with_cache_writes_1h = self.cache_writes_1h.map(|cache_writes_1h| {
            quote! { .with_cache_writes_1h(#cache_writes_1h) }
        });
        let maybe_with_cache_hits_refreshes = self.cache_hits_refreshes.map(|cache_hits| {
            quote! { .with_cache_hits_refreshes(#cache_hits) }
        });
        let maybe_with_web_search = self.web_search.map(|web_search| {
            quote! { .with_web_search(#web_search) }
        });

        let talin = talin();
        parse_quote! {
            #talin::llm::claude_messages::MessagesPricing::new(
                #input_tokens,
                #output_tokens
            )
            #maybe_with_cache_writes_5m
            #maybe_with_cache_writes_1h
            #maybe_with_cache_hits_refreshes
            #maybe_with_web_search
        }
    }
}

impl Serialize for ClaudeMessagesConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Self::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for ClaudeMessagesConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut value = toml::Value::deserialize(deserializer)?;

        if let Some(table) = value.as_table_mut()
            && !table.contains_key("backend")
        {
            table.insert("backend".to_owned(), toml::Value::from("anthropic"));
        }
        ClaudeMessagesConfig::deserialize(value).map_err(D::Error::custom)
    }
}
