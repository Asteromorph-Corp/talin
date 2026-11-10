//! OpenAPI description of the Responses API, derived from this crate's types.

use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::gemini_interactions::types::{
    CreateInteractionRequest, ErrorMessage, Interaction, InteractionSseStreamEnvelope,
};

/// The body of an error response (`4XX`/`5XX`), wrapping an [`ErrorMessage`].
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
struct ErrorResponse {
    /// The error details.
    pub error: ErrorMessage,
}

/// Creates a new interaction.
#[utoipa::path(
    post,
    path = "/{api_version}/interactions",
    operation_id = "CreateInteraction",
    tag = "interactions",
    request_body(
        content = CreateInteractionRequest,
        description = "The request body.",
        content_type = "application/json",
    ),
    responses(
        (
            status = 200,
            description = "Successful operation",
            content(
                (Interaction = "application/json"),
                (InteractionSseStreamEnvelope = "text/event-stream"),
            ),
        ),
        (status = "4XX", description = "Error creating interaction", body = ErrorResponse),
        (status = "5XX", description = "Error creating interaction", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
fn create_interaction() {}

/// The derived OpenAPI document.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Gemini Interactions API",
        description = "OpenAPI description of the Gemini Interactions API as modelled by the \
                       `gemini-interactions` crate.",
    ),
    servers((
        url = "https://generativelanguage.googleapis.com",
        description = "Global Endpoint"
    )),
    paths(create_interaction),
    components(schemas(
        CreateInteractionRequest,
        Interaction,
        InteractionSseStreamEnvelope,
        ErrorResponse,
    )),
    modifiers(&SecurityAddon, &Normalize),
    tags((name = "interactions", description = "The Interactions API.")),
)]
struct ApiDoc;

/// Registers the `googleGenAIAuth` security scheme referenced by the operation.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "googleGenAIAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .description(Some(
                        "An OAuth access token sent as a bearer `Authorization` header, or a \
                         Gemini API key sent as `x-goog-api-key`.",
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
        object_schemas, remove_title, tags_to_const, unconstrain_property_names, untag_variants,
        untype_schemas, visit_schemas,
    };

    /// Applies every correction to the derived document in place.
    pub(super) fn normalize(api: &mut OpenApi) {
        visit_schemas(api, |path, node| untype_schemas(TO_UNTYPE, path, node));
        visit_schemas(api, |path, node| {
            untag_variants(UNTAGGED_VARIANTS, path, node)
        });
        visit_schemas(api, unconstrain_property_names);
        visit_schemas(api, |path, node| tags_to_const(TAGGED, path, node));
        visit_schemas(api, |path, node| object_schemas(TO_OBJECT, path, node));
        visit_schemas(api, |path, node| remove_title(KEEP_TITLES, path, node));
    }

    /// The titles of objects that do not have `"type"` specification in the upstream spec.
    const TO_UNTYPE: &[&str] = &[
        "CreateInteractionRequest",
        "AgentConfig",
        "CreateModelInteractionParams",
        "CreateAgentInteractionParams",
        "Interaction",
    ];

    /// The titles of unions that have `"type": "object"` specification in the upstream spec.
    const TO_OBJECT: &[&str] = &[
        "Annotation",
        "Content",
        "InteractionSseEvent",
        "ResponseFormat",
        "Step",
        "StepDeltaData",
        "Tool",
    ];

    /// (`title`, `tag`) tuples of the `#[serde(untagged)]` variants.
    const UNTAGGED_VARIANTS: &[(&str, &str)] = &[("OtherResponseFormat", "type")];

    const TAGGED: &[(&str, &str)] = &[
        ("Content", "type"),
        ("Annotation", "type"),
        ("Step", "type"),
        ("ThoughtContent", "type"),
        ("FunctionResultSubContent", "type"),
        ("Tool", "type"),
        ("ResponseFormat", "type"),
        ("EnvironmentConfig", "type"),
        ("MediaProcessing", "type"),
        ("TranscriptionMode", "type"),
        ("Ranking", "ranking_config"),
        ("InteractionSseEvent", "event_type"),
        ("AgentConfig", "type"),
        ("ContentDeltaData", "type"),
        ("StepDeltaData", "type"),
    ];

    /// The titles to keep in the generated spec.
    const KEEP_TITLES: &[&str] = &["StepList", "ContentList", "ResponseFormatList"];
}

pub(crate) mod schema {
    use utoipa::openapi::schema::{AdditionalProperties, Schema, SchemaType};
    use utoipa::openapi::{ObjectBuilder, RefOr, Type};

    /// Generate the schema:
    /// ```json
    /// {
    ///   "type": "object",
    ///   "additionalProperties": { "type": "string" }
    /// }
    /// ```
    pub(in crate::gemini_interactions) fn map_schema() -> utoipa::openapi::schema::Schema {
        let additional_properties_inner = ObjectBuilder::new()
            .schema_type(SchemaType::Type(Type::String))
            .build();
        let additional_properties =
            AdditionalProperties::RefOr(RefOr::T(Schema::Object(additional_properties_inner)));
        let object_inner = ObjectBuilder::new()
            .schema_type(SchemaType::Type(Type::Object))
            .additional_properties(Some(additional_properties))
            .build();
        Schema::Object(object_inner)
    }

    /// Generate the schema:
    /// ```json
    /// {
    ///   "type": "object",
    ///   "additionalProperties": {}
    /// }
    /// ```
    pub(in crate::gemini_interactions) fn object_schema() -> utoipa::openapi::schema::Schema {
        let additional_properties_inner = ObjectBuilder::new()
            .schema_type(SchemaType::AnyValue)
            .build();
        let additional_properties =
            AdditionalProperties::RefOr(RefOr::T(Schema::Object(additional_properties_inner)));
        let object_inner = ObjectBuilder::new()
            .schema_type(SchemaType::Type(Type::Object))
            .additional_properties(Some(additional_properties))
            .build();
        Schema::Object(object_inner)
    }
}

#[cfg(test)]
mod tests {
    use utoipa::OpenApi;

    use super::ApiDoc;

    #[test]
    fn generated_spec_matches_types() {
        crate::spec::check(
            "spec/gemini_interactions/openapi.generated.json",
            &ApiDoc::openapi(),
        );
    }
}
