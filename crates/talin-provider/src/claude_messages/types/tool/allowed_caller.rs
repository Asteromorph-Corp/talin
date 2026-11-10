use crate::utils::string_enum;

string_enum! {
    /// Restricts which caller may invoke a server tool.
    #[serde(rename_all = "snake_case")]
    pub enum AllowedCaller {
        /// The tool is invoked directly by the model.
        Direct,

        /// The tool is invoked from within the `code_execution_20250825` tool.
        #[serde(rename = "code_execution_20250825")]
        CodeExecution20250825,

        /// The tool is invoked from within the `code_execution_20260120` tool.
        #[serde(rename = "code_execution_20260120")]
        CodeExecution20260120,

        /// The tool is invoked from within the `code_execution_20260521` tool.
        #[serde(rename = "code_execution_20260521")]
        CodeExecution20260521,
    }
}
