use std::convert::Infallible;

use async_trait::async_trait;

use crate::chain::OutputCtor;
use crate::llm::Llm;
use crate::strategy::Strategy;
use crate::tools::ToolResolver;

/// A [`Strategy`] that injects tools from a [`ToolResolver`] into the run.
pub struct ToolResolverStrategy<E>
where
    E: Send + Sync,
    for<'a> &'a E: ToolResolver<'a>,
{
    resolver: E,
}

#[async_trait]
impl<L, O, E> Strategy<L, O> for ToolResolverStrategy<E>
where
    E: Send + Sync,
    for<'a> &'a E: ToolResolver<'a>,
    L: Llm,
    O: OutputCtor,
{
    type Output = ();
    type Error = Infallible;

    fn additional_tools<'a>(&'a self) -> impl ToolResolver<'a> + 'a {
        &self.resolver
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
