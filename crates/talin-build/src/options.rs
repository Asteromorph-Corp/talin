use crate::build_artifacts_with;
use crate::error::TalinError;

/// Options for [`build_artifacts_with`].
///
/// ```no_run
/// # use talin_build::{ArtifactOptions, InputFieldOrder};
/// ArtifactOptions::default()
///     .input_field_order(InputFieldOrder::Appearance)
///     .use_newtype_inputs(true)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArtifactOptions {
    pub(crate) input_field_order: InputFieldOrder,
    pub(crate) use_newtype_inputs: bool,
}

/// The order of the input fields in generated input structs, their `new` constructors, and the
/// `*_immed` functions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputFieldOrder {
    /// In alphabetical order of the field name.
    ///
    /// This is the default order, as the ordering is stable across prompt edits.
    #[default]
    Alphabetical,
    /// In the order the variables first appear in the system prompt, then the user prompt, then
    /// the prompt overrides, followed by the media attachments.
    ///
    /// Editing a prompt can change the order of arguments for `new` and the `*_immed` functions.
    /// This can break silently if same-type arguments are swapped, e.g. `new(language, country)` vs
    /// `new(country, language)`.
    /// Consider pairing this with  [`ArtifactOptions::use_newtype_inputs`] to catch swapped
    /// arguments at compile time.
    Appearance,
}

impl ArtifactOptions {
    /// Sets the order of the input fields. Defaults to [`InputFieldOrder::Alphabetical`].
    pub fn input_field_order(mut self, order: InputFieldOrder) -> Self {
        self.input_field_order = order;
        self
    }

    /// Whether string inputs are wrapped in generated newtypes. Defaults to `false`.
    ///
    /// When enabled, an input `language` is typed `Language` with `struct Language(pub String);`
    /// instead of `String`, in the input struct, its `new` constructor, and the `*_immed`
    /// functions, so that swapped arguments fail to compile.
    pub fn use_newtype_inputs(mut self, enabled: bool) -> Self {
        self.use_newtype_inputs = enabled;
        self
    }

    /// Builds the artifacts with these options. Equivalent to
    /// [`build_artifacts_with(self)`](crate::build_artifacts_with).
    pub fn build(self) -> Result<(), TalinError> {
        build_artifacts_with(self)
    }
}
