//! Feeds an agent a turn that mixes text with media.
//!
//! The agent's user template declares media slots, and the input type fills them: each
//! `#[chain(into = "...")]` media field is published under its own slot name, so the
//! rendered user message carries the question text followed by the attached parts.

use talin::agent::Agent;
use talin::chain::{ChainInput, Ctor};
use talin::llm::ClaudeMessages;
use talin::schemas::{DocumentContent, DocumentMimeType, ImageContent, ImageMimeType, MediaSource};
use talin::template::MessageTemplate;

/// The agent input: the question text plus the media filling the template's slots.
#[derive(ChainInput, Ctor)]
pub struct Review<'a> {
    #[chain(into = "text")]
    question: &'a str,
    #[chain(into = "images")]
    screenshots: Vec<ImageContent>,
    /// Optional media fields are allowed; `None` simply leaves the slot empty.
    #[chain(into = "documents")]
    pub spec: Option<Vec<DocumentContent>>,
}

const PROMPT: &str = "Does the screenshot match the layout the spec describes?";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    // `{question}` is a text placeholder, and `screenshots` and `spec` are media slots.
    let user_message =
        MessageTemplate::fstring("{question}").with_media(["screenshots".into(), "spec".into()]);

    let llm = ClaudeMessages::builder().build();
    let agent: Agent<_, ReviewCtor> = Agent::builder().user_message(user_message).build(llm);

    // Media can come from raw bytes, a provider file id, or a URL, as here.
    let screenshot =
        MediaSource::from_url("https://example.com/dashboard.png").image(ImageMimeType::Png);
    let spec = MediaSource::from_url("https://example.com/layout-spec.pdf")
        .document(DocumentMimeType::Pdf);

    let input = Review {
        question: PROMPT,
        screenshots: vec![screenshot],
        spec: Some(vec![spec]),
    };

    match agent.run(input).await {
        Ok(answer) => println!("Result: {answer}"),
        Err(e) => eprintln!("Error running the agent: {e:?}"),
    }
}
