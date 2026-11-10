use talin_build::build_artifacts;

fn main() {
    build_artifacts().unwrap_or_else(|err| println!("cargo::error={err:?}"));
}
