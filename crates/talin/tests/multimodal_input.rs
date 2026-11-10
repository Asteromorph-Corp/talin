//! Attach media to a turn and send it to each provider:
//! - The request must carry the media in the provider's own encoding, bytes included.
//! - Providers with nowhere to put it must say so instead of quietly dropping it.
//!
//! The media itself is not in the repository. Fetch it with `tests/fixtures/fetch.sh`.
#![allow(unreachable_pub)]

mod common;

/// The media the tests attach, and the input that carries it.
mod media {
    use talin::agent::Agent;
    use talin::chain::{ChainInput, Ctor};
    use talin::llm::Llm;
    use talin::schemas::{
        DocumentContent, DocumentMimeType, ImageContent, ImageMimeType, MediaSource,
    };
    use talin::template::MessageTemplate;

    /// A photograph of a road sign reading `STOP`.
    const IMAGE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/stop_sign.jpg");
    /// A one-page PDF reading `Dummy PDF file`.
    const DOCUMENT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/dummy.pdf");

    pub const IMAGE_MIME: &str = "image/jpeg";
    pub const DOCUMENT_MIME: &str = "application/pdf";
    /// The file name attached to the document, which some providers cite back.
    pub const DOCUMENT_NAME: &str = "dummy.pdf";
    /// The word on the sign in the image.
    pub const SIGN: &str = "STOP";
    /// The text of the document.
    pub const DOCUMENT_TEXT: &str = "Dummy PDF file";

    /// The turn: a question, and the media it is asked about.
    #[derive(ChainInput, Ctor)]
    pub struct Ask {
        #[chain(into = "text")]
        pub question: String,
        #[chain(into = "images")]
        pub images: Option<Vec<ImageContent>>,
        #[chain(into = "documents")]
        pub documents: Option<Vec<DocumentContent>>,
    }

    impl Ask {
        /// Asks about [`image`].
        pub fn about_image(question: &str) -> Self {
            Self {
                question: question.to_owned(),
                images: Some(vec![image()]),
                documents: None,
            }
        }

        /// Asks about [`document`].
        pub fn about_document(question: &str) -> Self {
            Self {
                question: question.to_owned(),
                images: None,
                documents: Some(vec![document()]),
            }
        }
    }

    /// An agent whose user message is the question followed by whichever media slot is filled.
    pub fn agent<L: Llm>(llm: L) -> Agent<L, AskCtor> {
        let user_message = MessageTemplate::fstring("{question}")
            .with_media(["images".into(), "documents".into()]);
        Agent::builder().user_message(user_message).build(llm)
    }

    pub fn image() -> ImageContent {
        MediaSource::from_raw(read(IMAGE_PATH)).image(ImageMimeType::Jpeg)
    }

    pub fn document() -> DocumentContent {
        MediaSource::from_raw(read(DOCUMENT_PATH))
            .document(DocumentMimeType::Pdf)
            .with_name(DOCUMENT_NAME)
    }

    /// The image as the request carries it.
    pub fn image_base64() -> String {
        encode(read(IMAGE_PATH))
    }

    /// The document as the request carries it.
    pub fn document_base64() -> String {
        encode(read(DOCUMENT_PATH))
    }

    fn read(path: &str) -> Vec<u8> {
        std::fs::read(path)
            .unwrap_or_else(|e| panic!("{path}: {e}. Run `tests/fixtures/fetch.sh` to fetch it"))
    }

    fn encode(bytes: Vec<u8>) -> String {
        use base64::Engine;
        base64::prelude::BASE64_STANDARD.encode(bytes)
    }
}

/// Sends the media to a mock server.
mod mock {
    use mockito::Matcher;
    use serde_json::Value;
    use talin::agent::AgentError;
    use talin::llm::{Llm, LlmError};

    use crate::common::{MockServer, mock};
    use crate::media::{
        Ask, DOCUMENT_MIME, DOCUMENT_NAME, IMAGE_MIME, document_base64, image_base64,
    };
    use crate::{common, media};

    const QUESTION: &str = "What is this?";
    /// The reply, which only pins that the turn ran; the request body is the real assertion.
    const REPLY: &str = "a sign";

    /// Asserts the attached media reaches the request in the provider's own encoding.
    async fn assert_attaches_media<L: Llm>(
        path: &str,
        llm: fn(String, reqwest::Client) -> L,
        media_request: Value,
        response: fn(&str) -> Value,
        input: Ask,
    ) {
        let mut mock = MockServer::start().await;
        let request = Matcher::PartialJson(media_request);
        let m = mock.expect_post(path, request, response(REPLY)).await;

        let llm = llm(mock.base_url(), mock.http_client());
        let result = media::agent(llm).session().run_turn(input).await;

        m.assert_async().await;
        let answer = result.expect("agent turn failed");
        assert_eq!(answer, REPLY);
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn generic_chat_attaches_image() {
        assert_attaches_media(
            "/chat/completions",
            mock::generic_chat,
            common::openai_chat_image_request(&image_base64(), IMAGE_MIME),
            common::openai_chat_response,
            Ask::about_image(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn openai_chat_attaches_image() {
        assert_attaches_media(
            "/chat/completions",
            mock::openai_chat,
            common::openai_chat_image_request(&image_base64(), IMAGE_MIME),
            common::openai_chat_response,
            Ask::about_image(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn openai_responses_attaches_image() {
        assert_attaches_media(
            "/responses",
            mock::openai_responses,
            common::openai_responses_image_request(&image_base64(), IMAGE_MIME),
            common::openai_responses_response,
            Ask::about_image(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn xai_responses_attaches_image() {
        assert_attaches_media(
            "/responses",
            mock::xai_responses,
            common::xai_responses_image_request(&image_base64(), IMAGE_MIME),
            common::xai_responses_response,
            Ask::about_image(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn claude_messages_attaches_image() {
        assert_attaches_media(
            "/messages",
            mock::claude_messages,
            common::claude_messages_image_request(&image_base64(), IMAGE_MIME),
            common::claude_messages_response,
            Ask::about_image(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn gemini_generate_content_attaches_image() {
        assert_attaches_media(
            &mock::gemini_generate_content_path(),
            mock::gemini_generate_content,
            common::gemini_generate_content_image_request(&image_base64(), IMAGE_MIME),
            common::gemini_generate_content_response,
            Ask::about_image(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn gemini_interactions_attaches_image() {
        assert_attaches_media(
            "/v1/interactions",
            mock::gemini_interactions,
            common::gemini_interactions_image_request(&image_base64(), IMAGE_MIME),
            common::gemini_interactions_response,
            Ask::about_image(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn generic_chat_attaches_document() {
        assert_attaches_media(
            "/chat/completions",
            mock::generic_chat,
            common::openai_chat_document_request(&document_base64(), DOCUMENT_MIME, DOCUMENT_NAME),
            common::openai_chat_response,
            Ask::about_document(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn openai_chat_attaches_document() {
        assert_attaches_media(
            "/chat/completions",
            mock::openai_chat,
            common::openai_chat_document_request(&document_base64(), DOCUMENT_MIME, DOCUMENT_NAME),
            common::openai_chat_response,
            Ask::about_document(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn openai_responses_attaches_document() {
        assert_attaches_media(
            "/responses",
            mock::openai_responses,
            common::openai_responses_document_request(
                &document_base64(),
                DOCUMENT_MIME,
                DOCUMENT_NAME,
            ),
            common::openai_responses_response,
            Ask::about_document(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn xai_responses_attaches_document() {
        assert_attaches_media(
            "/responses",
            mock::xai_responses,
            common::xai_responses_document_request(
                &document_base64(),
                DOCUMENT_MIME,
                DOCUMENT_NAME,
            ),
            common::xai_responses_response,
            Ask::about_document(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn claude_messages_attaches_document() {
        assert_attaches_media(
            "/messages",
            mock::claude_messages,
            common::claude_messages_document_request(
                &document_base64(),
                DOCUMENT_MIME,
                DOCUMENT_NAME,
            ),
            common::claude_messages_response,
            Ask::about_document(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn gemini_generate_content_attaches_document() {
        assert_attaches_media(
            &mock::gemini_generate_content_path(),
            mock::gemini_generate_content,
            common::gemini_generate_content_document_request(
                &document_base64(),
                DOCUMENT_MIME,
                DOCUMENT_NAME,
            ),
            common::gemini_generate_content_response,
            Ask::about_document(QUESTION),
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn gemini_interactions_attaches_document() {
        assert_attaches_media(
            "/v1/interactions",
            mock::gemini_interactions,
            common::gemini_interactions_document_request(
                &document_base64(),
                DOCUMENT_MIME,
                DOCUMENT_NAME,
            ),
            common::gemini_interactions_response,
            Ask::about_document(QUESTION),
        )
        .await;
    }

    /// sglang's `/generate` is a text-completion endpoint, so media has nowhere to go. The turn
    /// must fail rather than send a prompt with the media missing.
    #[tokio::test]
    #[ignore = "requires fixtures files"]
    async fn sglang_rejects_media() {
        let mock = MockServer::start().await;
        let llm = mock::sglang_generate(mock.base_url(), mock.http_client());
        let error = media::agent(llm)
            .session()
            .run_turn(Ask::about_image(QUESTION))
            .await
            .expect_err("sglang has no way to send media");

        assert!(
            matches!(error, AgentError::Model(LlmError::UnsupportedContent(_))),
            "expected the media to be refused, got {error:?}"
        );
    }
}

/// Sends the media to a live LLM provider.
///
/// This is where the encoding is confirmed: the provider decoded the bytes and the model answered
/// from what they contain, which a mock server cannot tell us.
///
/// The tests hit real providers, cost money, and are non-deterministic, so every test is
/// `#[ignore]`d. Run the tests by:
///
/// ```sh
/// cargo test --test multimodal_input live::claude -- --ignored --nocapture
/// ```
mod live {
    use talin::llm::Llm;

    use crate::common::live;
    use crate::media::{self, Ask, DOCUMENT_TEXT, SIGN};

    const IMAGE_QUESTION: &str = "What word is written on the sign? Reply with that word only.";
    const DOCUMENT_QUESTION: &str = "Reply with the text of the document, verbatim.";

    /// Asks a provider about the media, and returns what the model made of it.
    async fn ask<L: Llm>(llm: L, input: Ask) -> String {
        let answer = media::agent(llm)
            .session()
            .run_turn(input)
            .await
            .expect("agent turn failed to reach a final answer");

        eprintln!("answer: {answer}");
        answer.to_lowercase()
    }

    /// The model read the word off the photograph, so the image arrived intact.
    async fn assert_attaches_image<L: Llm>(llm: L) {
        let answer = ask(llm, Ask::about_image(IMAGE_QUESTION)).await;
        assert!(
            answer.contains(&SIGN.to_lowercase()),
            "expected the sign to read {SIGN:?}, got {answer:?}"
        );
    }

    /// The model read the text out of the PDF, so the document arrived intact.
    async fn assert_attaches_document<L: Llm>(llm: L) {
        let answer = ask(llm, Ask::about_document(DOCUMENT_QUESTION)).await;
        assert!(
            answer.contains(&DOCUMENT_TEXT.to_lowercase()),
            "expected the document to read {DOCUMENT_TEXT:?}, got {answer:?}"
        );
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn generic_chat_attaches_image() {
        assert_attaches_image(live::generic_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_chat_attaches_image() {
        assert_attaches_image(live::openai_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_responses_attaches_image() {
        assert_attaches_image(live::openai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires XAI_API_KEY"]
    async fn xai_responses_attaches_image() {
        assert_attaches_image(live::xai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires CLAUDE_API_KEY"]
    async fn claude_messages_attaches_image() {
        assert_attaches_image(live::claude_messages()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_generate_content_attaches_image() {
        assert_attaches_image(live::gemini_generate_content()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_interactions_attaches_image() {
        assert_attaches_image(live::gemini_interactions()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn generic_chat_attaches_document() {
        assert_attaches_document(live::generic_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_chat_attaches_document() {
        assert_attaches_document(live::openai_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_responses_attaches_document() {
        assert_attaches_document(live::openai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires XAI_API_KEY"]
    async fn xai_responses_attaches_document() {
        assert_attaches_document(live::xai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires CLAUDE_API_KEY"]
    async fn claude_messages_attaches_document() {
        assert_attaches_document(live::claude_messages()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_generate_content_attaches_document() {
        assert_attaches_document(live::gemini_generate_content()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_interactions_attaches_document() {
        assert_attaches_document(live::gemini_interactions()).await;
    }
}
