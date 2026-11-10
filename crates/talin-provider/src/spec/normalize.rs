//! Rewrites `utoipa`'s rendering of the derived types into the shape each API actually uses.
//!
//! Holds the passes and helpers every provider's `openapi::normalize` module is built from; which
//! passes run, in what order and over which schemas, is up to the provider.

use serde_json::Value;
use utoipa::openapi::extensions::Extensions;
use utoipa::openapi::schema::{AdditionalProperties, AllOf, ArrayItems, SchemaType, Type};
use utoipa::openapi::{Object, OpenApi, RefOr, Schema};

/// The prefix the path of a schema registered in the document's components carries.
const SCHEMA_PATH: &str = "#/components/schemas/";

/// Removes the `tag` property from the `#[serde(untagged)]` variants, which is emitted
/// incorrectly by `utoipa`.
///
/// `variants` holds the (`title`, `tag`) tuples of those variants.
pub(crate) fn untag_variants(variants: &[(&str, &str)], _: &str, schema: &mut RefOr<Schema>) {
    let RefOr::T(Schema::AllOf(all_of)) = schema else {
        return;
    };
    let Some(tag_name) = variant_tag(all_of, variants) else {
        return;
    };

    all_of.items.retain(|item| as_tag(item, tag_name).is_none());
    if all_of.items.len() == 1
        && let Some(item) = all_of.items.pop()
    {
        *schema = item;
    }
}

/// Marks the `tag` property nullable for the variants specified as such by upstream.
///
/// `variants` holds the (`title`, `tag`) tuples of those variants.
pub(crate) fn mark_tags_nullable(variants: &[(&str, &str)], _: &str, schema: &mut RefOr<Schema>) {
    let RefOr::T(Schema::AllOf(all_of)) = schema else {
        return;
    };
    let Some(tag_name) = variant_tag(all_of, variants) else {
        return;
    };

    for item in all_of.items.iter_mut() {
        if let Some(object) = as_tag_mut(item, tag_name)
            && let Some(RefOr::T(Schema::Object(tag))) = object.properties.values_mut().next()
        {
            object.required = vec![];
            tag.schema_type = SchemaType::Array(vec![Type::String, Type::Null]);
        }
    }
}

/// Removes the `enum` constraint from the `tag` property of the variants specified as such by
/// upstream.
///
/// `variants` holds the (`title`, `tag`) tuples of those variants.
pub(crate) fn unconstrain_tags(variants: &[(&str, &str)], _: &str, schema: &mut RefOr<Schema>) {
    let RefOr::T(Schema::AllOf(all_of)) = schema else {
        return;
    };
    let Some(tag_name) = variant_tag(all_of, variants) else {
        return;
    };

    for item in all_of.items.iter_mut() {
        if let Some(object) = as_tag_mut(item, tag_name)
            && let Some(RefOr::T(Schema::Object(tag))) = object.properties.values_mut().next()
        {
            tag.enum_values = None;
        }
    }
}

/// Removes the `"propertyNames"` constraint from the schema.
pub(crate) fn unconstrain_property_names(_: &str, node: &mut RefOr<Schema>) {
    let RefOr::T(Schema::Object(schema)) = node else {
        return;
    };
    schema.property_names = None;
}

/// Adds values the API also accepts to the `tag` property of the variants specified as such by
/// upstream.
///
/// `variants` holds the (`title`, `tag`, aliases) tuples of those variants.
pub(crate) fn alias_variants(
    variants: &[(&str, &str, &[&str])],
    _: &str,
    node: &mut RefOr<Schema>,
) {
    let RefOr::T(Schema::AllOf(all_of)) = node else {
        return;
    };
    let Some(title) = all_of.title.as_deref() else {
        return;
    };

    let Some((tag_name, aliases)) = variants
        .iter()
        .find(|(item, _, _)| *item == title)
        .map(|(_, tag_name, aliases)| (*tag_name, *aliases))
    else {
        return;
    };

    for item in all_of.items.iter_mut() {
        if let Some(object) = as_tag_mut(item, tag_name)
            && let Some(RefOr::T(Schema::Object(tag))) = object.properties.values_mut().next()
        {
            let aliases = aliases.iter().map(|s| Value::from(*s));
            tag.enum_values.get_or_insert_default().extend(aliases);
        }
    }
}

/// Narrows every variant tag of a tagged enumeration from a single-valued enumeration
/// to a `const`.
pub(crate) fn tags_to_const(schemas: &[(&str, &str)], path: &str, node: &mut RefOr<Schema>) {
    let RefOr::T(schema) = node else {
        return;
    };
    let Some(tag_name) = schema_name(path, schema)
        .and_then(|name| schemas.iter().find_map(|(n, t)| (n == &name).then_some(*t)))
    else {
        return;
    };

    match schema {
        // An enumeration with more than one variant, each an arm of the `oneOf`.
        Schema::OneOf(one_of) => {
            for variant in one_of.items.iter_mut() {
                tag_to_const(variant, tag_name);
            }
        }
        // An enumeration with a single variant, written as the variant itself.
        _ => tag_to_const(node, tag_name),
    }
}

/// Narrows the tag of a single variant, if `variant` carries one.
fn tag_to_const(variant: &mut RefOr<Schema>, tag_name: &str) {
    let RefOr::T(Schema::AllOf(all_of)) = variant else {
        return;
    };
    for item in all_of.items.iter_mut() {
        if let Some(object) = as_tag_mut(item, tag_name)
            && let Some(RefOr::T(Schema::Object(tag))) = object.properties.values_mut().next()
            && let [Value::String(value)] = tag.enum_values.as_deref().unwrap_or_default()
        {
            tag.extensions = Some(Extensions::from_iter([("const", value.clone())]));
            tag.schema_type = SchemaType::AnyValue;
            tag.enum_values = None;
        }
    }
}

/// Removes the `"type"` specification from the schemas specified as such by upstream.
///
/// `titles` holds the names of those schemas.
pub(crate) fn untype_schemas(titles: &[&str], path: &str, node: &mut RefOr<Schema>) {
    let RefOr::T(schema) = node else {
        return;
    };
    if schema_name(path, schema).is_some_and(|name| titles.contains(&name)) {
        set_schema_type(schema, SchemaType::AnyValue);
    }
}

/// Adds the `"type": "object"` specification to the schemas specified as such by upstream.
///
/// `titles` holds the names of those schemas.
pub(crate) fn object_schemas(titles: &[&str], path: &str, node: &mut RefOr<Schema>) {
    let RefOr::T(schema) = node else {
        return;
    };
    if schema_name(path, schema).is_some_and(|name| titles.contains(&name)) {
        set_schema_type(schema, SchemaType::Type(Type::Object));
    }
}

/// Removes the `title` from every schema, except for those in `keep`.
pub(crate) fn remove_title(keep: &[&str], _: &str, schema: &mut RefOr<Schema>) {
    let RefOr::T(schema) = schema else {
        return;
    };
    let Some(title) = get_title(schema) else {
        return;
    };
    if keep.contains(&title) {
        return;
    }
    match schema {
        Schema::Object(obj) => obj.title = None,
        Schema::Array(arr) => arr.title = None,
        Schema::AllOf(all_of) => all_of.title = None,
        Schema::OneOf(one_of) => one_of.title = None,
        _ => {}
    }
}

/// The tag of the variant `all_of` renders, when its `title` is one of `variants`' (`title`,
/// `tag`) tuples.
fn variant_tag<'a>(all_of: &AllOf, variants: &[(&str, &'a str)]) -> Option<&'a str> {
    let title = all_of.title.as_deref()?;
    variants
        .iter()
        .find(|(item, _)| *item == title)
        .map(|(_, tag)| *tag)
}

/// The name of the schema at `path`: the component's name when `path` addresses a whole
/// component, and its `title` otherwise.
fn schema_name<'a>(path: &'a str, schema: &'a Schema) -> Option<&'a str> {
    component_name(path).or(get_title(schema))
}

/// The name of the component `path` addresses, when it addresses a whole component rather
/// than a schema nested inside one.
fn component_name(path: &str) -> Option<&str> {
    path.strip_prefix(SCHEMA_PATH)
        .filter(|name| !name.contains('/'))
}

/// The object `utoipa` writes a variant's tag as, when `item` is one.
fn as_tag<'a>(item: &'a RefOr<Schema>, tag: &str) -> Option<&'a Object> {
    let RefOr::T(Schema::Object(object)) = item else {
        return None;
    };
    (object.properties.len() == 1 && object.properties.contains_key(tag)).then_some(object)
}

/// The object `utoipa` writes a variant's tag as, when `item` is one.
fn as_tag_mut<'a>(item: &'a mut RefOr<Schema>, tag: &str) -> Option<&'a mut Object> {
    let RefOr::T(Schema::Object(object)) = item else {
        return None;
    };
    (object.properties.len() == 1 && object.properties.contains_key(tag)).then_some(object)
}

/// Returns the title of `schema`, if it has one.
fn get_title(schema: &Schema) -> Option<&str> {
    match schema {
        Schema::Object(s) => s.title.as_deref(),
        Schema::Array(s) => s.title.as_deref(),
        Schema::AllOf(s) => s.title.as_deref(),
        Schema::OneOf(s) => s.title.as_deref(),
        _ => None,
    }
}

/// Sets the `type` of `schema` to `ty`.
pub(crate) fn set_schema_type(schema: &mut Schema, ty: SchemaType) {
    match schema {
        Schema::Object(s) => s.schema_type = ty,
        Schema::Array(s) => s.schema_type = ty,
        Schema::AllOf(s) => s.schema_type = ty,
        Schema::OneOf(s) => s.schema_type = ty,
        _ => {}
    }
}

/// Calls `visit` on every schema reachable from the document, outermost first.
pub(crate) fn visit_schemas(api: &mut OpenApi, visit: impl Fn(&str, &mut RefOr<Schema>)) {
    let visit = &visit;

    if let Some(components) = api.components.as_mut() {
        for (name, schema) in components.schemas.iter_mut() {
            let path = format!("{SCHEMA_PATH}{name}");
            visit_schema(&path, schema, visit);
        }
    }

    for (endpoint, item) in api.paths.paths.iter_mut() {
        let operations = [
            ("get", &mut item.get),
            ("put", &mut item.put),
            ("post", &mut item.post),
            ("delete", &mut item.delete),
            ("options", &mut item.options),
            ("head", &mut item.head),
            ("patch", &mut item.patch),
            ("trace", &mut item.trace),
        ];

        for parameter in item.parameters.iter_mut().flatten() {
            if let Some(schema) = parameter.schema.as_mut() {
                let path = format!("#/paths/{}/parameters/{}", endpoint, parameter.name);
                visit_schema(&path, schema, visit);
            }
        }

        for (name, operation) in operations.into_iter() {
            let Some(operation) = operation else {
                continue;
            };

            for parameter in operation.parameters.iter_mut().flatten() {
                if let Some(schema) = parameter.schema.as_mut() {
                    let path = format!("#/paths/{}/{}", endpoint, name);
                    visit_schema(&path, schema, visit);
                }
            }
            if let Some(body) = operation.request_body.as_mut() {
                for (type_, content) in body.content.iter_mut() {
                    let Some(schema) = content.schema.as_mut() else {
                        continue;
                    };
                    let path = format!("#/paths/{}/requestBody/content/{}", endpoint, type_);
                    visit_schema(&path, schema, visit);
                }
            }
            for (status_code, response) in operation.responses.responses.iter_mut() {
                let RefOr::T(response) = response else {
                    continue;
                };
                for content in response.content.values_mut() {
                    let Some(schema) = content.schema.as_mut() else {
                        continue;
                    };
                    let path = format!("#/paths/{}/responses/{}", endpoint, status_code);
                    visit_schema(&path, schema, visit);
                }
            }
        }
    }
}

/// Calls `visit` on `node` and then on every schema nested inside it.
fn visit_schema(path: &str, node: &mut RefOr<Schema>, visit: &impl Fn(&str, &mut RefOr<Schema>)) {
    visit(path, node);

    let RefOr::T(schema) = node else {
        return;
    };
    visit_subschema(path, schema, visit);
}

fn visit_subschema(path: &str, node: &mut Schema, visit: &impl Fn(&str, &mut RefOr<Schema>)) {
    match node {
        Schema::Object(object) => {
            for (field, property) in object.properties.iter_mut() {
                let path = format!("{path}/{field}");
                visit_schema(&path, property, visit);
            }
            if let Some(AdditionalProperties::RefOr(additional)) =
                object.additional_properties.as_deref_mut()
            {
                let path = format!("{path}/additionalProperties");
                visit_schema(&path, additional, visit);
            }
        }
        Schema::Array(array) => {
            if let ArrayItems::RefOrSchema(items) = &mut array.items {
                let path = format!("{path}/items");
                visit_schema(&path, items, visit);
            }
        }
        Schema::OneOf(one_of) => {
            for (idx, arm) in one_of.items.iter_mut().enumerate() {
                let path = format!("{path}/oneOf[subschema #{idx}]");
                visit_schema(&path, arm, visit);
            }
        }
        Schema::AllOf(all_of) => {
            for (idx, arm) in all_of.items.iter_mut().enumerate() {
                let path = format!("{path}/allOf[subschema #{idx}]");
                visit_schema(&path, arm, visit);
            }
        }
        Schema::AnyOf(any_of) => {
            for (idx, arm) in any_of.items.iter_mut().enumerate() {
                let path = format!("{path}/anyOf[subschema #{idx}]");
                visit_schema(&path, arm, visit);
            }
        }
        _ => {}
    }
}
