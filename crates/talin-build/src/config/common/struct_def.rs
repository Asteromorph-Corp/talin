use std::collections::BTreeMap;

use serde::de::Visitor;
use serde::de::value::MapAccessDeserializer;
use serde::{Deserialize, Serialize};

use crate::config::FieldDef;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[repr(transparent)]
pub(crate) struct StructDef(pub BTreeMap<String, FieldDef>);

#[derive(Deserialize)]
struct LegacyFieldDef {
    name: String,
    #[serde(flatten)]
    def: FieldDef,
}

impl<'de> Deserialize<'de> for StructDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(StructDefVisitor)
    }
}

struct StructDefVisitor;

impl<'de> Visitor<'de> for StructDefVisitor {
    type Value = StructDef;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(r#"sequence or a map"#)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut map = BTreeMap::new();
        while let Some(entry) = seq.next_element::<LegacyFieldDef>()? {
            map.insert(entry.name, entry.def);
        }
        Ok(StructDef(map))
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        let map = BTreeMap::<String, FieldDef>::deserialize(MapAccessDeserializer::new(map))?;
        Ok(StructDef(map))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use syn::parse_quote;

    use super::*;

    #[rstest]
    #[case::seq(
        serde_json::json!([
            { "name": "field1", "type": "String", "description": "A string field" },
            { "name": "field2", "type": "i32" }
        ]),
        StructDef(BTreeMap::from_iter([
            ("field1".to_string(), FieldDef { type_: parse_quote! { String }, description: Some("A string field".to_string()) }),
            ("field2".to_string(), FieldDef { type_: parse_quote! { i32 }, description: None })
        ]))
    )]
    #[case::map(
        serde_json::json!({
            "field1": { "type": "String", "description": "A string field" },
            "field2": { "type": "i32" }
        }),
        StructDef(BTreeMap::from_iter([
            ("field1".to_string(), FieldDef { type_: parse_quote! { String }, description: Some("A string field".to_string()) }),
            ("field2".to_string(), FieldDef { type_: parse_quote! { i32 }, description: None })
        ]))
    )]
    fn deserializes(#[case] value: serde_json::Value, #[case] expected: StructDef) {
        let struct_def: StructDef = serde_json::from_value(value).unwrap();
        assert_eq!(struct_def, expected);
    }
}
