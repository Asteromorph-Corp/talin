use talin_build::build_artifacts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_artifacts()?;
    Ok(())
}
