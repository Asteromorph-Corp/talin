use std::collections::{BTreeMap, HashSet};

use indoc::formatdoc;
use regex::Regex;
use schemars::Schema;
use serde_json::Value;

use crate::text::render::add_indent;

/// Generates a human-readable description of the given JSON schema.
pub(crate) fn render_schema(parameters: &Schema) -> Result<String, String> {
    let definitions = collect_definitions(parameters)?;
    render_schema_root(parameters, true, &definitions, 0)
}

fn render_schema_root(
    schema: &Schema,
    required: bool,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    if depth > 10 {
        return Ok("object // Too deep".into());
    }

    if let Some(b) = schema.as_bool() {
        return Ok(if b { "any".into() } else { "never".into() });
    }

    let Some(obj) = schema.as_object() else {
        return Err("schema is not an object or a boolean".into());
    };

    if obj.is_empty() {
        return Ok("{} // an empty object".into());
    }

    if let Some(Value::String(reference)) = obj.get("$ref") {
        return resolve_reference(reference, required, definitions, depth);
    }

    let type_ = match obj.get("type") {
        Some(Value::String(t)) => t.as_str(),
        Some(Value::Array(arr)) => arr
            .iter()
            .find_map(|v| {
                let ty = v.as_str()?;
                (ty != "null").then_some(ty)
            })
            .unwrap_or("null"),
        other => {
            return Err(format!(
                "invalid value for `type`: expected string or array, got: {other:?}"
            ));
        }
    };

    let description = obj.get("description").and_then(|v| v.as_str());
    let enum_values = obj
        .get("enum")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>());
    let comment = generate_comment(description, enum_values.as_deref(), required);

    let full_description = match type_ {
        "null" => "{} // A null value".to_string(),
        "boolean" => format!("bool{comment}"),
        "number" => format!("number{comment}"),
        "integer" => format!("integer{comment}"),
        "string" => format!("string{comment}"),
        "object" => render_object_schema(obj, &comment, definitions, depth)?,
        "array" => render_array_schema(obj, &comment, definitions, depth)?,
        other => return Err(format!("unsupported type: {other}")),
    }
    .trim()
    .to_string();

    Ok(full_description)
}

fn render_object_schema(
    obj: &serde_json::Map<String, Value>,
    comment: &str,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    if obj.get("patternProperties").is_some() {
        log::warn!("Pattern properties are not supported, they will be ignored");
    }

    let required: HashSet<&str> = obj
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let Some(properties) = obj.get("properties").and_then(|v| v.as_object()) else {
        return Ok(format!("{{}}{comment}"));
    };

    let properties = properties
        .iter()
        .map(|(name, schema)| {
            let is_required = required.contains(name.as_str());
            let subschema = schema
                .try_into()
                .map_err(|e| format!("Invalid schema for property {name}: {e}"))?;
            let description = render_schema_root(subschema, is_required, definitions, depth + 1)?;
            Ok(format!("\"{name}\": {description}"))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join("\n");

    Ok(formatdoc! {"
        {{{comment}
        {}
        }}",
        add_indent(&properties, 4, true)
    })
}

fn render_array_schema(
    obj: &serde_json::Map<String, Value>,
    comment: &str,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    let Some(items) = obj.get("items") else {
        return Ok(format!("[]{comment}"));
    };

    let items_schema = match items {
        Value::Array(arr) => {
            log::warn!("Union types for array items are not supported, using the first one");
            if let Some(first) = arr.first() {
                first
            } else {
                return Ok(format!("[]{comment}"));
            }
        }
        other => other,
    };
    let items_schema = items_schema
        .try_into()
        .map_err(|e| format!("Invalid schema for array items: {e}"))?;

    let item_description = format!(
        "{} // ...",
        render_schema_root(items_schema, true, definitions, depth + 1)?
    );

    Ok(formatdoc! {"
        [{comment}
        {}
        ]",
        add_indent(&item_description, 4, true)
    })
}

fn collect_definitions(root: &Schema) -> Result<BTreeMap<&str, &Schema>, String> {
    ["definitions", "$defs"]
        .iter()
        .filter_map(|key| root.get(*key).and_then(|v| v.as_object()))
        .flat_map(|defs| defs.iter())
        .map(|(k, v)| {
            let def = v
                .try_into()
                .map_err(|e| format!("Failed to parse inner definition for {k}: {e}"))?;
            Ok((k.as_str(), def))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()
}

fn resolve_reference(
    reference: &str,
    required: bool,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    if reference == "#" {
        return Ok("object // Same as the root object".into());
    }

    let re = Regex::new(r"^#\/(?:\$defs|definitions)\/(.+)$").unwrap();

    let Some(captures) = re.captures(reference) else {
        return Err(format!("invalid reference {reference}"));
    };

    let definition_name = captures.get(1).ok_or("invalid reference")?.as_str();

    let Some(definition) = definitions.get(definition_name) else {
        return Err(format!("definition {definition_name} not found"));
    };

    render_schema_root(definition, required, definitions, depth)
}

fn generate_comment(
    description: Option<&str>,
    enum_values: Option<&[&str]>,
    required: bool,
) -> String {
    let enum_comment =
        enum_values.map(|values| format!("should be one of: [{}]", values.join(", ")));
    let optional = if required { None } else { Some("(optional)") };

    match (description, enum_comment, optional) {
        (Some(desc), Some(enum_desc), Some(opt)) => format!(" // {desc}, {enum_desc} {opt}"),
        (Some(desc), Some(enum_desc), None) => format!(" // {desc}, {enum_desc}"),
        (Some(desc), None, Some(opt)) => format!(" // {desc} {opt}"),
        (None, Some(enum_desc), Some(opt)) => format!(" // {enum_desc} {opt}"),
        (Some(desc), None, None) => format!(" // {desc}"),
        (None, Some(enum_desc), None) => format!(" // {enum_desc}"),
        (None, None, Some(opt)) => format!(" // {opt}"),
        (None, None, None) => String::new(),
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use indoc::indoc;
    use schemars::{JsonSchema, schema_for};

    use super::*;
    use crate::tools::DefaultFunctionInput;

    #[test]
    fn schema_renders() {
        let schema = schema_for!(DefaultFunctionInput);
        let description = render_schema(&schema).unwrap();

        assert_eq!(
            description,
            indoc! {"
                { // The input for the tool.
                    \"input\": string // The input string.
                }"
            }
        );
    }

    #[test]
    fn object_schema_renders() {
        #[derive(JsonSchema)]
        #[serde(deny_unknown_fields)]
        #[schemars(description = "The input for the tool")]
        struct TestObject {
            #[schemars(description = "The name of the person")]
            name: String,
            #[schemars(description = "The age of the person")]
            age: u32,
        }

        let schema = schema_for!(TestObject);
        let description = render_schema(&schema).unwrap();

        assert_eq!(
            description,
            indoc! {"
                { // The input for the tool
                    \"name\": string // The name of the person
                    \"age\": integer // The age of the person
                }"}
        );
    }

    #[test]
    fn array_schema_renders() {
        #[derive(JsonSchema)]
        #[serde(deny_unknown_fields)]
        #[schemars(description = "The list of numbers")]
        struct TestArray(Vec<u32>);

        let schema = schema_for!(TestArray);
        let description = render_schema(&schema).unwrap();

        assert_eq!(
            description,
            indoc! {"
                [ // The list of numbers
                    integer // ...
                ]"}
        );
    }

    #[test]
    fn complex_schema_renders() {
        #[derive(JsonSchema)]
        #[serde(deny_unknown_fields)]
        #[schemars(description = "The input for the tool")]
        struct ComplexInput {
            #[schemars(description = "The name of the person")]
            name: String,
            #[schemars(description = "The age of the person")]
            age: u32,
            #[schemars(description = "The list of phone numbers")]
            numbers: Vec<PhoneNumber>,
        }

        #[derive(JsonSchema)]
        #[serde(deny_unknown_fields)]
        #[schemars(description = "A phone number")]
        struct PhoneNumber {
            #[schemars(description = "The phone number")]
            number: String,
            #[schemars(description = "The type of the phone number")]
            #[serde(rename = "type")]
            type_: String,
        }

        let schema = schema_for!(ComplexInput);
        let description = render_schema(&schema).unwrap();

        assert_eq!(
            description,
            indoc! {"
                { // The input for the tool
                    \"name\": string // The name of the person
                    \"age\": integer // The age of the person
                    \"numbers\": [ // The list of phone numbers
                        { // A phone number
                            \"number\": string // The phone number
                            \"type\": string // The type of the phone number
                        } // ...
                    ]
                }"}
        );
    }
}
