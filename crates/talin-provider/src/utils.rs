/// A macro to create an enum with a single variant for fixed name serialization/deserialization.
macro_rules! fixed_value {
    ($name:ident:: $variant:ident) => {
        /// A fixed discriminant.
        ///
        /// This type has a single variant, and always round-trips to the same value.
        #[cfg_attr(test, derive(utoipa::ToSchema))]
        #[derive(Debug, Clone, Default, Copy, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            /// The only value of this type.
            #[default]
            $variant,
        }
    };

    ($name:ident:: $variant:ident => $value:literal) => {
        /// A fixed discriminant.
        ///
        /// This type has a single variant, and always round-trips to the same value.
        #[cfg_attr(test, derive(utoipa::ToSchema))]
        #[derive(Debug, Clone, Default, Copy, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            /// The only value of this type.
            #[default]
            #[serde(rename = $value)]
            $variant,
        }
    };
}
pub(crate) use fixed_value;

/// A macro to create a string enum that accepts values it does not list.
///
/// Besides the listed variants, the enum gets an `Unknown(String)` variant that keeps any other
/// value as sent, for forward compatibility.
///
/// In tests, the enum describes itself as a closed enumeration of the listed variants, as the API
/// documents it, so the generated OpenAPI description has no catch-all. The schema comes from a
/// `Known` copy of the enum that lives in an anonymous scope, and the enum delegates to it, so
/// fields of the enum need no `schema(value_type = ...)` override.
///
/// The enum derives `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, and `Eq`.
/// Doc comments and `#[serde(...)]` attributes are accepted on the enum, and doc comments and
/// `#[serde(rename = ...)]` on its variants.
macro_rules! string_enum {
    (
        $(#[doc = $doc:literal])*
        $(#[serde($($serde:tt)*)])*
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $vdoc:literal])*
                $(#[serde(rename = $rename:tt)])?
                $variant:ident
            ),+ $(,)?
        }
    ) => {
        $(#[doc = $doc])*
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        #[derive(Debug, Clone, PartialEq, Eq)]
        $(#[serde($($serde)*)])*
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[doc = $vdoc])*
                $(#[serde(rename = $rename)])?
                $variant,
            )+

            /// A value not represented by this enum, kept as sent for forward compatibility.
            #[serde(untagged)]
            Unknown(String),
        }

        #[cfg(test)]
        const _: () = {
            $(#[doc = $doc])*
            #[allow(dead_code)]
            #[derive(utoipa::ToSchema)]
            #[schema(as = $name)]
            $(#[serde($($serde)*)])*
            enum Known {
                $(
                    $(#[doc = $vdoc])*
                    $(#[serde(rename = $rename)])?
                    $variant,
                )+
            }

            $crate::utils::delegate_schema!($name => Known);
        };
    };
}
pub(crate) use string_enum;

/// A macro to create an internally tagged enum that accepts variants it does not list.
///
/// Besides the listed variants, the enum gets an `Unknown(serde_json::Value)` variant that keeps
/// any other value as sent, for forward compatibility.
///
/// In tests, the enum describes itself as a closed union of the listed variants, as the API
/// documents it, so the generated OpenAPI description has no catch-all. See [`string_enum!`] for
/// how that is done.
///
/// Attributes on the enum and on its variants are applied to both the enum and its closed copy,
/// so they must not be test-only. Attributes that apply to the description alone go after a `@`
/// on a variant, as `@schema(title = "Foo")`, and are applied to the closed copy as
/// `#[schema(title = "Foo")]`. A variant is a unit, a tuple of one field, or a struct.
macro_rules! tagged_enum {
    (
        $(#[$($meta:tt)*])*
        $vis:vis enum $name:ident {
            $(
                $(#[$($vmeta:tt)*])*
                $(@schema($($vschema:tt)*))?
                $variant:ident
                $(($($tuple:tt)*))?
                $({$($fields:tt)*})?
            ),+ $(,)?
        }
    ) => {
        $(#[$($meta)*])*
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$($vmeta)*])*
                $variant
                $(($($tuple)*))?
                $({$($fields)*})?,
            )+

            /// A value not represented by this enum, kept as sent for forward compatibility.
            #[serde(untagged)]
            Unknown(serde_json::Value),
        }

        #[cfg(test)]
        const _: () = {
            $(#[$($meta)*])*
            #[allow(dead_code)]
            #[derive(utoipa::ToSchema)]
            #[schema(as = $name)]
            #[allow(clippy::large_enum_variant)]
            enum Known {
                $(
                    $(#[$($vmeta)*])*
                    $(#[schema($($vschema)*)])?
                    $variant
                    $(($($tuple)*))?
                    $({$($fields)*})?,
                )+
            }

            $crate::utils::delegate_schema!($name => Known);
        };
    };
}
pub(crate) use tagged_enum;

/// A macro to make a type describe itself as another type in the generated OpenAPI description.
///
/// `$known` must derive `utoipa::ToSchema` with `#[schema(as = $name)]`. The macro is for tests,
/// where the description is generated, and is used by [`string_enum!`] and [`tagged_enum!`]. Use
/// it directly for an open enum those cannot express, such as one whose variants have fields with
/// test-only `schema` attributes.
#[cfg(test)]
macro_rules! delegate_schema {
    ($name:ident => $known:ident) => {
        impl utoipa::PartialSchema for $name {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                <$known as utoipa::PartialSchema>::schema()
            }
        }

        impl utoipa::ToSchema for $name {
            fn name() -> ::std::borrow::Cow<'static, str> {
                <$known as utoipa::ToSchema>::name()
            }

            fn schemas(
                schemas: &mut Vec<(
                    String,
                    utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
                )>,
            ) {
                <$known as utoipa::ToSchema>::schemas(schemas)
            }
        }
    };
}
#[cfg(test)]
pub(crate) use delegate_schema;
