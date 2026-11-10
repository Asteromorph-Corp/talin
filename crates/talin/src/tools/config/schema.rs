//! Compile-time description of which tool configuration fields must be supplied.
//!
//! A tool's config type can describe itself as a const tree of [`SchemaField`].
//! If it does, the configuration for the tool (supplied by `agents.toml`) is checked against the
//! schema at compile time.

pub use talin_macros::ConfigSchema;

/// A type that can describe itself as a schema of required fields.
pub trait ConfigSchema {
    /// The specification of fields required for the configuration.
    const SCHEMA: SchemaDef;
}

impl ConfigSchema for () {
    const SCHEMA: SchemaDef = SchemaDef::empty();
}
/// An `Option`'s inner fields are not checked, as the bound `T: ConfigSchema` would make
/// `Option<T>` of a type without a schema required.
impl<T> ConfigSchema for Option<T> {
    const SCHEMA: SchemaDef = SchemaDef::empty();
}
impl<T> ConfigSchema for Vec<T> {
    const SCHEMA: SchemaDef = SchemaDef::empty();
}
impl<K, V, S: Default> ConfigSchema for std::collections::HashMap<K, V, S> {
    const SCHEMA: SchemaDef = SchemaDef::empty();
}
impl<K, V> ConfigSchema for std::collections::BTreeMap<K, V> {
    const SCHEMA: SchemaDef = SchemaDef::empty();
}
impl<T, S: Default> ConfigSchema for std::collections::HashSet<T, S> {
    const SCHEMA: SchemaDef = SchemaDef::empty();
}
impl<T> ConfigSchema for std::collections::BTreeSet<T> {
    const SCHEMA: SchemaDef = SchemaDef::empty();
}

/// A field of a tool config type.
#[derive(Debug, Clone, Copy)]
pub struct SchemaField {
    name: &'static str,
    def: SchemaDef,
}

/// The schema of a type: whether a field of the type must be supplied, and its sub-fields.
#[derive(Debug, Clone, Copy)]
pub struct SchemaDef {
    children: &'static [SchemaField],
    pub(super) required: bool,
}

/// A field supplied by an agent.
#[derive(Debug, Clone, Copy)]
pub struct Supplied {
    name: &'static str,
    children: &'static [Supplied],
}

impl SchemaField {
    /// A field with the given name and schema definition.
    pub const fn new(name: &'static str, def: SchemaDef) -> Self {
        Self { name, def }
    }
}

impl SchemaDef {
    /// A schema definition that is not required and has no children.
    pub const fn empty() -> Self {
        Self {
            children: &[],
            required: false,
        }
    }

    /// A schema definition that is not required and has the given children.
    pub const fn node(children: &'static [SchemaField]) -> Self {
        Self {
            children,
            required: false,
        }
    }

    /// Sets the `required` property of the schema definition.
    pub const fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Returns `true` if all required fields are supplied.
    pub const fn is_complete(&self, supplied: &'static [Supplied]) -> bool {
        let mut i = 0;
        while i < self.children.len() {
            let field = &self.children[i];
            match find(supplied, field.name) {
                Some(found) => {
                    if !field.def.is_complete(found.children) {
                        return false;
                    }
                }
                None if field.def.required => return false,
                _ => {}
            }
            i += 1;
        }
        true
    }
}

impl Supplied {
    /// A supplied field with no inspectable sub-fields.
    pub const fn leaf(name: &'static str) -> Self {
        Self::node(name, &[])
    }

    /// A supplied field of which the sub-fields are supplied individually.
    pub const fn node(name: &'static str, children: &'static [Supplied]) -> Self {
        Self { name, children }
    }
}

const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn find(supplied: &'static [Supplied], name: &str) -> Option<&'static Supplied> {
    let mut i = 0;
    while i < supplied.len() {
        if str_eq(supplied[i].name, name) {
            return Some(&supplied[i]);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;
    use crate::tools::ConfigSchema;

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

    #[test]
    fn requires_required_fields() {
        const MISSING: bool = Config::SCHEMA.is_complete(&[]);
        const SUPPLIED: bool = Config::SCHEMA.is_complete(&[Supplied::leaf("api_key")]);
        const { assert!(!MISSING) };
        const { assert!(SUPPLIED) };
    }

    #[test]
    fn checks_inside_supplied_nested_fields_only() {
        const PARTIAL: bool = Config::SCHEMA.is_complete(&[
            Supplied::leaf("api_key"),
            Supplied::node("inner", &[Supplied::leaf("host")]),
        ]);
        const FULL: bool = Config::SCHEMA.is_complete(&[
            Supplied::leaf("api_key"),
            Supplied::node("inner", &[Supplied::leaf("port")]),
        ]);
        const { assert!(!PARTIAL) };
        const { assert!(FULL) };
    }

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

    /// Fields are required unless their type says otherwise, and a field whose type has children
    /// is checked inside when it is supplied.
    #[test]
    fn derive_requires_fields_by_default() {
        const OK: bool = Derived::SCHEMA.is_complete(&[
            Supplied::leaf("api_key"),
            Supplied::leaf("opaque"),
            Supplied::node("type", &[Supplied::leaf("port")]),
        ]);
        const NO_OPAQUE: bool = Derived::SCHEMA.is_complete(&[Supplied::leaf("api_key")]);
        const NO_KEY: bool = Derived::SCHEMA.is_complete(&[Supplied::leaf("opaque")]);
        const NO_PORT: bool = Derived::SCHEMA.is_complete(&[
            Supplied::leaf("api_key"),
            Supplied::leaf("opaque"),
            Supplied::node("type", &[Supplied::leaf("host")]),
        ]);
        const { assert!(OK) };
        const { assert!(!NO_OPAQUE) };
        const { assert!(!NO_KEY) };
        const { assert!(!NO_PORT) };
    }

    /// `Option` and collections may be left out, and a derived struct field may be left out as a
    /// whole.
    #[test]
    fn omittable_types() {
        #[derive(Default, ConfigSchema)]
        #[allow(dead_code)]
        struct Omittable {
            a: Option<Opaque>,
            b: Vec<Opaque>,
            c: std::collections::HashMap<String, Opaque>,
            d: std::collections::BTreeSet<Opaque>,
            e: (),
            f: DerivedInner,
        }
        const NONE: bool = Omittable::SCHEMA.is_complete(&[]);
        const PARTIAL: bool =
            Omittable::SCHEMA.is_complete(&[Supplied::node("f", &[Supplied::leaf("host")])]);
        const { assert!(NONE) };
        const { assert!(!PARTIAL) };
    }
}
