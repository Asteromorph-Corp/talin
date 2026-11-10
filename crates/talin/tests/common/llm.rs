//! Provider client builders.

/// Mock clients with test API keys and model names.
pub mod mock {
    use talin::llm::{
        ClaudeMessages, GeminiGenerateContent, GeminiInteractions, GenericChat, OpenAIChat,
        OpenAIResponses, SglangGenerate, XaiResponses,
    };

    const TEST_API_KEY: &str = "api-key-test";
    const TEST_MODEL: &str = "model-test";

    pub fn openai_chat(base: String, http: reqwest::Client) -> OpenAIChat {
        OpenAIChat::builder()
            .with_api_key(TEST_API_KEY)
            .with_api_base(base)
            .with_http_client(http)
            .with_model(TEST_MODEL)
            .build()
    }

    pub fn openai_responses(base: String, http: reqwest::Client) -> OpenAIResponses {
        OpenAIResponses::builder()
            .with_api_key(TEST_API_KEY)
            .with_api_base(base)
            .with_http_client(http)
            .with_model(TEST_MODEL)
            .build()
    }

    pub fn generic_chat(base: String, http: reqwest::Client) -> GenericChat {
        GenericChat::builder()
            .with_api_key(TEST_API_KEY)
            .with_api_base(base)
            .with_http_client(http)
            .with_model(TEST_MODEL)
            .build()
    }

    pub fn claude_messages(base: String, client: reqwest::Client) -> ClaudeMessages {
        ClaudeMessages::builder()
            .with_api_key(TEST_API_KEY)
            .with_api_base(base)
            .with_http_client(client)
            .with_model(TEST_MODEL)
            .build()
    }

    /// The exact path [`gemini_generate_content`] POSTs to (the model id is in the URL).
    pub fn gemini_generate_content_path() -> String {
        format!("/v1beta/models/{TEST_MODEL}:generateContent")
    }

    pub fn gemini_generate_content(base: String, http: reqwest::Client) -> GeminiGenerateContent {
        GeminiGenerateContent::builder()
            .with_api_key(TEST_API_KEY)
            .with_api_base(base)
            .with_http_client(http)
            .with_model(TEST_MODEL)
            .build()
    }

    pub fn gemini_interactions(base: String, http: reqwest::Client) -> GeminiInteractions {
        GeminiInteractions::builder()
            .with_api_key(TEST_API_KEY)
            .with_api_base(base)
            .with_http_client(http)
            .with_model(TEST_MODEL)
            .build()
    }

    pub fn xai_responses(base: String, http: reqwest::Client) -> XaiResponses {
        XaiResponses::builder()
            .with_api_key(TEST_API_KEY)
            .with_api_base(base)
            .with_http_client(http)
            .with_model(TEST_MODEL)
            .build()
    }

    pub fn sglang_generate(base: String, http: reqwest::Client) -> SglangGenerate {
        SglangGenerate::builder(base)
            .with_api_key(TEST_API_KEY)
            .with_http_client(http)
            .with_model(TEST_MODEL)
            .build()
    }
}

/// Live clients that read API keys from `.env` and use real model names.
pub mod live {
    use talin::llm::{
        ClaudeMessages, GeminiGenerateContent, GeminiInteractions, GenericChat, Llm, LlmOptions,
        OpenAIChat, OpenAIResponses, XaiResponses,
    };
    use talin::schemas::ReasoningEffort;

    const OPENAI_MODEL: &str = "gpt-4o-mini";
    const CLAUDE_MODEL: &str = "claude-haiku-4-5";
    const GEMINI_MODEL: &str = "gemini-3.5-flash-lite";
    const XAI_MODEL: &str = "grok-4.3";
    const OPENAI_REASONING_MODEL: &str = "o4-mini";
    const CLAUDE_REASONING_MODEL: &str = "claude-sonnet-5";

    /// Loads `.env` and returns the named key.
    fn key(name: &str) -> String {
        dotenvy::var(name).unwrap_or_else(|_| panic!("{name} is not set"))
    }

    pub fn generic_chat() -> GenericChat {
        key("OPENAI_API_KEY");
        GenericChat::builder().with_model(OPENAI_MODEL).build()
    }

    pub fn openai_chat() -> OpenAIChat {
        key("OPENAI_API_KEY");
        OpenAIChat::builder().with_model(OPENAI_MODEL).build()
    }

    pub fn openai_responses() -> OpenAIResponses {
        key("OPENAI_API_KEY");
        OpenAIResponses::builder().with_model(OPENAI_MODEL).build()
    }

    /// [`openai_responses`] on a reasoning model.
    pub fn openai_responses_reasoning() -> OpenAIResponses {
        key("OPENAI_API_KEY");
        OpenAIResponses::builder()
            .with_model(OPENAI_REASONING_MODEL)
            .build()
    }

    pub fn xai_responses() -> XaiResponses {
        XaiResponses::builder()
            .with_api_key(key("XAI_API_KEY"))
            .with_model(XAI_MODEL)
            .build()
    }

    pub fn claude_messages() -> ClaudeMessages {
        ClaudeMessages::builder()
            .with_api_key(key("CLAUDE_API_KEY"))
            .with_model(CLAUDE_MODEL)
            .build()
    }

    /// [`claude_messages`] on a model that supports adaptive thinking.
    pub fn claude_messages_reasoning() -> ClaudeMessages {
        ClaudeMessages::builder()
            .with_api_key(key("CLAUDE_API_KEY"))
            .with_model(CLAUDE_REASONING_MODEL)
            .build()
    }

    pub fn gemini_generate_content() -> GeminiGenerateContent {
        GeminiGenerateContent::builder()
            .with_api_key(key("GEMINI_API_KEY"))
            .with_model(GEMINI_MODEL)
            .build()
    }

    pub fn gemini_interactions() -> GeminiInteractions {
        GeminiInteractions::builder()
            .with_api_key(key("GEMINI_API_KEY"))
            .with_model(GEMINI_MODEL)
            .build()
    }

    /// Caps the output tokens, so that a prompt demanding a long answer is cut short.
    pub fn with_max_tokens<L>(llm: L, max_tokens: u32) -> L
    where
        L: Llm,
        L::Options: From<LlmOptions>,
    {
        llm.with_default_options(LlmOptions::default().with_max_tokens(max_tokens))
    }

    /// Asks the model to reason before it answers.
    pub fn with_reasoning<L>(llm: L, effort: ReasoningEffort) -> L
    where
        L: Llm,
        L::Options: From<LlmOptions>,
    {
        llm.with_default_options(LlmOptions::default().with_reasoning_effort(effort))
    }
}
