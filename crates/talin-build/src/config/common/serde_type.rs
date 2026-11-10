use quote::ToTokens;
use serde::Deserialize;

/// A serde helper for `syn::Type`.
#[repr(transparent)]
pub(crate) struct SerdeType;

impl SerdeType {
    pub(crate) fn serialize<S>(ty: &syn::Type, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = ty.to_token_stream().to_string().replace(" ", "");
        serializer.serialize_str(&s)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<syn::Type, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        syn::parse_str(&s).map_err(serde::de::Error::custom)
    }
}
