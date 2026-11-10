//! OpenAPI description of the Responses API, derived from this crate's types.

use utoipa::OpenApi;

use crate::xai_responses::types::{ModelRequest, ModelResponse};

/// Creates a new response.
#[utoipa::path(
    post,
    path = "/v1/responses",
    operation_id = "handle_generic_model_request",
    tag = "v1",
    summary = "Generates a response based on text or image prompts. The response ID can be used \
               to retrieve the response later or to continue the conversation without repeating \
               prior context. New responses will be stored for 30 days and then permanently \
               deleted.",
    request_body(content = ModelRequest, content_type = "application/json"),
    responses((
        status = 200,
        description = "Success",
        content((ModelResponse = "application/json")),
    )),
    security(("bearerAuth" = [])),
)]
#[allow(dead_code)]
fn create_response() {}

/// The derived OpenAPI document.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "xAI Responses API",
        description = "OpenAPI description of the xAI Responses API as modelled by the \
                       `xai-responses` crate.",
    ),
    paths(create_response),
    components(schemas(ModelRequest, ModelResponse)),
    modifiers(&SecurityAddon, &Normalize),
    tags((name = "v1", description = "The Responses API.")),
)]
struct ApiDoc;

/// Registers the `bearerAuth` security scheme referenced by the operation.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .description(Some(
                        "An xAI API key sent as a bearer `Authorization` header.",
                    ))
                    .build(),
            ),
        );
    }
}

struct Normalize;

impl utoipa::Modify for Normalize {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        normalize::normalize(openapi);
        openapi.info.version = String::from("0.0.0");
    }
}

/// Rewrites `utoipa`'s rendering of the derived types into the shape the API actually uses.
mod normalize {
    use utoipa::openapi::OpenApi;

    use crate::spec::normalize::{
        mark_tags_nullable, remove_title, unconstrain_tags, untag_variants, visit_schemas,
    };

    pub(super) fn normalize(api: &mut OpenApi) {
        visit_schemas(api, |path, node| {
            untag_variants(UNTAGGED_VARIANTS, path, node)
        });
        visit_schemas(api, |path, node| {
            unconstrain_tags(TAG_UNCONSTRAINED_VARIANTS, path, node)
        });
        visit_schemas(api, |path, node| {
            mark_tags_nullable(TAG_NULLABLE_VARIANTS, path, node)
        });
        visit_schemas(api, |path, node| remove_title(KEEP_TITLES, path, node));
    }

    /// (`title`, `tag`) tuples of the `#[serde(untagged)]` variants.
    const UNTAGGED_VARIANTS: &[(&str, &str)] =
        &[("ModelOutput", "type"), ("ModelToolChoiceMode", "type")];

    /// (`title`, `tag`) tuples of the variants whose `tag` property is unconstrained in the
    /// upstream spec.
    const TAG_UNCONSTRAINED_VARIANTS: &[(&str, &str)] = &[
        ("InputMessage", "type"),
        ("FunctionToolCallOutput", "type"),
        ("ShellCallOutput", "type"),
        ("CompactionOutputItem", "type"),
        ("OutputMessage", "type"),
        ("FunctionToolCall", "type"),
        ("Reasoning", "type"),
        ("ReasoningTextInner", "type"),
        ("SummaryTextInner", "type"),
        ("WebSearchCall", "type"),
        ("FileSearchCall", "type"),
        ("CodeInterpreterCall", "type"),
        ("McpCall", "type"),
        ("CustomToolCall", "type"),
        ("ShellCall", "type"),
        ("LocalShellEnvironment", "type"),
        ("ShellCallActionExec", "type"),
        ("WebSearchUserLocationApproximate", "type"),
        ("ModelToolChoiceTool", "type"),
    ];

    /// (`title`, `tag`) tuples of the variants whose `tag` property is nullable in the upstream
    /// spec.
    const TAG_NULLABLE_VARIANTS: &[(&str, &str)] =
        &[("InputMessage", "type"), ("ShellCallActionExec", "type")];

    /// The titles to keep in the generated spec.
    const KEEP_TITLES: &[&str] = &[];
}

#[cfg(test)]
mod tests {
    use utoipa::OpenApi;

    use super::ApiDoc;

    #[test]
    fn generated_spec_matches_types() {
        crate::spec::check(
            "spec/xai_responses/openapi.generated.json",
            &ApiDoc::openapi(),
        );
    }
}
