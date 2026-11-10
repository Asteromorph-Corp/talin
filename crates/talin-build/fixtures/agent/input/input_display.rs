#[automatically_derived]
impl std::fmt::Display for MyAgentInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}: {}",
            stringify!(_preset_current_date),
            self._preset_current_date
        )?;
        writeln!(f, "{}: {}", stringify!(input), self.input)?;
        Ok(())
    }
}
