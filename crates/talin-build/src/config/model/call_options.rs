use std::collections::BTreeMap;

use quote::quote;
use serde::{Deserialize, Serialize};
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::{Merge, ReasoningEffort, ToolChoice};

macro_rules! with_option {
    ($config:ident, $field:ident) => {
        $config.$field.map(|value| {
            let method = ::quote::format_ident!("with_{}", stringify!($field));
            ::quote::quote! { .#method(#value) }
        })
    };
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CallOptions {
    pub stream: Option<bool>,
    pub tool_choice: Option<ToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    pub n: Option<u8>,
    pub reasoning_effort: Option<ReasoningEffort>,

    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,

    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub repetition_penalty: Option<f32>,

    pub max_tokens: Option<u32>,
    pub max_completion_tokens: Option<u32>,
    pub stop_words: Option<Vec<String>>,
    pub extra_params: Option<BTreeMap<String, toml::Value>>,
}

impl CallOptions {
    pub(crate) fn to_expr(&self) -> syn::Expr {
        let talin = talin();

        let with_tool_choice = self.tool_choice.as_ref().map(|choice| {
            let choice = choice.to_expr();
            quote! {
                .with_tool_choice(#talin::schemas::ToolChoice::#choice)
            }
        });
        let with_reasoning_effort = self.reasoning_effort.as_ref().map(|effort| {
            let effort = effort.as_variant();
            quote! {
                .with_reasoning_effort(#talin::schemas::ReasoningEffort::#effort)
            }
        });
        let with_stop_words = self.stop_words.as_ref().map(|words| {
            quote! {
                .with_stop_words(vec![#(String::from(#words)),*])
            }
        });

        let with_extra_params = self.extra_params.as_ref().map(|extra_params| {
            let entries = extra_params.iter().map(|(k, v)| -> syn::Expr {
                let v_tokens = toml_to_json_tokens(v);
                parse_quote! { (String::from(#k), #v_tokens)}
            });
            quote! {
                .with_extra_params(std::collections::HashMap::from_iter([
                    #(#entries,)*
                ]))
            }
        });

        let with_options = [
            with_option!(self, stream),
            with_tool_choice,
            with_option!(self, parallel_tool_calls),
            with_option!(self, n),
            with_reasoning_effort,
            with_option!(self, temperature),
            with_option!(self, top_p),
            with_option!(self, top_k),
            with_option!(self, frequency_penalty),
            with_option!(self, presence_penalty),
            with_option!(self, repetition_penalty),
            with_option!(self, max_tokens),
            with_option!(self, max_completion_tokens),
            with_stop_words,
            with_extra_params,
        ];

        parse_quote! {
            #talin::llm::LlmOptions::default()
                #(#with_options)*
        }
    }
}

impl Merge<Self> for CallOptions {
    /// Merges this CallOptions with a parent CallOptions.
    /// Child values override parent values where specified.
    fn merge(self, parent: Self) -> Self {
        CallOptions {
            stream: self.stream.or(parent.stream),
            tool_choice: self.tool_choice.or(parent.tool_choice),
            parallel_tool_calls: self.parallel_tool_calls.or(parent.parallel_tool_calls),
            n: self.n.or(parent.n),
            reasoning_effort: self.reasoning_effort.or(parent.reasoning_effort),
            temperature: self.temperature.or(parent.temperature),
            top_p: self.top_p.or(parent.top_p),
            top_k: self.top_k.or(parent.top_k),
            frequency_penalty: self.frequency_penalty.or(parent.frequency_penalty),
            presence_penalty: self.presence_penalty.or(parent.presence_penalty),
            repetition_penalty: self.repetition_penalty.or(parent.repetition_penalty),
            max_tokens: self.max_tokens.or(parent.max_tokens),
            max_completion_tokens: self.max_completion_tokens.or(parent.max_completion_tokens),
            stop_words: self.stop_words.or(parent.stop_words),
            extra_params: self.extra_params.merge(parent.extra_params),
        }
    }
}

fn toml_to_json_tokens(value: &toml::Value) -> syn::Expr {
    let talin = talin();
    let serde_json: syn::Path = parse_quote! { #talin::__private::serde_json };

    match value {
        toml::Value::String(s) => parse_quote! { #serde_json::Value::String(String::from(#s)) },
        toml::Value::Integer(i) => {
            parse_quote! { #serde_json::Value::Number(#serde_json::Number::from(#i)) }
        }
        toml::Value::Float(f) => {
            parse_quote! { #serde_json::Value::Number(#serde_json::Number::from_f64(#f).expect("toml float is finite")) }
        }
        toml::Value::Boolean(b) => parse_quote! { #serde_json::Value::Bool(#b) },
        toml::Value::Datetime(datetime) => {
            let s = datetime.to_string();
            parse_quote! { #serde_json::Value::String(String::from(#s)) }
        }
        toml::Value::Array(arr) => {
            let elems = arr.iter().map(toml_to_json_tokens);
            parse_quote! { #serde_json::Value::Array(vec![#(#elems),*]) }
        }
        toml::Value::Table(table) => {
            let entries = table.iter().map(|(k, v)| -> syn::Expr {
                let value_expr = toml_to_json_tokens(v);
                parse_quote! { (String::from(#k), #value_expr) }
            });
            parse_quote! {
                #serde_json::Value::Object(#serde_json::Map::from_iter([
                    #(#entries,)*
                ]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{assert_expr_eq, call_options};

    #[rstest]
    #[case::normal(call_options(), "common/options/model_options.rs")]
    #[case::default(CallOptions::default(), "common/options/model_options.default.rs")]
    fn test_model_options_expr(#[case] call_options: CallOptions, #[case] fixture_path: &str) {
        let expr = call_options.to_expr();
        assert_expr_eq(&expr, fixture_path);
    }
}
