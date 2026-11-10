use crate::error::TalinError;

pub(crate) const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";
pub(crate) const OUT_DIR: &str = "OUT_DIR";

pub(crate) fn get_env_var(env: &'static str) -> Result<String, TalinError> {
    std::env::var(env).map_err(|_| TalinError::MissingEnvVar(env))
}
