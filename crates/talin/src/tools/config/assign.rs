//! Dtolnay's Autoref Specialization via Wrapper
//!
//! The build-artifact codegen crate emits `navigate!()` macro calls that expand into `Wrapper(…)`
//! chains.  Autoref-based method resolution picks the correct impl at compile time WITHOUT the
//! macro ever inspecting the concrete user types. The runtime surface is:
//!
//! - navigate!(root.a.b)   — traverse owned root through fields a, b
//! - navigate!(*ref.a.b)   — traverse &mut ref (step_node at root first)
//! - .set(val)             — leaf assignment (handles `T` / `Option<T>`)
//! - .push_elem()          — Vec: push Default, return &mut to last element
//! - .insert_elem(key)     — HashMap: entry(key).or_default(), return &mut to value

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// A thin wrapper around a mutable reference to value.
///
/// Used to trigger autoref specialization for [`PeelOption`] and [`SetOption`].
#[repr(transparent)]
pub struct Wrapper<'a, T>(pub &'a mut T);

impl<T> Deref for Wrapper<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T> DerefMut for Wrapper<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

// --- Node Navigation ---

/// Dtolnay-specialized trait for peeling a layer of `Option`.
pub trait PeelOption {
    /// The peeled reference type.
    type Inner;

    /// Peels one layer of `Option` if present, returning a reference to the inner value.
    fn peel(self) -> Self::Inner;
}

/// Priority 1: by-value — matches only `Wrapper<Option<T>>`
impl<'a, T: Default> PeelOption for Wrapper<'a, Option<T>> {
    type Inner = &'a mut T;
    fn peel(self) -> &'a mut T {
        self.0.get_or_insert_with(T::default)
    }
}

/// Priority 2: autoref (&mut Wrapper) — matches everything else.
impl<'a, T> PeelOption for &'a mut Wrapper<'a, T> {
    type Inner = &'a mut T;
    fn peel(self) -> &'a mut T {
        self.0
    }
}

// ── Value Assignment ---

/// Dtolnay-specialized trait for assigining a value through `Option`.
pub trait SetOption<T> {
    /// Sets the value, wrapping in `Some` if the target is an `Option`.
    fn set(self, val: T);

    /// Sets the value, wrapping in `Some` if the target is an `Option`.
    fn set_from<V: Into<T>>(self, val: V)
    where
        Self: Sized,
    {
        self.set(val.into())
    }

    /// Sets the value parsed from the environment variable `var`.
    ///
    /// # Panics
    ///
    /// Panics if the variable is unset or cannot be parsed.
    fn set_from_env(self, var: &str)
    where
        Self: Sized,
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let value: T = std::env::var(var)
            .unwrap_or_else(|e| panic!("Failed to get env var '{var}': {e}"))
            .parse()
            .unwrap_or_else(|e| panic!("Failed to parse env var '{var}': {e}"));
        self.set(value);
    }
}

/// Priority 1: by-value — matches only `Wrapper<Option<T>>`
impl<T> SetOption<T> for Wrapper<'_, Option<T>> {
    fn set(self, val: T) {
        *self.0 = Some(val);
    }
}

/// Priority 2: autoref (&mut Wrapper) — matches everything else.
impl<T> SetOption<T> for &'_ mut Wrapper<'_, T> {
    fn set(self, val: T) {
        *self.0 = val;
    }
}

// --- Navigation Helpers ---

/// Navigates from a root through a dot-separated path of fields and returns a Wrapper to the leaf.
///
/// Leading `*` means the root is already a `&mut T` reference.
///
/// Peels any `Option` wrappers along the way.
#[doc(hidden)]
#[macro_export]
macro_rules! navigate {
    // Entry: owned root + fields — &mut taken at each field access
    // $($fields:tt).+ matches both named fields (ident) and tuple indices (literal)
    ($root:ident . $($fields:tt).+) => {
        $crate::__private::navigate!(@step $root, $($fields),+)
    };
    // Entry: &mut root + fields — wrap+step root first, then fields
    (* $root:ident . $($fields:tt).+) => {
        $crate::__private::navigate!(@step $crate::__private::Wrapper($root).peel(), $($fields),+)
    };
    // Base: zero fields, owned T — take &mut
    ($root:ident) => {
        $crate::__private::Wrapper(&mut $root)
    };
    // Base: zero fields, already &mut T — wrap directly
    (* $root:ident) => {
        $crate::__private::Wrapper($root)
    };
    // Internal base: one field remaining — wrap as leaf (named field)
    (@step $root:expr, $field:ident) => {
        $crate::__private::Wrapper(&mut $root.$field)
    };
    // Internal base: one field remaining — wrap as leaf (tuple index)
    (@step $root:expr, $idx:tt) => {
        $crate::__private::Wrapper(&mut $root.$idx)
    };
    // Internal recursive: wrap + step current field, continue to next (named field)
    (@step $root:expr, $first:ident, $($rest:tt),+) => {
        $crate::__private::navigate!(@step $crate::__private::Wrapper(&mut $root.$first).peel(), $($rest),+)
    };
    // Internal recursive: wrap + step current field, continue to next (tuple index)
    (@step $root:expr, $first:tt, $($rest:tt),+) => {
        $crate::__private::navigate!(@step $crate::__private::Wrapper(&mut $root.$first).peel(), $($rest),+)
    };
}

/// Helper trait to push an element to a `Vec`.
pub trait VecPushElem {
    /// The reference type to the pushed element.
    type Inner<'s>
    where
        Self: 's;

    /// Push a default element to a Vec and return a mutable reference to it.
    fn push_elem(&mut self) -> Self::Inner<'_>;
}

impl<T: Default> VecPushElem for &mut Vec<T> {
    type Inner<'s>
        = &'s mut T
    where
        Self: 's;
    fn push_elem(&mut self) -> &mut T {
        self.push(T::default());
        self.last_mut().unwrap()
    }
}

impl<T: Default> VecPushElem for &mut Option<Vec<T>> {
    type Inner<'s>
        = &'s mut T
    where
        Self: 's;

    fn push_elem(&mut self) -> &mut T {
        let v = self.get_or_insert_with(Vec::new);
        v.push(T::default());
        v.last_mut().unwrap()
    }
}

impl<T: Default> VecPushElem for Wrapper<'_, Vec<T>> {
    type Inner<'s>
        = &'s mut T
    where
        Self: 's;
    fn push_elem(&mut self) -> &mut T {
        self.0.push_elem()
    }
}

impl<T: Default> VecPushElem for Wrapper<'_, Option<Vec<T>>> {
    type Inner<'s>
        = &'s mut T
    where
        Self: 's;

    fn push_elem(&mut self) -> &mut T {
        self.0.push_elem()
    }
}

/// Helper trait to insert an element to a `HashMap`.
pub trait MapInsertElem<K> {
    /// The reference type to the inserted element.
    type Inner<'s>
    where
        Self: 's;

    /// Insert a default element to a HashMap at the given key and return a mutable reference to it.
    fn insert_elem(&mut self, key: K) -> Self::Inner<'_>;
}

impl<K: Eq + std::hash::Hash, V: Default> MapInsertElem<K> for &mut HashMap<K, V> {
    type Inner<'s>
        = &'s mut V
    where
        Self: 's;

    fn insert_elem(&mut self, key: K) -> &mut V {
        self.entry(key).or_default()
    }
}

impl<K: Eq + std::hash::Hash, V: Default> MapInsertElem<K> for &mut Option<HashMap<K, V>> {
    type Inner<'s>
        = &'s mut V
    where
        Self: 's;

    fn insert_elem(&mut self, key: K) -> &mut V {
        let map = self.get_or_insert_with(HashMap::new);
        map.entry(key).or_default()
    }
}

impl<K: Eq + std::hash::Hash, V: Default> MapInsertElem<K> for Wrapper<'_, HashMap<K, V>> {
    type Inner<'s>
        = &'s mut V
    where
        Self: 's;

    fn insert_elem(&mut self, key: K) -> &mut V {
        self.0.insert_elem(key)
    }
}

impl<K: Eq + std::hash::Hash, V: Default> MapInsertElem<K> for Wrapper<'_, Option<HashMap<K, V>>> {
    type Inner<'s>
        = &'s mut V
    where
        Self: 's;

    fn insert_elem(&mut self, key: K) -> &mut V {
        self.0.insert_elem(key)
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    #![allow(clippy::useless_conversion)]

    use super::*;

    // =====================================================================
    // User-Defined Types (completely opaque to the macro)
    //
    //    Intentionally uses meaningless names so the focus stays on the
    //    *structural* patterns the codegen must handle.
    // =====================================================================

    /// Root aggregate — every field exercises a different nesting pattern.
    #[derive(Default, Debug)]
    struct Qux {
        alpha: usize,                              // bare primitive
        beta: (i32, i32),                          // tuple
        gamma: Option<Waldo>,                      // Option<struct> (struct has nested Options)
        delta: Vec<Option<Plugh>>,                 // Vec<Option<struct>>
        epsilon: Blegh,                            // enum
        zeta: Option<Vec<Corge>>,                  // Option<Vec<struct>>  (struct has Option sub)
        eta: HashMap<String, Option<Grault>>,      // HashMap<_, Option<struct>>
        iota: Garply,                              // bare enum (NOT wrapped in Option)
        kappa: Option<Box<Thud>>,                  // Option<Box<struct>>  (Box + DerefMut)
        lambda: Option<HashMap<String, Vec<i32>>>, // Option<HashMap<_, Vec<_>>>
    }

    #[derive(Default, Debug)]
    struct Waldo {
        x: Option<Plugh>,
        y: i32,
    }

    #[derive(Default, Debug)]
    struct Plugh {
        m: Option<String>,
        n: Option<Xyzzy>,
    }

    /// Nested enum — used inside Option<Xyzzy> deep in the tree.
    #[derive(Debug, Default)]
    enum Xyzzy {
        Foo {
            a: i32,
            b: String,
        },
        #[default]
        Bar,
    }

    /// Enum with tuple-like variant
    #[derive(Debug, Default)]
    enum Blegh {
        Mewo(i32, String),
        #[default]
        Gah,
    }

    #[derive(Default, Debug)]
    struct Corge {
        p: String,
        q: Option<Fred>,
    }

    #[derive(Default, Debug)]
    struct Fred {
        v: i32,
    }

    #[derive(Default, Debug)]
    struct Grault {
        s: String,
        t: i32,
    }

    /// Bare enum — sits directly in Qux (not behind Option), so the codegen
    /// hits `AssignFallback` instead of `AssignOption`.
    #[derive(Debug, Default)]
    enum Garply {
        Left {
            j: i32,
            k: String,
        },
        #[default]
        Right,
    }

    #[derive(Default, Debug)]
    struct Thud {
        r: String,
        w: Option<Plugh>,
    }

    // =====================================================================
    // 3. Macro Codegen Simulation
    // =====================================================================
    //
    // Original TOML:
    // -----------------------------------------------------------------
    //     phi = 42
    //     epsilon = 77
    //
    //     [exa]
    //     "$0" = 1
    //
    //     [alpha]
    //     y = 7
    //
    //     [alpha.x]
    //     m = "deep"
    //
    //     [alpha.x.n]
    //     "$enum" = "Xyzzy"
    //     "$variant" = "Foo"
    //     a = 10
    //     b = "nested_enum"
    //
    //     [[beta]]
    //     m = "first"
    //
    //     [[beta]]
    //     m = "second"
    //
    //     [beeta]
    //     "$enum" = "Blegh"
    //     "$variant" = "Mewo"
    //     "$0" = 99
    //     "$1" = "enum_tuple"
    //
    //     [[gamma]]
    //     p = "g1"
    //
    //     [gamma.q]
    //     v = 99
    //
    //     [delta.key_a]
    //     s = "hello"
    //     t = 5
    //
    //     [zeta]
    //     "$enum" = "Garply"
    //     "$variant" = "Left"
    //     j = 99
    //     k = "bare"
    //
    //     [eta]
    //     r = "boxed"
    //
    //     [eta.w]
    //     m = "inside_box"
    //
    //     [theta]
    //     scores = [10, 20, 30]
    // -----------------------------------------------------------------

    #[test]

    fn main() {
        let mut __x = Qux::default();

        // ── phi = 42 ─────────────────────────────────────────────────────
        // Bare primitive — navigate wraps, set writes via Into (identity).
        navigate!(__x.alpha).set(42);

        // ── exa = (1, 2) ─────────────────────────────────────────────────
        navigate!(__x.beta.0).set(1);

        // ── [alpha] y = 7 ────────────────────────────────────────────────
        // step_node peels Option<Waldo>, then set writes y directly.
        navigate!(__x.gamma.y).set(7);

        // ── [alpha.x] m = "deep" ─────────────────────────────────────────
        // Two step_nodes deep: Option<Waldo> → Option<Plugh> → Plugh.m
        navigate!(__x.gamma.x.m).set_from("deep");

        // ── [alpha.x.n] _variant = "Foo", a = 10, b = "nested_enum" ─────
        // Enum variant — same set call, SetOption wraps in Some automatically.
        navigate!(__x.gamma.x.n).set(Xyzzy::Foo {
            a: Default::default(),
            b: Default::default(),
        });
        if let Xyzzy::Foo { a, b } = navigate!(__x.gamma.x.n).peel() {
            navigate!(*a).set(10);
            navigate!(*b).set_from("nested_enum");
        }

        // ── [[beta]]  (Vec<Option<Plugh>>) ───────────────────────────────
        // push_elem appends a default element; navigate!(*__e…) steps
        // through the Option wrapper before accessing fields.
        {
            let mut __v = navigate!(__x.delta);

            {
                let __e = __v.push_elem();
                navigate!(*__e.m).set_from("first");
            }

            {
                let __e = __v.push_elem();
                navigate!(*__e.m).set_from("second");
                navigate!(*__e.n).set_from(Xyzzy::Bar);
            }
        }

        navigate!(__x.epsilon).set(Blegh::Mewo(Default::default(), Default::default()));
        if let Blegh::Mewo(_0, _1) = navigate!(__x.epsilon).peel() {
            navigate!(*_0).set(99);
            navigate!(*_1).set_from("enum_tuple");
        }

        // ── [[gamma]]  (Option<Vec<Corge>>) ──────────────────────────────
        // navigate! wraps Option<Vec<…>>; push_elem peels the Option and
        // appends.  Nested Option<Fred> is stepped through transparently.
        {
            let mut __v = navigate!(__x.zeta);
            {
                let mut __e = __v.push_elem();
                navigate!(*__e.p).set_from("g1");
                navigate!(*__e.q.v).set(99);
            }
        }

        // ── [delta.key_a]  (HashMap<String, Option<Grault>>) ─────────────
        // insert_elem does entry().or_default(); navigate!(*__e…) steps
        // through the Option value before accessing struct fields.
        {
            let mut __m = navigate!(__x.eta);
            {
                let mut __e = __m.insert_elem("key_a".into());
                navigate!(*__e.s).set_from("hello");
                navigate!(*__e.t).set(5);
            }
        }

        // ── [zeta]  (Garply — bare enum, NOT Option) ─────────────────────
        // SetFallback fires (not SetOption) — direct assignment via Into.
        navigate!(__x.iota).set(Garply::Left {
            j: 99.into(),
            k: "bare".into(),
        });

        // ── [eta]  (Option<Box<Thud>>) ───────────────────────────────────
        // Box's DerefMut makes field access transparent to the codegen.
        {
            navigate!(__x.kappa.r).set_from("boxed");
            navigate!(__x.kappa.w.m).set_from("inside_box");
        }

        // ── [theta] scores = [10, 20, 30]  (Option<HashMap<String, Vec<i32>>>)
        // Combines all three helpers: navigate → insert_elem → push_elem.
        {
            let mut __m = navigate!(__x.lambda);
            {
                let mut __v = __m.insert_elem("scores".into());
                {
                    let mut __e = __v.push_elem();
                    navigate!(*__e).set(10);
                }
                {
                    let mut __e = __v.push_elem();
                    navigate!(*__e).set(20);
                }
                {
                    let mut __e = __v.push_elem();
                    navigate!(*__e).set(30);
                }
            }
        }

        // ── Output validation ────────────────────────────────────────────
        println!("{:#?}", __x);
    }
}
