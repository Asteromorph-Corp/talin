pub mod my_agent_input {
    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
    pub struct Language(pub String);
    impl Language {
        #[allow(dead_code)]
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }
    }
    #[automatically_derived]
    impl std::fmt::Display for Language {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
}
#[allow(unused_imports)]
use my_agent_input::Language;
