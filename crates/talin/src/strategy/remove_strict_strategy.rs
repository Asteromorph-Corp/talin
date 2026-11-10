use std::convert::Infallible;

use async_trait::async_trait;
use talin_provider::openai_chat::Config;
use talin_provider::openai_chat::types::{ChatCompletionTools, ResponseFormat};

use crate::chain::OutputCtor;
use crate::llm::{OpenAIChat, RequestOf};
use crate::strategy::Strategy;

/// A [`Strategy`] that strips the `strict` field from tool function definitions and JSON schema
/// response formats.
///
/// Most models require the `strict` field to be set for tool function definitions and JSON schema
/// response formats, but some models do not support it.
/// By default, this crate sets the `strict` field to `true`, but this strategy can be used to
/// remove it for models that do not support it.
///
/// Only compatible with [`OpenAIChat`].
///
/// No return value.
#[derive(Default, Clone)]
pub struct RemoveStrictStrategy;

#[async_trait]
impl<O: OutputCtor, C: Config + Clone + Send + Sync + 'static> Strategy<OpenAIChat<C>, O>
    for RemoveStrictStrategy
{
    type Output = ();
    type Error = Infallible;

    async fn process_request(
        &mut self,
        mut request: RequestOf<OpenAIChat<C>>,
    ) -> Result<RequestOf<OpenAIChat<C>>, Self::Error> {
        if let Some(response_format) = &mut request.inner.response_format
            && let ResponseFormat::JsonSchema { json_schema } = response_format
        {
            json_schema.strict = None;
        }

        if let Some(tools) = &mut request.inner.tools {
            for tool in tools.iter_mut() {
                let ChatCompletionTools::Function(function) = tool else {
                    continue;
                };
                function.function.strict = None;
            }
        }
        Ok(request)
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
