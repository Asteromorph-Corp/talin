use async_trait::async_trait;

use crate::chain::{
    Chain, ChainError, GetPrompt, InputCtor, OutputCtor, PartialOutput, PartialWithInput, StrCtor,
    StringCtor,
};
use crate::llm::{Llm, LlmError, LlmExt, LlmRequestOptions};
use crate::schemas::{IntoWithUsage, LlmStep, Prompt, WithUsage};
use crate::template::{PromptTemplate, TemplateError};

/// A chain that formats a prompt template and calls an LLM.
///
/// # Type Parameters
///
/// - `I`: A [constructor](crate::chain::Ctor) for the input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the output type.
///
/// # Example
///
/// ```ignore
/// use talin::chain::{Chain, LlmChain};
/// use talin::llm::OpenAIChat;
/// use talin::template::{MessageTemplate, TemplateRole};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let prompt = MessageTemplate::fstring("Tell me about {input}")
///         .role(TemplateRole::User);
///     let llm = OpenAIChat::default();
///     let chain: LlmChain = LlmChain::new(prompt, llm);
///
///     let result = chain.call("rust programming").await?;
///     println!("{}", result.content);
///     Ok(())
/// }
/// ```
pub struct LlmChain<L, I = StrCtor, O = StringCtor>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
{
    /// The prompt template used to format inputs.
    pub(super) prompt: PromptTemplate,
    /// The LLM used by the chain.
    pub(super) llm: L,
    pub(super) _phantom: std::marker::PhantomData<(I, O)>,
}

impl<L, I, O> LlmChain<L, I, O>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
{
    /// Creates a new `LlmChain` with the given prompt template and LLM.
    pub fn new(prompt: impl Into<PromptTemplate>, llm: L) -> Self {
        Self {
            prompt: prompt.into(),
            llm,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Implementation of [`Chain`] for [`LlmChain`].
///
/// The chain uses the input to format the prompt template and calls the LLM.
#[async_trait]
impl<L, I, O> Chain<I, O> for LlmChain<L, I, O>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Partial: PartialWithInput<I::Target<'any>, O::Target<'any>>,
{
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let prompt = self.prompt.format(&input)?;
        let WithUsage { content, usage } = self
            .llm
            .generate(
                prompt,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                L::Options::from_default(),
            )
            .await?;

        log::trace!("\nLLM output:\n{content}");
        if !usage.is_empty() {
            log::trace!("\nToken usage:\n{usage}");
        }

        let partial: O::Partial = match content {
            LlmStep::Finish(m) => O::Partial::from_parts(m.content)?,
            LlmStep::ToolCall(_) => {
                return Err(LlmError::invalid_response("unexpected tool call").into());
            }
        };
        let content: O::Target<'_> = partial.with_input(input);
        Ok(content.with_usage(usage))
    }
}

impl<L, I, O> GetPrompt<&I::Target<'_>> for LlmChain<L, I, O>
where
    L: Llm,
    I: InputCtor,
    O: OutputCtor,
{
    fn get_prompt(&self, input: &I::Target<'_>) -> Result<Prompt, TemplateError> {
        let prompt = self.prompt.format(input)?;
        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use talin_provider::openai_chat::OpenAIConfig;

    use super::*;
    use crate::chain::{Chain, ChainInput, Ctor};
    use crate::llm::{GenericChat, OpenAIModel};
    use crate::prompt_template;
    use crate::template::{MessageTemplate, TemplateRole};

    #[tokio::test]
    #[ignore]
    async fn test_invoke_chain() {
        #[derive(Clone, ChainInput, Ctor)]
        #[allow(dead_code)]
        struct NombreInput<'a> {
            #[chain(into = "text")]
            pub nombre: &'a str,
        }

        let input = NombreInput { nombre: "Juan" };

        // Create an AI message prompt template
        let human_message_prompt =
            MessageTemplate::fstring("Mi nombre es: {nombre} ").role(TemplateRole::User);

        // Use the `message_formatter` macro to construct the formatter
        let prompt = prompt_template!(human_message_prompt);

        let llm: GenericChat<OpenAIConfig> = GenericChat::builder()
            .with_model(OpenAIModel::Gpt35)
            .build();
        let chain: LlmChain<GenericChat, NombreInputCtor> = LlmChain::new(prompt, llm);

        // Execute `chain.invoke` and assert that it should succeed
        let result = chain.call(input).await;
        assert!(
            result.is_ok(),
            "Error invoking LlmChain: {:?}",
            result.err()
        )
    }
}
