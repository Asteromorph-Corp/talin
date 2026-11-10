use heck::ToSnakeCase;
use quote::format_ident;
use syn::parse_quote;

use crate::codegen::common::{model_expr, talin};
use crate::config::{AgentModel, ModelPreset};

pub(super) fn fn_default_model(model: &AgentModel, preset: &ModelPreset) -> syn::ImplItemFn {
    let preset_ty = preset.to_ty();
    let model_def = match model {
        AgentModel::Inline(model) => model_expr(model),
        AgentModel::Reference(id) => {
            let fn_name = format_ident!("{}", id.to_snake_case());
            parse_quote! { __agent_deps::Models::#fn_name() }
        }
    };

    parse_quote! {
        pub fn default_model() -> #preset_ty {
            #model_def
        }
    }
}

pub(super) fn fn_default(
    preset: &ModelPreset,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
) -> [syn::ImplItemFn; 2] {
    let talin = talin();
    let model_ty = preset.model_ty();
    let strategy_ty = preset.strategy_ty();

    [
        parse_quote! {
            #[deprecated(note = "Use `default` instead")]
            pub async fn async_default() -> #talin::agent::Agent<#model_ty, #input_ctor, #output_ctor, #strategy_ty> {
                Self::default().await
            }
        },
        parse_quote! {
            pub async fn default() -> #talin::agent::Agent<#model_ty, #input_ctor, #output_ctor, #strategy_ty> {
                Self::new(Self::default_model()).await
            }
        },
    ]
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{
        assert_item_eq, default_model_inline, default_model_ref, input_ctor, output_ctor,
        preset_generic,
    };

    #[rstest]
    #[case(
        default_model_ref(),
        preset_generic(),
        "agent/default/fn_default_model.ref.rs"
    )]
    #[case(
        default_model_inline(),
        preset_generic(),
        "agent/default/fn_default_model.inline.rs"
    )]
    fn test_fn_default_model(
        #[case] model: AgentModel,
        #[case] preset: ModelPreset,
        #[case] fixture_path: &str,
    ) {
        let fn_default_model = fn_default_model(&model, &preset);
        assert_item_eq(&fn_default_model, fixture_path);
    }

    #[rstest]
    #[case(preset_generic())]
    fn test_fn_default(#[case] preset: ModelPreset, input_ctor: syn::Type, output_ctor: syn::Type) {
        let [fn_async_default, fn_default] = fn_default(&preset, &input_ctor, &output_ctor);
        assert_item_eq(&fn_async_default, "agent/default/fn_async_default.rs");
        assert_item_eq(&fn_default, "agent/default/fn_default.rs");
    }
}
