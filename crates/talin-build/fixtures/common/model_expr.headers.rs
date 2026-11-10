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
                        .default_headers(
                            ::talin::__private::reqwest::header::HeaderMap::from_iter([
                                (
                                    ::talin::__private::reqwest::header::HeaderName::from_static("x-gateway-token"),
                                    {
                                        let mut value = ::talin::__private::reqwest::header::HeaderValue::from_str(
                                                &std::env::var("GATEWAY_TOKEN").unwrap_or_else(|e| {
                                                    panic!("Failed to get env var '{}': {}", "GATEWAY_TOKEN", e)
                                                }),
                                            )
                                            .unwrap_or_else(|e| {
                                                panic!("Invalid value for header '{}': {}", "x-gateway-token", e)
                                            });
                                        value.set_sensitive(true);
                                        value
                                    },
                                ),
                                (
                                    ::talin::__private::reqwest::header::HeaderName::from_static("x-tenant"),
                                    {
                                        let mut value = ::talin::__private::reqwest::header::HeaderValue::from_str(&String::from("acme"))
                                            .unwrap_or_else(|e| {
                                                panic!("Invalid value for header '{}': {}", "x-tenant", e)
                                            });
                                        value.set_sensitive(false);
                                        value
                                    },
                                )
                            ])
                        )
                        .build()
                        .unwrap(),
                )
                .build()
                .with_default_options(::talin::llm::LlmOptions::default()),
            <::talin::strategy::DropThoughtStrategy>::default(),
        )
    };
}
