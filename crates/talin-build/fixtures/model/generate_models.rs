#[allow(dead_code)]
pub struct Models;
impl Models {
    /// The names accepted by [`Models::any`].
    #[allow(dead_code)]
    pub const NAMES: &'static [&'static str] = &["my_model"];
    /// Looks up a model preset by name at runtime.
    ///
    /// Fails with `UnknownModel` if no model has the given name; see [`Models::NAMES`] for the accepted names.
    #[allow(dead_code, clippy::match_single_binding)]
    pub fn any(
        name: &str,
    ) -> ::std::result::Result<
        ::talin::llm::AnyPreset,
        ::talin::llm::UnknownModel,
    > {
        match name {
            "my_model" => ::std::result::Result::Ok(Self::my_model().into()),
            other => ::std::result::Result::Err(::talin::llm::UnknownModel::new(
                other,
                Self::NAMES,
            )),
        }
    }
    #[allow(dead_code)]
    pub fn my_model() -> ::talin::llm::LlmPreset<
        ::talin::llm::GenericChat,
        ::talin::strategy::DropThoughtStrategy,
    > {
        #[allow(clippy::default_constructed_unit_structs, clippy::unit_arg)]
        {
            use ::talin::llm::Llm;
            ::talin::llm::LlmPreset::new(
                ::talin::llm::GenericChat::builder()
                    .with_api_base(String::from("https://www.example.com"))
                    .with_model(String::from("my_model"))
                    .with_http_client(
                        ::talin::__private::reqwest::Client::builder()
                            .connection_verbose(true)
                            .build()
                            .unwrap(),
                    )
                    .build()
                    .with_default_options(::talin::llm::LlmOptions::default()),
                <::talin::strategy::DropThoughtStrategy>::default(),
            )
        }
    }
}
