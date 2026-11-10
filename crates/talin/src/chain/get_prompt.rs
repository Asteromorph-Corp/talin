use crate::schemas::Prompt;
use crate::template::TemplateError;

/// A trait for chains that can render their prompt for a given input.
///
/// This is useful for debugging and inspecting the actual prompt that will
/// be sent to the LLM before executing the chain.
///
/// # Example
///
/// ```ignore
/// let chain: LlmChain = LlmChain::new(prompt, llm);
/// let rendered_prompt = chain.get_prompt(&input)?;
/// println!("Prompt: {:?}", rendered_prompt);
/// ```
pub trait GetPrompt<I> {
    /// Renders the prompt for the given input without calling the LLM.
    ///
    /// This allows you to inspect what prompt would be sent to the LLM
    /// for debugging or logging purposes.
    fn get_prompt(&self, input: I) -> Result<Prompt, TemplateError>;
}
