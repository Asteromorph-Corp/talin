//! OpenAPI description of the Messages API, derived from this crate's types.

use utoipa::OpenApi;

use crate::claude_messages::types::{
    ApiErrorResponse, ContentBlockParam, ContentBlockSourceContent, MessagesRequest,
    MessagesResponse, RawMessageStreamEvent, SystemBlockParam, ToolResultBlockParamContent,
};

/// Creates a message.
#[utoipa::path(
    post,
    path = "/v1/messages",
    operation_id = "createMessage",
    tag = "Messages",
    summary = "Send a structured list of input messages with text and/or image content, and the \
               model will generate the next message in the conversation.",
    params(
        (
            "anthropic-version" = String, Header,
            description = "The version of the Anthropic API to use (e.g. 2023-06-01).",
        ),
        ("anthropic-user-profile-id" = Option<String>, Header, nullable = false),
        ("anthropic-workspace-id" = Option<String>, Header, nullable = false)
    ),
    request_body(
        content = MessagesRequest,
        description = "The request body.",
        content_type = "application/json",
    ),
    responses(
        (
            status = 200,
            description = "Successful message response",
            content((MessagesResponse = "application/json"), (RawMessageStreamEvent = "text/event-stream")),
        ),
        (status = "4XX", description = "Error creating a message", body = ApiErrorResponse),
        (status = "5XX", description = "Error creating a message", body = ApiErrorResponse),
    ),
    security(("apiKeyAuth" = [])),
)]
#[allow(dead_code)]
fn create_message() {}

/// The derived OpenAPI document.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Claude Messages API",
        description = "OpenAPI description of the Claude Messages API as modelled by the \
                       `claude-messages` crate.",
    ),
    servers((url = "https://api.anthropic.com", description = "Production Server")),
    paths(create_message),
    components(schemas(
        MessagesRequest,
        MessagesResponse,
        ApiErrorResponse,
        RawMessageStreamEvent,
        // Reached only through a nested generic, which `utoipa` inlines rather than follows, so
        // the `schema_with` helper that writes the `$ref` needs the component to exist.
        SystemBlockParam,
        ContentBlockParam,
        ContentBlockSourceContent,
        ToolResultBlockParamContent,
    )),
    modifiers(&SecurityAddon, &Normalize),
    tags((name = "Messages", description = "The Messages API.")),
)]
struct ApiDoc;

/// Registers the `apiKeyAuth` security scheme referenced by the operation.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "apiKeyAuth",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "x-api-key",
                "An Anthropic API key sent as the `x-api-key` header.",
            ))),
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
#[allow(dead_code)]
mod normalize {
    use utoipa::openapi::OpenApi;

    use crate::spec::normalize::{alias_variants, mark_tags_nullable, remove_title, visit_schemas};

    pub(super) fn normalize(api: &mut OpenApi) {
        visit_schemas(api, |path, node| {
            mark_tags_nullable(TAG_NULLABLE_VARIANTS, path, node)
        });
        visit_schemas(api, |path, node| {
            alias_variants(ALIASED_VARIANTS, path, node)
        });
        visit_schemas(api, |path, node| remove_title(KEEP_TITLES, path, node));
    }

    /// (`title`, `tag`) tuples of the variants whose `tag` property is nullable in the upstream
    /// spec.
    const TAG_NULLABLE_VARIANTS: &[(&str, &str)] = &[("Tool", "type")];

    /// (`title`, `tag`, aliases) tuples of the variants whose `tag` property accepts more values
    /// in the upstream spec.
    const ALIASED_VARIANTS: &[(&str, &str, &[&str])] = &[
        (
            "RegexToolSearchTool20251119",
            "type",
            &["tool_search_tool_regex"],
        ),
        ("Bm25ToolSearch20251119", "type", &["tool_search_tool_bm25"]),
    ];

    /// The titles to keep in the generated spec.
    const KEEP_TITLES: &[&str] = &[];
}

pub(crate) mod schema {
    use utoipa::openapi::schema::{AdditionalProperties, ObjectBuilder, SchemaType, Type};
    use utoipa::openapi::{ArrayBuilder, OneOfBuilder, Ref, RefOr, Schema};

    /// Generate the schema:
    ///
    /// ```json
    /// { "type": "number" }
    /// ```
    pub(in crate::claude_messages) fn number_schema() -> utoipa::openapi::schema::Schema {
        Schema::Object(ObjectBuilder::new().schema_type(Type::Number).build())
    }

    /// Generate the schema:
    ///
    /// ```json
    /// { "type": ["number", "null"] }
    /// ```
    pub(in crate::claude_messages) fn nullable_number_schema() -> utoipa::openapi::schema::Schema {
        Schema::Object(
            ObjectBuilder::new()
                .schema_type(SchemaType::Array(vec![Type::Number, Type::Null]))
                .build(),
        )
    }

    /// Generates the schema:
    ///
    /// ```json
    /// {
    ///   "properties": {
    ///     "properties": {
    ///       "additionalProperties": true,
    ///       "type": ["object", "null"]
    ///     },
    ///     "required": {
    ///       "items": { "type": "string" },
    ///       "type": ["array", "null"]
    ///     },
    ///     "type": { "enum": ["object"], "type": "string" }
    ///   },
    ///   "required": ["type"],
    ///   "type": "object"
    /// }
    /// ```
    pub(in crate::claude_messages) fn input_schema_schema() -> utoipa::openapi::schema::Schema {
        let properties = ObjectBuilder::new()
            .schema_type(SchemaType::Array(vec![Type::Object, Type::Null]))
            .additional_properties(Some(AdditionalProperties::FreeForm(true)))
            .build();
        let required = ArrayBuilder::new()
            .schema_type(SchemaType::Array(vec![Type::Array, Type::Null]))
            .items(utoipa::openapi::schema::ArrayItems::RefOrSchema(Box::new(
                RefOr::T(Schema::Object(
                    ObjectBuilder::new().schema_type(Type::String).build(),
                )),
            )))
            .build();
        let type_ = ObjectBuilder::new()
            .schema_type(Type::String)
            .enum_values(Some(vec![serde_json::Value::from("object")]))
            .build();
        let object = ObjectBuilder::new()
            .property("properties", properties)
            .property("required", required)
            .property("type", type_)
            .required("type")
            .schema_type(Type::Object)
            .build();
        Schema::Object(object)
    }

    pub(in crate::claude_messages) fn text_or_items_schema(
        name: &str,
    ) -> utoipa::openapi::schema::Schema {
        let text = ObjectBuilder::new().schema_type(Type::String).build();
        let items = ArrayBuilder::new()
            .items(RefOr::Ref(Ref::new(format!("#/components/schemas/{name}"))))
            .build();
        let one_of = OneOfBuilder::new().item(text).item(items).build();
        Schema::OneOf(one_of)
    }

    pub(in crate::claude_messages) fn text_or_system_block_params_schema()
    -> utoipa::openapi::schema::Schema {
        text_or_items_schema("SystemBlockParam")
    }

    pub(in crate::claude_messages) fn text_or_content_block_params_schema()
    -> utoipa::openapi::schema::Schema {
        text_or_items_schema("ContentBlockParam")
    }

    pub(in crate::claude_messages) fn text_or_content_block_source_contents_schema()
    -> utoipa::openapi::schema::Schema {
        text_or_items_schema("ContentBlockSourceContent")
    }

    pub(in crate::claude_messages) fn text_or_tool_result_block_param_contents_schema()
    -> utoipa::openapi::schema::Schema {
        text_or_items_schema("ToolResultBlockParamContent")
    }

    /// Generate the schema:
    ///
    /// ```json
    /// {
    ///   "type": "array",
    ///   "items": {
    ///     "type": "object",
    ///     "additionalProperties": true
    ///   }
    /// }
    /// ```
    pub(in crate::claude_messages) fn object_array_schema() -> utoipa::openapi::schema::Schema {
        Schema::Array(ArrayBuilder::new().items(RefOr::T(object_schema())).build())
    }

    /// Generate the schema:
    /// ```json
    /// {
    ///   "type": "object",
    ///   "additionalProperties": true
    /// }
    /// ```
    pub(in crate::claude_messages) fn object_schema() -> utoipa::openapi::schema::Schema {
        let object = ObjectBuilder::new()
            .schema_type(Type::Object)
            .additional_properties(Some(AdditionalProperties::FreeForm(true)))
            .build();
        Schema::Object(object)
    }
}

#[cfg(test)]
mod tests {
    use utoipa::OpenApi;

    use super::ApiDoc;

    #[test]
    #[cfg_attr(feature = "mcp-client-2025-11-20", ignore)]
    fn generated_spec_matches_types() {
        crate::spec::check(
            "spec/claude_messages/openapi.generated.json",
            &ApiDoc::openapi(),
        );
    }
}
