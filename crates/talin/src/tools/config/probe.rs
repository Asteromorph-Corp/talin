use std::marker::PhantomData;

use crate::tools::ConfigSchema;
use crate::tools::config::schema::SchemaDef;

/// Looks up the [`ConfigSchema`] of `T` if it has one.
///
/// `ConfigProbe::<T>::SCHEMA` resolves to `T::SCHEMA` when `T: ConfigSchema`, and otherwise to
/// [`ConfigProbeFallback::SCHEMA`], a required leaf.
/// This lets the generated code check configs that opt in without requiring every config to.
#[doc(hidden)]
pub struct ConfigProbe<T>(PhantomData<T>);

impl<T: ConfigSchema> ConfigProbe<T> {
    #[doc(hidden)]
    pub const SCHEMA: SchemaDef = T::SCHEMA;
}

#[doc(hidden)]
pub trait ConfigProbeFallback {
    const SCHEMA: SchemaDef = SchemaDef::empty().required(true);
}

impl<T> ConfigProbeFallback for ConfigProbe<T> {}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;
    use crate::tools::ConfigSchema;
    use crate::tools::config::schema::SchemaField;

    #[derive(Default)]
    struct Inner;
    impl ConfigSchema for Inner {
        const SCHEMA: SchemaDef =
            SchemaDef::node(&[SchemaField::new("port", SchemaDef::empty().required(true))]);
    }

    #[derive(Default)]
    struct Config;
    impl ConfigSchema for Config {
        const SCHEMA: SchemaDef = SchemaDef::node(&[
            SchemaField::new("api_key", SchemaDef::empty().required(true)),
            SchemaField::new("lang", Option::<String>::SCHEMA),
            SchemaField::new("inner", Inner::SCHEMA),
        ]);
    }

    struct Unchecked;

    /// A type without a schema.
    #[derive(Default)]
    struct Opaque;

    #[derive(Default, ConfigSchema)]
    #[allow(dead_code)]
    struct Derived {
        api_key: String,
        lang: Option<String>,
        opaque: Opaque,
        tags: Vec<String>,
        r#type: DerivedInner,
    }

    #[derive(Default, ConfigSchema)]
    #[allow(dead_code)]
    struct DerivedInner {
        port: u16,
    }

    #[test]
    fn probe_prefers_schema_when_present() {
        // `Config` has two required fields, `()` and the fallback have none.
        assert!(!ConfigProbe::<Config>::SCHEMA.is_complete(&[]));
        assert!(ConfigProbe::<()>::SCHEMA.is_complete(&[]));
        assert!(ConfigProbe::<Unchecked>::SCHEMA.is_complete(&[]));
        // The fallback is a required leaf, the schema of `()` is not required.
        const FALLBACK: SchemaDef = <ConfigProbe<Unchecked> as ConfigProbeFallback>::SCHEMA;
        const PROBED: SchemaDef = ConfigProbe::<Unchecked>::SCHEMA;
        const UNIT: SchemaDef = ConfigProbe::<()>::SCHEMA;
        assert!(FALLBACK.required && PROBED.required && !UNIT.required);
    }
}
