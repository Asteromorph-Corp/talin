use crate::utils::string_enum;

string_enum! {
    /// The service tier configuration for a request.
    #[serde(rename_all = "snake_case")]
    pub enum ServiceTierParam {
        /// Uses the priority tier capacity if available, falling back to your other capacity if not.
        Auto,

        /// Only use standard tier capacity, useful if you don't want to use your priority tier
        /// capacity.
        StandardOnly,
    }
}

string_enum! {
    /// The service tier that was used to process a request.
    #[serde(rename_all = "snake_case")]
    pub enum ServiceTier {
        /// The standard tier was used.
        Standard,

        /// The priority tier was used.
        Priority,

        /// The batch tier was used.
        Batch,
    }
}

string_enum! {
    /// The inference speed mode for a request, and the mode that was used to process it.
    #[serde(rename_all = "snake_case")]
    pub enum Speed {
        /// Standard inference speed.
        Standard,

        /// High output-tokens-per-second inference.
        ///
        /// Some models require the `fast-mode-2026-02-01` beta header for any `speed` value.
        Fast,
    }
}
