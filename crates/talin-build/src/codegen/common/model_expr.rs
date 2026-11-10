use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::Model;

pub(crate) fn model_expr(model: &Model) -> syn::Expr {
    let talin = talin();

    let model_ty = model.model_preset().model_ty();
    let model_name = model.model.to_expr();

    let builder_args = model.builder_args();
    let api_options = model.builder_method_calls();
    let provider_options = model.provider_config.builder_options();
    let strategy_expr = model.strategy_expr();
    let model_options = model.call_options.to_expr();
    let default_headers = model.default_headers();

    let expr = parse_quote! {
        #[allow(clippy::default_constructed_unit_structs, clippy::unit_arg)]
        {
            use #talin::llm::Llm;

            #talin::llm::LlmPreset::new(
                #model_ty::builder(#builder_args)
                    #(#api_options)*
                    #(#provider_options)*
                    .with_model(#model_name)
                    .with_http_client(
                        #talin::__private::reqwest::Client::builder()
                            .connection_verbose(true)
                            #default_headers
                            .build()
                            .unwrap()
                    )
                    .build()
                    .with_default_options(#model_options),
                #strategy_expr
            )
        }
    };
    expr
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{assert_expr_eq, model_generic};

    #[rstest]
    fn test_model_expr(model_generic: Model) {
        let expr = model_expr(&model_generic);
        assert_expr_eq(&expr, "common/model_expr.rs");
    }

    #[test]
    fn test_model_expr_sglang_passes_api_base_to_builder() {
        let model: Model = toml::from_str(
            r#"
                api_base = "http://localhost:30000"
                model = "my_model"
                model_type = "sglang"
            "#,
        )
        .unwrap();
        let rendered = model_expr(&model).to_token_stream().to_string();
        assert!(
            rendered.contains(
                "SglangGenerate :: builder (String :: from (\"http://localhost:30000\"))"
            )
        );
        assert!(!rendered.contains("with_api_base"));
    }

    #[rstest]
    fn test_model_expr_with_headers() {
        let model: Model = toml::from_str(
            r#"
                api_base = "https://www.example.com"
                model = "my_model"
                headers = { "X-Tenant" = "acme", "x-gateway-token".env = "GATEWAY_TOKEN" }
            "#,
        )
        .unwrap();
        let expr = model_expr(&model);
        assert_expr_eq(&expr, "common/model_expr.headers.rs");
    }
}
