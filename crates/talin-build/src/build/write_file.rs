use std::fs;
use std::path::Path;

/// Write the file only if its contents are different from the new contents.
/// This avoids unnecessary rebuilds triggered by file timestamp changes.
pub(super) fn write_if_changed(path: &Path, new: &syn::File) -> std::io::Result<()> {
    let new_pretty = prettyplease::unparse(new);
    // Try reading the file; compare to new content
    if !fs::read_to_string(path).is_ok_and(|content| content == new_pretty) {
        fs::write(path, new_pretty)?;
    }
    Ok(())
}

macro_rules! write_file {
    ($path:expr, $code:expr $(,)?) => {{
        let path = $path;
        let code = $code;
        $crate::build::write_file::write_if_changed(&path, &code)
            .map_err(|e| IoError::write_file_failure(&path, e))
    }};
}

pub(super) use write_file;
