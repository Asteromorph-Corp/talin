//! Declaring a payload together with the request-side twin upstream describes beside it.
//!
//! Upstream states each server tool result payload twice, once as it is returned and once as it
//! is passed back. The two carry the same information today, but they are separate schemas, so
//! upstream can add a field to one of them without touching the other. Aliasing them together
//! here would make that change impossible to express, so both are declared — from one source, to
//! keep the duplication from being written out by hand.

/// Declares a payload and its request-side twin.
///
/// The two differ only in how they treat an absent value: a response states every field, writing
/// `null` when it has nothing to say, so all of them are required; a request leaves the field out
/// instead. That is the whole of it, so each field is written once.
///
/// A field whose own type has a request-side twin names both, `response | request`. `Vec<T>` is
/// understood, and converts element by element; every other pair converts through [`From`].
///
/// ```ignore
/// paired_block! {
///     /// A successful bash code execution result block.
///     BashCodeExecutionResultBlock | BashCodeExecutionResultBlockParam {
///         /// The files the executed bash code generated.
///         content: Vec<BashCodeExecutionResultBlockContentItem>
///                | Vec<BashCodeExecutionResultBlockContentItemParam>,
///
///         /// The exit code of the executed bash code.
///         return_code: isize,
///     }
/// }
/// ```
macro_rules! paired_block {
    (
        $(#[$meta:meta])*
        $block:ident | $param:ident { $($fields:tt)* }
    ) => {
        // The trailing comma makes the last field look like every other one; a lone one left
        // over is swallowed below.
        paired_block! {
            // `value` is threaded rather than minted in the rule that finally binds it: an
            // identifier is only the same identifier as the one from the expansion it came from.
            @munch [$(#[$meta])*] [$block] [$param] [value] [] [] [] [] [] [] [] [] [] [$($fields)* ,]
        }
    };

    // A field of a `Vec` of paired types.
    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*]
        [
            $(#[$fmeta:meta])* $field:ident: Vec<$ty:ty> | Vec<$pty:ty>,
            $($rest:tt)*
        ]
    ) => {
        paired_block! {
            @munch [$($meta)*] [$block] [$param] [$value]
            [
                $($bfield)*
                $(#[$fmeta])*
                #[cfg_attr(test, schema(required = true))]
                pub $field: Vec<$ty>,
            ]
            [ $($pfield)* $(#[$fmeta])* pub $field: Vec<$pty>, ]
            [ $($convert)* $field: $value.$field.into_iter().map(Into::into).collect(), ]
            [ $($barg)* $field: Vec<$ty>, ]
            [ $($binit)* $field, ]
            [ $($bsetter)* ]
            [ $($arg)* $field: Vec<$pty>, ]
            [ $($init)* $field, ]
            [ $($setter)* ]
            [$($rest)*]
        }
    };

    // A field of a paired type.
    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*]
        [
            $(#[$fmeta:meta])* $field:ident: $ty:ty | $pty:ty,
            $($rest:tt)*
        ]
    ) => {
        paired_block! {
            @munch [$($meta)*] [$block] [$param] [$value]
            [
                $($bfield)*
                $(#[$fmeta])*
                #[cfg_attr(test, schema(required = true))]
                pub $field: $ty,
            ]
            [ $($pfield)* $(#[$fmeta])* pub $field: $pty, ]
            [ $($convert)* $field: $value.$field.into(), ]
            [ $($barg)* $field: $ty, ]
            [ $($binit)* $field, ]
            [ $($bsetter)* ]
            [ $($arg)* $field: $pty, ]
            [ $($init)* $field, ]
            [ $($setter)* ]
            [$($rest)*]
        }
    };

    // A `String` a request may leave out, which its setter takes anything convertible into.
    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*]
        [
            $(#[$fmeta:meta])* $field:ident: Option<String> with $with:ident,
            $($rest:tt)*
        ]
    ) => {
        paired_block! {
            @munch [$($meta)*] [$block] [$param] [$value]
            [
                $($bfield)*
                $(#[$fmeta])*
                #[cfg_attr(test, schema(required = true))]
                pub $field: Option<String>,
            ]
            [ $($pfield)* $(#[$fmeta])* pub $field: Option<String>, ]
            [ $($convert)* $field: $value.$field, ]
            [ $($barg)* ]
            [ $($binit)* $field: None, ]
            [
                $($bsetter)*
                #[doc = concat!(
                    "Sets the [`", stringify!($field), "`](Self::", stringify!($field), ").",
                )]
                pub fn $with(mut self, $field: impl Into<String>) -> Self {
                    self.$field = Some($field.into());
                    self
                }
            ]
            [ $($arg)* ]
            [ $($init)* $field: None, ]
            [
                $($setter)*
                #[doc = concat!(
                    "Sets the [`", stringify!($field), "`](Self::", stringify!($field), ").",
                )]
                pub fn $with(mut self, $field: impl Into<String>) -> Self {
                    self.$field = Some($field.into());
                    self
                }
            ]
            [$($rest)*]
        }
    };

    // A field a request may leave out, which the constructor takes a setter for instead.
    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*]
        [
            $(#[$fmeta:meta])* $field:ident: Option<$ty:ty> with $with:ident,
            $($rest:tt)*
        ]
    ) => {
        paired_block! {
            @munch [$($meta)*] [$block] [$param] [$value]
            [
                $($bfield)*
                $(#[$fmeta])*
                #[cfg_attr(test, schema(required = true))]
                pub $field: Option<$ty>,
            ]
            [ $($pfield)* $(#[$fmeta])* pub $field: Option<$ty>, ]
            [ $($convert)* $field: $value.$field, ]
            [ $($barg)* ]
            [ $($binit)* $field: None, ]
            [
                $($bsetter)*
                #[doc = concat!(
                    "Sets the [`", stringify!($field), "`](Self::", stringify!($field), ").",
                )]
                pub fn $with(mut self, $field: $ty) -> Self {
                    self.$field = Some($field);
                    self
                }
            ]
            [ $($arg)* ]
            [ $($init)* $field: None, ]
            [
                $($setter)*
                #[doc = concat!(
                    "Sets the [`", stringify!($field), "`](Self::", stringify!($field), ").",
                )]
                pub fn $with(mut self, $field: $ty) -> Self {
                    self.$field = Some($field);
                    self
                }
            ]
            [$($rest)*]
        }
    };

    // The fixed discriminant of a block, which a caller has no say over.
    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*]
        [
            $(#[$fmeta:meta])* type_: $ty:ty,
            $($rest:tt)*
        ]
    ) => {
        paired_block! {
            @munch [$($meta)*] [$block] [$param] [$value]
            [
                $($bfield)*
                $(#[$fmeta])*
                #[cfg_attr(test, schema(required = true))]
                pub type_: $ty,
            ]
            [ $($pfield)* $(#[$fmeta])* pub type_: $ty, ]
            [ $($convert)* type_: $value.type_, ]
            [ $($barg)* ]
            [ $($binit)* type_: <$ty>::default(), ]
            [ $($bsetter)* ]
            [ $($arg)* ]
            [ $($init)* type_: <$ty>::default(), ]
            [ $($setter)* ]
            [$($rest)*]
        }
    };

    // A `String` both sides share, which the constructor takes anything convertible into.
    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*]
        [
            $(#[$fmeta:meta])* $field:ident: String,
            $($rest:tt)*
        ]
    ) => {
        paired_block! {
            @munch [$($meta)*] [$block] [$param] [$value]
            [
                $($bfield)*
                $(#[$fmeta])*
                #[cfg_attr(test, schema(required = true))]
                pub $field: String,
            ]
            [ $($pfield)* $(#[$fmeta])* pub $field: String, ]
            [ $($convert)* $field: $value.$field, ]
            [ $($barg)* $field: impl Into<String>, ]
            [ $($binit)* $field: $field.into(), ]
            [ $($bsetter)* ]
            [ $($arg)* $field: impl Into<String>, ]
            [ $($init)* $field: $field.into(), ]
            [ $($setter)* ]
            [$($rest)*]
        }
    };

    // A field both sides share.
    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*]
        [
            $(#[$fmeta:meta])* $field:ident: $ty:ty,
            $($rest:tt)*
        ]
    ) => {
        paired_block! {
            @munch [$($meta)*] [$block] [$param] [$value]
            [
                $($bfield)*
                $(#[$fmeta])*
                #[cfg_attr(test, schema(required = true))]
                pub $field: $ty,
            ]
            [ $($pfield)* $(#[$fmeta])* pub $field: $ty, ]
            [ $($convert)* $field: $value.$field, ]
            [ $($barg)* $field: $ty, ]
            [ $($binit)* $field, ]
            [ $($bsetter)* ]
            [ $($arg)* $field: $ty, ]
            [ $($init)* $field, ]
            [ $($setter)* ]
            [$($rest)*]
        }
    };

    // The comma the entry rule added, when the last field brought its own.
    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*] [,]
    ) => {
        paired_block! {
            @munch [$($meta)*] [$block] [$param] [$value]
            [$($bfield)*] [$($pfield)*] [$($convert)*]
            [$($barg)*] [$($binit)*] [$($bsetter)*]
            [$($arg)*] [$($init)*] [$($setter)*] []
        }
    };

    (
        @munch [$($meta:tt)*] [$block:ident] [$param:ident] [$value:ident]
        [$($bfield:tt)*] [$($pfield:tt)*] [$($convert:tt)*]
        [$($barg:tt)*] [$($binit:tt)*] [$($bsetter:tt)*]
        [$($arg:tt)*] [$($init:tt)*] [$($setter:tt)*] []
    ) => {
        $($meta)*
        #[cfg_attr(test, derive(utoipa::ToSchema))]
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[non_exhaustive]
        pub struct $block {
            $($bfield)*
        }

        $($meta)*
        #[serde_with::skip_serializing_none]
        #[cfg_attr(test, derive(utoipa::ToSchema))]
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        #[non_exhaustive]
        pub struct $param {
            $($pfield)*
        }

        // The constructor of a block whose every field is optional takes no arguments. A
        // `Default` cannot stand in for it: one expansion serves every block, and the fields the
        // rest require have no default of their own.
        #[allow(clippy::new_without_default)]
        impl $block {
            #[doc = concat!("Creates a new [`", stringify!($block), "`].")]
            pub fn new($($barg)*) -> Self {
                Self { $($binit)* }
            }

            $($bsetter)*
        }

        #[allow(clippy::new_without_default)]
        impl $param {
            #[doc = concat!("Creates a new [`", stringify!($param), "`].")]
            pub fn new($($arg)*) -> Self {
                Self { $($init)* }
            }

            $($setter)*
        }

        impl From<$block> for $param {
            fn from($value: $block) -> Self {
                Self { $($convert)* }
            }
        }
    };
}

/// Declares a `type`-tagged union over payloads and its request-side twin.
///
/// A variant whose payload has a request-side twin names both, `response | request`; one whose
/// payload is shared names it once. Both unions carry the catch-all the rest of the crate uses,
/// which the generated description drops again.
///
/// ```ignore
/// paired_content! {
///     /// The content of a result from Claude's internal bash code execution tool.
///     BashCodeExecutionToolResultBlockContent as "BashCodeExecutionToolResultBlockContent"
///         | BashCodeExecutionToolResultBlockParamContent
///             as "BashCodeExecutionToolResultBlockParamContent"
///     {
///         /// A successful bash code execution result.
///         BashCodeExecutionResult(
///             BashCodeExecutionResultBlock | BashCodeExecutionResultBlockParam
///         ),
///     }
/// }
/// ```
macro_rules! paired_content {
    (
        $(#[$meta:meta])*
        $content:ident as $ctitle:literal | $param:ident as $ptitle:literal
        { $($variants:tt)* }
    ) => {
        // The title is what the description is matched by, so a title that is not the type's own
        // name would go unnoticed until the spec check drifted.
        const _: () = assert!(
            super::pair::same_name(stringify!($content), $ctitle)
                && super::pair::same_name(stringify!($param), $ptitle),
            "a union must be titled with its own name",
        );

        paired_content! {
            @munch [$(#[$meta])*] [$content] [$param] [$ctitle] [$ptitle] [] [] []
                [$($variants)* ,]
        }
    };

    // A variant over a pair of payloads.
    (
        @munch [$($meta:tt)*] [$content:ident] [$param:ident] [$ctitle:literal] [$ptitle:literal]
        [$($cvariant:tt)*] [$($pvariant:tt)*] [$($arm:tt)*]
        [
            $(#[$vmeta:meta])* $variant:ident($ty:ty | $pty:ty),
            $($rest:tt)*
        ]
    ) => {
        paired_content! {
            @munch [$($meta)*] [$content] [$param] [$ctitle] [$ptitle]
            [ $($cvariant)* $(#[$vmeta])* $variant($ty), ]
            [ $($pvariant)* $(#[$vmeta])* $variant($pty), ]
            [ $($arm)* $content::$variant(inner) => $param::$variant(inner.into()), ]
            [$($rest)*]
        }
    };

    // A variant over a payload both sides share.
    (
        @munch [$($meta:tt)*] [$content:ident] [$param:ident] [$ctitle:literal] [$ptitle:literal]
        [$($cvariant:tt)*] [$($pvariant:tt)*] [$($arm:tt)*]
        [
            $(#[$vmeta:meta])* $variant:ident($ty:ty),
            $($rest:tt)*
        ]
    ) => {
        paired_content! {
            @munch [$($meta)*] [$content] [$param] [$ctitle] [$ptitle]
            [ $($cvariant)* $(#[$vmeta])* $variant($ty), ]
            [ $($pvariant)* $(#[$vmeta])* $variant($ty), ]
            [ $($arm)* $content::$variant(inner) => $param::$variant(inner.into()), ]
            [$($rest)*]
        }
    };

    // The comma the entry rule added, when the last variant brought its own.
    (
        @munch [$($meta:tt)*] [$content:ident] [$param:ident] [$ctitle:literal] [$ptitle:literal]
        [$($cvariant:tt)*] [$($pvariant:tt)*] [$($arm:tt)*] [,]
    ) => {
        paired_content! {
            @munch [$($meta)*] [$content] [$param] [$ctitle] [$ptitle]
            [$($cvariant)*] [$($pvariant)*] [$($arm)*] []
        }
    };

    (
        @munch [$($meta:tt)*] [$content:ident] [$param:ident] [$ctitle:literal] [$ptitle:literal]
        [$($cvariant:tt)*] [$($pvariant:tt)*] [$($arm:tt)*] []
    ) => {
        $crate::utils::tagged_enum! {
            $($meta)*
            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
            #[serde(tag = "type", rename_all = "snake_case")]
            pub enum $content {
                $($cvariant)*
            }
        }

        $crate::utils::tagged_enum! {
            $($meta)*
            #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
            #[serde(tag = "type", rename_all = "snake_case")]
            pub enum $param {
                $($pvariant)*
            }
        }

        impl From<$content> for $param {
            fn from(value: $content) -> Self {
                match value {
                    $($arm)*
                    $content::Unknown(value) => $param::Unknown(value),
                }
            }
        }
    };
}

/// Whether two names are the same, in a `const` context.
pub(super) const fn same_name(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

pub(super) use paired_block;
pub(super) use paired_content;
