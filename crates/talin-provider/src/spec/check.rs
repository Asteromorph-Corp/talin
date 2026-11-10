//! Keeps the committed OpenAPI descriptions in step with the types they are derived from.
//!
//! When a change to the types moves a description, regenerate it:
//!
//! ```sh
//! SPEC_BLESS=1 cargo test -p talin-provider generated_spec_matches_types
//! ```

use std::path::Path;
use std::{env, fs};

use utoipa::openapi::OpenApi;

/// Set to any value to rewrite the committed description instead of checking it.
const BLESS: &str = "SPEC_BLESS";

/// Checks that `openapi` matches the description committed at `spec`, relative to the crate root.
pub(crate) fn check(spec: &str, openapi: &OpenApi) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(spec);

    let mut openapi_json = serde_json::to_value(openapi).expect("serializing OpenAPI spec");
    openapi_json.sort_all_objects();
    let generated = format!("{:#}\n", openapi_json);

    if env::var_os(BLESS).is_some() {
        fs::write(&path, &generated)
            .unwrap_or_else(|error| panic!("writing {}: {error}", path.display()));
        return;
    }

    let committed = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));

    assert!(
        committed == generated,
        "{}",
        report(spec, &committed, &generated)
    );
}

/// Describes the first line where the two descriptions part, and how to reconcile them.
///
/// Not an `assert_eq!`, which would print both copies of a four-thousand-line document and bury
/// the handful of lines that moved.
fn report(spec: &str, committed: &str, generated: &str) -> String {
    let parted = committed
        .lines()
        .zip(generated.lines())
        .enumerate()
        .find(|(_, (committed, generated))| committed != generated);

    let detail = match parted {
        Some((index, (committed, generated))) => format!(
            "they part at line {}:\n     committed: {}\n     generated: {}",
            index + 1,
            committed.trim(),
            generated.trim(),
        ),
        // Reached when one is a prefix of the other, which zip stops short of reporting.
        None => format!(
            "the committed description is {} lines and the generated one {}",
            committed.lines().count(),
            generated.lines().count(),
        ),
    };

    format!(
        "{spec} is not what this crate's types produce — {detail}\n\n\
         Regenerate it with \
         `{BLESS}=1 cargo test -p talin-provider generated_spec_matches_types`, and \
         check the diff: it is the change this crate makes to the wire format."
    )
}
