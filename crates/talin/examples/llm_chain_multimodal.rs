//! An `LlmChain` fed a turn that mixes text with media.
//!
//! The user template declares media slots, and the input type fills them with images and
//! documents.

use talin::chain::{Chain, LlmChain};
use talin::llm::ClaudeMessages;
use talin::prompt_template;
use talin::schemas::{DocumentContent, DocumentMimeType, ImageContent, ImageMimeType, MediaSource};
use talin::template::{MessageTemplate, TemplateRole};
use talin_macros::{ChainInput, Ctor};

/// The agent input: the question text plus the media filling the template's slots.
#[derive(ChainInput, Ctor)]
pub struct Review<'a> {
    /// The question text.
    #[chain(into = "text")]
    pub question: &'a str,
    /// Screenshots attached to the question.
    #[chain(into = "images")]
    pub screenshots: Vec<ImageContent>,
    /// Optional media fields are allowed; `None` simply leaves the slot empty.
    #[chain(into = "documents")]
    pub spec: Option<Vec<DocumentContent>>,
}

const PROMPT: &str = "Does the screenshot match the layout the spec describes?";

#[tokio::main]
async fn main() {
    // `{question}` is a text placeholder, and `screenshots` and `spec` are media slots.
    let user_message = MessageTemplate::fstring("{question}")
        .with_media(["screenshots".into(), "spec".into()])
        .role(TemplateRole::User);

    let prompt = prompt_template![user_message];
    let llm = ClaudeMessages::builder().build();
    let chain: LlmChain<_, ReviewCtor> = LlmChain::new(prompt, llm);

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

    match chain.call(input).await {
        Ok(output) => println!("Result: {:?}", output.content),
        Err(e) => panic!("Error invoking LlmChain: {e:?}"),
    }
}
