//! Chains two LLM calls: the first generates a store name, and the second writes a slogan for it.

#![allow(missing_docs)] // `ChainOutput` generates a public partial type with undocumented fields.

use std::io::{self, Write};

use serde::Serialize;
use talin::chain::{Chain, LlmChain};
use talin::llm::{OpenAIChat, OpenAIModel};
use talin::sequential_chain;
use talin::template::{MessageTemplate, TemplateRole};
use talin_macros::{ChainInput, ChainOutput, Ctor}; // Include io Library for terminal input

/// Input of the first chain: the product to name a store after.
#[derive(Debug, Clone, ChainInput, Ctor)]
pub struct GetNameInput<'a> {
    /// The product the store sells.
    #[chain(into = "text")]
    product: &'a str,
}

/// Output of the first chain: the generated store name.
#[derive(Debug, Clone, Serialize, ChainInput, ChainOutput, Ctor)]
#[chain(from_input = GetNameInput<'_>)]
pub struct GetNameOutput {
    /// The generated store name.
    #[chain(from = response_json, into = "text")]
    name: String,
}

/// The second chain consumes the first chain's output as its input.
pub type GetSloganInputCtor = GetNameOutputCtor;

/// Output of the second chain: the store name together with its slogan.
#[derive(Debug, Clone, Serialize, ChainOutput, Ctor)]
#[chain(from_input = GetNameOutput)]
pub struct GetSloganOutput {
    /// The store name carried over from the input.
    #[chain(from = input)]
    name: String,
    /// The generated slogan.
    #[chain(from = response_json)]
    slogan: String,
}

#[tokio::main]
async fn main() {
    let llm = OpenAIChat::builder().with_model(OpenAIModel::Gpt35).build();

    let prompt =
        MessageTemplate::jinja2("Give me a creative name for a store that sells: {{product}}")
            .role(TemplateRole::User);
    let get_name_chain: LlmChain<_, GetNameInputCtor, GetNameOutputCtor> =
        LlmChain::new(prompt, llm.clone());

    let prompt = MessageTemplate::jinja2("Give me a slogan for the following name: {{name}}")
        .role(TemplateRole::User);
    let get_slogan_chain: LlmChain<_, GetSloganInputCtor, GetSloganOutputCtor> =
        LlmChain::new(prompt, llm);

    let sequential_chain = sequential_chain!(get_name_chain, get_slogan_chain);

    print!("Please enter a product: ");
    io::stdout().flush().unwrap(); // Display prompt to terminal

    let mut product = String::new();
    io::stdin().read_line(&mut product).unwrap(); // Get product from terminal input

    let product = product.trim();
    let input = GetNameInput { product };
    let output = sequential_chain.call(input).await.unwrap();

    let content = output.content;

    println!("Name: {}", content.name);
    println!("Slogan: {}", content.slogan);
}
