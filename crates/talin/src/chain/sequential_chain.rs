use std::fmt::Debug;

use async_trait::async_trait;

use crate::chain::{Chain, ChainError, InputCtor, OutputCtor};
use crate::schemas::{IntoWithUsage, WithUsage};

/// A trait for adding a chain to create a sequential composition.
///
/// This trait enables the builder pattern for composing chains, allowing you
/// to connect chains together where the output of one becomes the input of the next.
///
/// # Example
///
/// ```ignore
/// use talin::chain::{AddChain, LlmChain};
///
/// let chain1: impl Chain<InitialCtor, IntermediateCtor> = SomeChain::new();
/// let chain2: impl Chain<IntermediateCtor, FinalCtor> = SomeOtherChain::new();
///
/// let combined_chain: impl Chain<InitialCtor, FinalCtor> = chain1.add_chain(chain2);
/// ```
///
/// # `sequential_chain!`
///
/// Use the [`sequential_chain!`](crate::sequential_chain) macro for convenient composition:
///
/// ```ignore
/// let combined_chain = sequential_chain!(chain1, chain2);
/// let result = chain.call(input).await?;
/// ```
pub trait AddChain<'a, I, M1, M2, O>: 'a
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
{
    /// Adds another chain to execute after this one.
    fn add_chain(self, chain: impl Chain<M2, O> + 'a) -> impl Chain<I, O> + 'a;
}

/// A chain that executes two chains in sequence.
///
/// The output of the first chain is converted to the input of the second chain,
/// allowing you to build pipelines of LLM operations.
///
/// # Type Parameters
///
/// - `I`: The input type of the first chain.
/// - `M1`: The output type of the first chain.
/// - `M2`: The input type of the second chain.
/// - `O`: The output type of the second chain.
///
/// # Example
///
/// Use the [`sequential_chain!`](crate::sequential_chain) macro for convenient composition:
///
/// ```ignore
/// let chain = sequential_chain!(chain1, chain2, chain3);
/// let result = chain.call(input).await?;
/// ```
pub struct SequentialChain<'a, I, M1, M2, O>
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
{
    /// The first chain to execute.
    first: Box<dyn Chain<I, M1> + 'a>,
    /// The second chain to execute with the output of the first.
    second: Box<dyn Chain<M2, O> + 'a>,
}

/// Implementation of [`AddChain`] for composing two chains sequentially.
impl<'a, I, M1, M2, O, Op> AddChain<'a, I, M1, M2, O> for Op
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
    Op: Chain<I, M1> + 'a,
    for<'any> M1::Target<'any>: Debug + Into<M2::Target<'any>>,
{
    fn add_chain(self, chain: impl Chain<M2, O> + 'a) -> impl Chain<I, O> + 'a {
        SequentialChain {
            first: Box::new(self),
            second: Box::new(chain),
        }
    }
}

/// Implementation of [`Chain`] for [`SequentialChain`].
///
/// This implementation executes the first chain, logs the intermediate result,
/// and then passes the output to the second chain, combining their token usage.
#[async_trait]
impl<I, M1, M2, O> Chain<I, O> for SequentialChain<'_, I, M1, M2, O>
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
    for<'any> M1::Target<'any>: Debug + Into<M2::Target<'any>>,
{
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let result1 = self.first.call(input).await?;
        log::debug!(
            "Intermediate result for sequential chain: {:?}",
            result1.content
        );
        let result2 = self.second.call(result1.content.into()).await?;

        let usage = result1.usage.merge(&result2.usage);
        Ok(result2.content.with_usage(usage))
    }
}

/// Macro for composing multiple chains into a sequential pipeline.
///
/// This macro provides a convenient way to chain multiple operations together,
/// where the output of each chain becomes the input of the next.
///
/// # Example
///
/// ```ignore
/// let chain = sequential_chain!(
///     name_generator_chain,
///     slogan_generator_chain,
///     formatter_chain,
/// );
/// let result = chain.call(input).await?;
/// ```
#[macro_export]
macro_rules! sequential_chain {
    () => {
        $crate::chain::EmptyChain
    };
    ( $first:expr $(, $chain:expr )* $(,)? ) => {
        {
            let chain = $first;
            $(
                let chain = $crate::chain::AddChain::add_chain(chain, $chain);
            )*
            chain
        }
    };
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use serde::Serialize;

    use super::*;
    use crate::chain::{ChainInput, ChainOutput, Ctor, LlmChain};
    use crate::llm::OpenAIChat;
    use crate::template::{MessageTemplate, TemplateRole};

    #[tokio::test]
    #[ignore]
    async fn test_sequential() {
        #[derive(Debug, Clone, ChainInput, Ctor)]
        struct Chain1Input<'a> {
            #[chain(into = "text")]
            input: &'a str,
            palabra: &'a str,
        }
        #[derive(Debug, Clone, Serialize, ChainInput, ChainOutput, Ctor)]
        #[chain(from_input = Chain1Input<'a>)]
        struct Chain2Input<'a> {
            #[chain(from = response_json, into = "text")]
            nombre: Cow<'a, str>,
            #[chain(from = input, into = "text")]
            palabra: Cow<'a, str>,
        }

        let llm = OpenAIChat::default();
        let prompt1 = MessageTemplate::fstring("dame un nombre para una tienda de {input}")
            .role(TemplateRole::User);
        let prompt2 = MessageTemplate::fstring(
            "dame un slogan para una tienda llamada {nombre}, tiene que incluir la palabra {palabra}",
        )
        .role(TemplateRole::User);

        let chain1: LlmChain<_, Chain1InputCtor, Chain2InputCtor> =
            LlmChain::new(prompt1, llm.clone());
        let chain2: LlmChain<_, Chain2InputCtor> = LlmChain::new(prompt2, llm.clone());

        let chain = sequential_chain!(chain1, chain2);
        let result = chain
            .call(Chain1Input {
                input: "medias",
                palabra: "arroz",
            })
            .await;
        assert!(
            result.is_ok(),
            "Expected `chain.call` to succeed, but it failed with error: {:?}",
            result.err()
        );

        if let Ok(output) = result {
            println!("{output:?}");
        }
    }
}
