fn __dummy() {
    let _ = #[allow(clippy::default_constructed_unit_structs, clippy::unit_arg)]
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
    };
}
