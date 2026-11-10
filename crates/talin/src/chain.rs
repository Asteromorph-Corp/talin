//! A module for the [`Chain`] trait and related components.
//!
//! Chains are the core abstraction for composing LLM calls with other operations.
//! They allow you to build pipelines that transform inputs, call language models,
//! and process outputs in a structured way.
//!
//! # Key Types
//!
//! - [`Chain`]: The core trait for all chain implementations.
//! - [`LlmChain`]: A basic chain that formats a prompt and calls an LLM.
//!
//! # Type Constructors
//!
//! Due to Rust's lack of higher-kinded types, this module uses "constructor" traits
//! ([`Ctor`], [`InputCtor`], [`OutputCtor`]) to allow chains to work with types that
//! may contain borrowed data with varying lifetimes.

use async_trait::async_trait;

use crate::schemas::{IntoWithUsage, Usage, WithUsage};

mod chain_input;
mod chain_output;
mod ctor;
mod error;
mod get_prompt;
mod llm_chain;
mod sequential_chain;

pub use chain_input::{ChainInput, MediaReplacements, PlaceholderReplacements, TextReplacements};
pub use chain_output::{ChainOutput, Partial, PartialOutput, PartialWithInput};
pub use ctor::{Ctor, InputCtor, OutputCtor, PartialTarget, StrCtor, StringCtor};
pub use error::ChainError;
pub use get_prompt::GetPrompt;
pub use llm_chain::LlmChain;
pub use sequential_chain::{AddChain, SequentialChain};

/// The core trait for chain implementations.
///
/// A chain transforms an input of type `I::Target<'a>` into an output of type `O::Target<'a>`,
/// potentially calling an LLM or performing other operations along the way.
///
/// # Type Parameters
///
/// - `I`: A [constructor](crate::chain::Ctor) for the input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the output type.
///
/// # Example
///
/// ```ignore
/// let chain: LlmChain = LlmChain::new(prompt, llm);
/// let result = chain.call("What is the capital of France?").await?;
/// println!("{}", result.content);
/// ```
#[async_trait]
pub trait Chain<I, O>: Sync + Send
where
    I: InputCtor,
    O: OutputCtor,
{
    /// Executes the chain with the given input.
    ///
    /// Returns the output wrapped in [`WithUsage`] to include token usage information
    /// when available.
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError>;
}

/// Identity chain implementation for the unit type.
///
/// This implementation passes the input through unchanged, useful as a no-op
/// in chain composition or as a default chain.
#[async_trait]
impl<T> Chain<T, T> for ()
where
    T: InputCtor + OutputCtor,
    for<'any> T: OutputCtor<Target<'any> = <T as InputCtor>::Target<'any>>,
{
    async fn call<'a>(
        &self,
        input: <T as InputCtor>::Target<'a>,
    ) -> Result<WithUsage<<T as OutputCtor>::Target<'a>>, ChainError> {
        Ok(input.with_usage(Usage::default()))
    }
}
