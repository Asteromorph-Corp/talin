use std::str::FromStr;

use serde::de::Visitor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(crate) enum InlineOrCustom<T> {
    Inline(T),
    Custom,
}

impl<T> FromStr for InlineOrCustom<T> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "custom" {
            Ok(InlineOrCustom::Custom)
        } else {
            Err(format!(
                r#"Only "custom" is allowed as a string value for config, found: {s}"#
            ))
        }
    }
}

impl<'de, T> Deserialize<'de> for InlineOrCustom<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(InlineOrCustomVisitor(std::marker::PhantomData))
    }
}

struct InlineOrCustomVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T> Visitor<'de> for InlineOrCustomVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = InlineOrCustom<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an inline value or the string 'custom'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(E::custom)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let t = T::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))?;
        Ok(InlineOrCustom::Inline(t))
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        let t = T::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
        Ok(InlineOrCustom::Inline(t))
    }
}
