#[automatically_derived]
impl<'a> From<&'a str> for MyAgentInput<'a> {
    fn from(input: &'a str) -> Self {
        Self::new(input)
    }
}
