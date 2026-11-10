use crate::utils::string_enum;

string_enum! {
    /// How a server tool's result blocks appear in the response when the result was consumed by a
    /// completed `code_execution` call in the same turn.
    ///
    /// Results from direct calls, or from `code_execution` calls that paused before completing, are
    /// always returned in full so they can be sent back on the next turn.
    #[serde(rename_all = "snake_case")]
    pub enum ResponseInclusion {
        /// Return the complete content. This is the default.
        Full,

        /// Drop the nested server tool use and result block pair entirely.
        Excluded,
    }
}

// `string_enum!` does not support `#[default]`.
#[allow(clippy::derivable_impls)]
impl Default for ResponseInclusion {
    fn default() -> Self {
        Self::Full
    }
}
