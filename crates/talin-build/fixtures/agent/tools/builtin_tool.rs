fn __dummy() {
    let _ = ::talin::__private::serde_json::from_str::<
        ::talin::llm::claude_messages::MessagesBuiltinTool,
    >("{\"type\":\"web_search_20260318\",\"name\":\"web_search\"}")
    .expect("Tool deserialization shouldn't fail");
}
