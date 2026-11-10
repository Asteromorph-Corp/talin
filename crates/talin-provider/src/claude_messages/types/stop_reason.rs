use crate::utils::string_enum;

string_enum! {
    /// The reason that Claude stopped.
    #[serde(rename_all = "snake_case")]
    pub enum StopReason {
        /// `"end_turn"`
        EndTurn,

        /// `"max_tokens"`
        MaxTokens,

        /// `"stop_sequence"`
        StopSequence,

        /// `"tool_use"`
        ToolUse,

        /// `"pause_turn"`
        PauseTurn,

        /// `"compaction"`
        Compaction,

        /// `"refusal"`
        Refusal,

        /// `"model_context_window_exceeded"`
        ModelContextWindowExceeded,
    }
}
