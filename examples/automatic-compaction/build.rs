fn main() -> Result<(), Box<dyn std::error::Error>> {
    talin_build::build_artifacts()?;
    Ok(())
}
