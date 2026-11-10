use base64::Engine;
use base64::prelude::BASE64_STANDARD;

/// Renders raw image bytes as a base64 `data:` URL.
pub(crate) fn render_bytes(bytes: impl AsRef<[u8]>, mime: impl std::fmt::Display) -> String {
    let content = BASE64_STANDARD.encode(bytes);
    format!("data:{mime};base64,{content}")
}
