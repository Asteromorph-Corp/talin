use std::collections::BTreeMap;

use serde::de::value::MapAccessDeserializer;
use serde::de::{Error as DeError, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};

use crate::error::ToolInitParamsError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub(crate) struct AgentLocalTool {
    pub name: String,
    pub config: Option<AgentLocalToolConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum AgentLocalToolConfig {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Env(String),
    Array(Vec<AgentLocalToolConfig>),
    EnumVariant {
        enum_: String,
        variant: String,
        fields: Option<AgentLocalToolConfigNested>,
    },
    Nested(AgentLocalToolConfigNested),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum AgentLocalToolConfigNested {
    Tuple(Vec<Option<AgentLocalToolConfig>>),
    Struct(BTreeMap<String, AgentLocalToolConfig>),
}

impl From<String> for AgentLocalTool {
    fn from(name: String) -> Self {
        AgentLocalTool { name, config: None }
    }
}

impl Serialize for AgentLocalTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Self::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for AgentLocalTool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AgentLocalToolVisitor)
    }
}

struct AgentLocalToolVisitor;

impl<'de> Visitor<'de> for AgentLocalToolVisitor {
    type Value = AgentLocalTool;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(r#"either "tool_name" or { name = "...", init = { ... } }"#)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(AgentLocalTool::from(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(AgentLocalTool::from(v))
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        AgentLocalTool::deserialize(MapAccessDeserializer::new(map))
    }
}

impl<'de> Deserialize<'de> for AgentLocalToolConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AgentLocalToolConfigVisitor)
    }
}

struct AgentLocalToolConfigVisitor;

impl<'de> Visitor<'de> for AgentLocalToolConfigVisitor {
    type Value = AgentLocalToolConfig;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a scalar, array, or table")
    }

    fn visit_bool<E: DeError>(self, v: bool) -> Result<Self::Value, E> {
        Ok(AgentLocalToolConfig::Boolean(v))
    }

    fn visit_i64<E: DeError>(self, v: i64) -> Result<Self::Value, E> {
        Ok(AgentLocalToolConfig::Integer(v))
    }

    fn visit_u64<E: DeError>(self, v: u64) -> Result<Self::Value, E> {
        i64::try_from(v)
            .map(AgentLocalToolConfig::Integer)
            .map_err(|_| E::custom(format!("integer {v} out of range for i64")))
    }

    fn visit_f64<E: DeError>(self, v: f64) -> Result<Self::Value, E> {
        Ok(AgentLocalToolConfig::Float(v))
    }

    fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
        Ok(AgentLocalToolConfig::String(v.to_owned()))
    }

    fn visit_string<E: DeError>(self, v: String) -> Result<Self::Value, E> {
        Ok(AgentLocalToolConfig::String(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = Vec::new();
        while let Some(elem) = seq.next_element::<AgentLocalToolConfig>()? {
            items.push(elem);
        }
        Ok(AgentLocalToolConfig::Array(items))
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        let mut entries = BTreeMap::<String, AgentLocalToolConfig>::new();
        while let Some((key, value)) = map.next_entry::<String, AgentLocalToolConfig>()? {
            entries.insert(key, value);
        }
        classify_table::<M::Error>(entries)
    }
}

/// Classifies a deserialized table into the appropriate [`AgentLocalToolConfig`] variant
/// using the same rules as `tool_params::visit_table`:
/// - `{ env = "VAR" }` with exactly one key maps to [`AgentLocalToolConfig::Env`].
/// - A table with both `$enum` and `$variant` maps to [`AgentLocalToolConfig::EnumVariant`].
/// - Otherwise the table is nested as either a tuple (keys `$0`, `$1`, …) or a struct.
fn classify_table<E: DeError>(
    mut entries: BTreeMap<String, AgentLocalToolConfig>,
) -> Result<AgentLocalToolConfig, E> {
    if entries.len() == 1
        && let Some(AgentLocalToolConfig::String(env)) = entries.remove("env")
    {
        return Ok(AgentLocalToolConfig::Env(env));
    }

    match (entries.remove("$enum"), entries.remove("$variant")) {
        (Some(enum_val), Some(variant_val)) => {
            let AgentLocalToolConfig::String(enum_) = enum_val else {
                return Err(E::custom(ToolInitParamsError::NonStringMetadataValue(
                    "$enum",
                )));
            };
            let AgentLocalToolConfig::String(variant) = variant_val else {
                return Err(E::custom(ToolInitParamsError::NonStringMetadataValue(
                    "$variant",
                )));
            };
            let fields = if entries.is_empty() {
                None
            } else {
                Some(nested_from_entries::<E>(entries)?)
            };
            Ok(AgentLocalToolConfig::EnumVariant {
                enum_,
                variant,
                fields,
            })
        }
        (Some(_), None) | (None, Some(_)) => {
            Err(E::custom(ToolInitParamsError::MissingEnumOrVariant))
        }
        (None, None) => Ok(AgentLocalToolConfig::Nested(nested_from_entries::<E>(
            entries,
        )?)),
    }
}

fn nested_from_entries<E: DeError>(
    mut entries: BTreeMap<String, AgentLocalToolConfig>,
) -> Result<AgentLocalToolConfigNested, E> {
    let has_tuple_idx = entries.keys().any(|k| {
        k.strip_prefix('$')
            .and_then(|s| s.parse::<usize>().ok())
            .is_some()
    });
    let has_regular_field = entries.keys().any(|k| !k.starts_with('$'));

    match (has_tuple_idx, has_regular_field) {
        (false, _) => Ok(AgentLocalToolConfigNested::Struct(entries)),
        (true, false) => {
            let max_idx = entries
                .keys()
                .filter_map(|k| k.strip_prefix('$').and_then(|s| s.parse::<usize>().ok()))
                .max()
                .unwrap_or(0);

            let slots = (0..=max_idx)
                .map(|idx| {
                    let key = format!("${idx}");
                    entries.remove(&key)
                })
                .collect();
            Ok(AgentLocalToolConfigNested::Tuple(slots))
        }
        (true, true) => Err(E::custom(ToolInitParamsError::MixedFields)),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::string(
        serde_json::json!("some_tool"),
        AgentLocalTool {
            name: "some_tool".to_string(),
            config: None,
        }
    )]
    #[case::map(
        serde_json::json!({
            "name": "some_tool",
            "config": {
                "param1": "value1",
                "param2": 42
            }
        }),
        AgentLocalTool {
            name: "some_tool".to_string(),
            config: Some(AgentLocalToolConfig::Nested(
                AgentLocalToolConfigNested::Struct(BTreeMap::from([
                    ("param1".to_string(), AgentLocalToolConfig::String("value1".to_string())),
                    ("param2".to_string(), AgentLocalToolConfig::Integer(42)),
                ])),
            )),
        }
    )]
    fn deserializes(#[case] value: serde_json::Value, #[case] expected: AgentLocalTool) {
        let tool: AgentLocalTool = serde_json::from_value(value).unwrap();
        assert_eq!(tool, expected);
    }

    #[rstest]
    #[case::string(
        serde_json::json!("value"),
        AgentLocalToolConfig::String("value".to_string())
    )]
    #[case::integer(
        serde_json::json!(42),
        AgentLocalToolConfig::Integer(42)
    )]
    #[case::float(
        serde_json::json!(3.1234),
        AgentLocalToolConfig::Float(3.1234)
    )]
    #[case::boolean(
        serde_json::json!(true),
        AgentLocalToolConfig::Boolean(true)
    )]
    #[case::env(
        serde_json::json!({ "env": "MY_ENV_VAR" }),
        AgentLocalToolConfig::Env("MY_ENV_VAR".to_string())
    )]
    #[case::array(
        serde_json::json!([1, 2, 3]),
        AgentLocalToolConfig::Array(vec![
            AgentLocalToolConfig::Integer(1),
            AgentLocalToolConfig::Integer(2),
            AgentLocalToolConfig::Integer(3),
        ])
    )]
    #[case::enum_variant(
        serde_json::json!({
            "$enum": "MyEnum",
            "$variant": "MyVariant",
            "field1": "value1",
            "field2": 42
        }),
        AgentLocalToolConfig::EnumVariant {
            enum_: "MyEnum".to_string(),
            variant: "MyVariant".to_string(),
            fields: Some(AgentLocalToolConfigNested::Struct(BTreeMap::from([
                ("field1".to_string(), AgentLocalToolConfig::String("value1".to_string())),
                ("field2".to_string(), AgentLocalToolConfig::Integer(42)),
            ]))),
        }
    )]
    #[case::nested_struct(
        serde_json::json!({
            "field1": "value1",
            "field2": 42
        }),
        AgentLocalToolConfig::Nested(AgentLocalToolConfigNested::Struct(BTreeMap::from([
            ("field1".to_string(), AgentLocalToolConfig::String("value1".to_string())),
            ("field2".to_string(), AgentLocalToolConfig::Integer(42)),
        ]))),
    )]
    fn deserialize_config(
        #[case] value: serde_json::Value,
        #[case] expected: AgentLocalToolConfig,
    ) {
        let config: AgentLocalToolConfig = serde_json::from_value(value).unwrap();
        assert_eq!(config, expected);
    }
}
