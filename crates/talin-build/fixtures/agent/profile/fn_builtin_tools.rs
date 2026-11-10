fn builtin_tools() -> Vec<<Self::Llm as ::talin::llm::Llm>::BuiltinTool> {
    vec![
        ::talin::__private::serde_json::from_str::<
            ::talin::llm::claude_messages::MessagesBuiltinTool,
        >("{\"type\":\"web_search_20260318\",\"name\":\"web_search\"}")
        .expect("Tool deserialization shouldn't fail"),
        ::talin::__private::serde_json::from_str::<
            ::talin::llm::claude_messages::MessagesBuiltinTool,
        >("{\"type\":\"code_execution_20260521\",\"name\":\"code_execution\"}")
        .expect("Tool deserialization shouldn't fail"),
    ]
}
