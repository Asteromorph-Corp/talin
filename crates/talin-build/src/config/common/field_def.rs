use serde::de::Visitor;
use serde::{Deserialize, Serialize};

use crate::config::SerdeType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, remote = "Self")]
pub(crate) struct FieldDef {
    #[serde(rename = "type", with = "SerdeType")]
    pub type_: syn::Type,
    pub description: Option<String>,
}

impl Serialize for FieldDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Self::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for FieldDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(FieldConfigVisitor)
    }
}

struct FieldConfigVisitor;

impl<'de> Visitor<'de> for FieldConfigVisitor {
    type Value = FieldDef;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string or a map")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let type_: syn::Type = syn::parse_str(v).map_err(E::custom)?;

        Ok(FieldDef {
            type_,
            description: None,
        })
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        FieldDef::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use syn::parse_quote;

    use super::*;

    #[rstest]
    #[case(serde_json::json!("u8"), FieldDef { type_: parse_quote! { u8 }, description: None })]
    #[case(
        serde_json::json!({"type": "String", "description": "A string field"}),
        FieldDef { type_: parse_quote! { String }, description: Some("A string field".to_string()) }
    )]
    fn deserializes(#[case] value: serde_json::Value, #[case] expected: FieldDef) {
        let def: FieldDef = serde_json::from_value(value).unwrap();
        assert_eq!(def, expected);
    }
}
