pub use talin_macros::Ctor;

use crate::chain::{ChainInput, ChainOutput, PartialOutput};

mod sealed {
    pub trait Sealed {}
}

/// A type alias for accessing the partial type of a [`OutputCtor`].
///
/// This is a convenience alias for `<O as OutputCtor>::Partial`.
pub type PartialTarget<O> = <O as OutputCtor>::Partial;

/// A constructor trait for types that can be constructed with a lifetime parameter.
///
/// Since Rust does not support higher-kinded types yet, this trait serves as a workaround to
/// allow generic structs to be parameterized by type constructors rather than concrete types.
///
/// # The Problem
///
/// Suppose you want a `Chain` struct that can work with different input/output types, where
/// those types may contain borrowed data. You might try:
///
/// ```ignore
/// struct Chain<I, O> { /* ... */ }
///
/// impl<I, O> Chain<I, O> {
///     fn call<'a>(&self, input: I<'a>) -> O<'a> { /* ... */ }
///     //                        ^^^^    ^^^^
///     //                        Error! lifetime arguments are not allowed on type parameters I, O
/// }
/// ```
///
/// This doesn't work because lifetime arguments are not allowed on type parameters `I` and `O`.
///
/// # The Solution
///
/// The `Ctor` trait provides a way to represent type constructors. Instead of parameterizing
/// `Chain` with concrete types, we parameterize it with types that implement `Ctor`:
///
/// ```ignore
/// struct Chain<I: InputCtor, O: OutputCtor> { /* ... */ }
///
/// impl<I: InputCtor, O: OutputCtor> Chain<I, O> {
///     fn call<'a>(&self, input: I::Target<'a>) -> O::Target<'a> { /* ... */ }
///     //                        ^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^
///     //                        Now we can apply a lifetime to get the actual type!
/// }
/// ```
///
/// # Example
///
/// ```ignore
/// // Define a type that borrows data
/// struct MyInput<'a> {
///     data: &'a str,
/// }
///
/// // Create a Ctor for it (or derive with #[derive(Ctor)])
/// struct MyInputCtor;
/// impl Ctor for MyInputCtor {
///     type Target<'a> = MyInput<'a>;
/// }
///
/// // Now Chain<MyInputCtor, SomeOutputCtor> can have methods that work with
/// // MyInput<'a> for any lifetime 'a, without Chain itself having a lifetime parameter.
/// ```
pub trait Ctor: Send + Sync + 'static {
    /// The target type.
    type Target<'a>: Send + Sync + 'a;
}

/// A [constructor](Ctor) trait for types that can be used as chain inputs.
///
/// This trait is sealed, and automatically implemented for all types that implement [`Ctor`] if its
/// target is a [`ChainInput`].
pub trait InputCtor: sealed::Sealed + Send + Sync + 'static {
    /// The target type.
    type Target<'a>: ChainInput + 'a;
}

/// A [constructor](Ctor) trait for types that can be used as chain outputs.
///
/// This trait is sealed, and automatically implemented for all types that implement [`Ctor`] if its
/// target is a [`ChainOutput`] and has a consistent [`PartialOutput`].
pub trait OutputCtor: sealed::Sealed + Send + Sync + 'static {
    /// The target type.
    type Target<'a>: ChainOutput<Partial = Self::Partial> + 'a;

    /// The partial type for the target type.
    ///
    /// While the partial type should remain consistent for all lifetime `'a`, we need a lifetime
    /// `'a` when accessing it through the associated type (like `Self::Target::<'a>::Partial`).
    ///
    /// This associated type prevents such dillema.
    type Partial: PartialOutput;
}

impl<T: Ctor> sealed::Sealed for T {}

impl<T> InputCtor for T
where
    T: Ctor + sealed::Sealed,
    for<'a> T::Target<'a>: ChainInput,
{
    type Target<'a> = T::Target<'a>;
}

impl<T, P> OutputCtor for T
where
    T: Ctor + sealed::Sealed,
    P: PartialOutput,
    for<'any> T::Target<'any>: ChainOutput<Partial = P>,
{
    type Target<'a> = T::Target<'a>;
    type Partial = P;
}

/// A [constructor](Ctor) for `String`.
pub struct StringCtor;
impl Ctor for StringCtor {
    type Target<'a> = String;
}

/// A [constructor](Ctor) for `&str`.
pub struct StrCtor;
impl Ctor for StrCtor {
    type Target<'a> = &'a str;
}

/// An implementation of [constructor](Ctor) for the unit type `()`.
impl Ctor for () {
    type Target<'a> = ();
}
