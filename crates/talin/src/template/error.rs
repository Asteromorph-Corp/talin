use thiserror::Error;

/// Errors that can occur while formatting prompt template.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum TemplateError {
    /// A required input variable is missing.
    #[error("missing input variable: {0}")]
    MissingVariable(String),
}
