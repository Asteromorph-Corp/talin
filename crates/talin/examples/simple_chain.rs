//! A minimal `LlmChain` that asks for a store name given a product read from the terminal.

use std::borrow::Cow;
use std::io::{self, Write};

use talin::chain::{Chain, ChainInput, Ctor, LlmChain};
use talin::llm::{OpenAIChat, OpenAIModel};
use talin::template::{MessageTemplate, TemplateRole}; // Include io Library for terminal input

/// The chain input: the product to name a store after.
#[derive(Clone, ChainInput, Ctor)]
pub struct ProductInput<'a> {
    /// The product the store sells.
    #[chain(into = "text")]
    pub product: Cow<'a, str>,
}

#[tokio::main]
async fn main() {
    let prompt =
        MessageTemplate::jinja2("Give me a creative name for a store that sells: {{product}}")
            .role(TemplateRole::User);

    let llm = OpenAIChat::builder().with_model(OpenAIModel::Gpt35).build();
    let chain: LlmChain<_, ProductInputCtor> = LlmChain::new(prompt, llm);

    print!("Please enter a product: ");
    io::stdout().flush().unwrap(); // Display prompt to terminal

    let mut product = String::new();
    io::stdin().read_line(&mut product).unwrap(); // Get product from terminal input

    let product = product.trim();

    let output = chain
        .call(ProductInput {
            product: product.into(),
        }) // Use product input here
        .await
        .unwrap();

    println!("Output: {}", output.content);
}
