//! Provider inspection uses native types. This observer never changes a request or transcript.
use talin::llm::claude_messages::types::{
    CompactionBlockParam, ContentBlock, ContentBlockParam, TextOrItems, Usage,
};
use talin::llm::openai_responses::types::{
    CompactionSummaryItemParam, InputItem, InputParam, Item, ResponseUsage,
};
use talin::llm::{ClaudeMessages, Llm, OpenAIResponses};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Compaction {
    OpenAI(CompactionSummaryItemParam),
    Claude(CompactionBlockParam),
}

impl Compaction {
    /// Compare the complete native payload, including opaque fields.
    pub fn wire(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Serialize)]
pub enum TokenUsage {
    OpenAI(Option<ResponseUsage>),
    Claude(Usage),
}

pub trait Evidence: Llm {
    fn request_compactions(request: &Self::Request) -> Vec<Compaction>;
    fn response_compactions(response: &Self::Response) -> Vec<Compaction>;
    fn response_usage(response: &Self::Response) -> TokenUsage;
}

impl Evidence for OpenAIResponses {
    fn request_compactions(request: &Self::Request) -> Vec<Compaction> {
        match &request.input {
            InputParam::Items(items) => items
                .iter()
                .filter_map(|item| match item {
                    InputItem::Item(Item::Compaction(block)) => {
                        Some(Compaction::OpenAI(block.clone()))
                    }
                    _ => None,
                })
                .collect(),
            InputParam::Text(_) => Vec::new(),
        }
    }

    fn response_compactions(response: &Self::Response) -> Vec<Compaction> {
        response
            .output
            .iter()
            .filter_map(|item| {
                let native = item.clone().into();
                match native {
                    InputItem::Item(Item::Compaction(block)) => Some(Compaction::OpenAI(block)),
                    _ => None,
                }
            })
            .collect()
    }

    fn response_usage(response: &Self::Response) -> TokenUsage {
        TokenUsage::OpenAI(response.usage.clone())
    }
}

impl Evidence for ClaudeMessages {
    fn request_compactions(request: &Self::Request) -> Vec<Compaction> {
        request
            .messages
            .iter()
            .flat_map(|message| match &message.content() {
                Some(TextOrItems::Items(items)) => items
                    .iter()
                    .filter_map(|item| match item {
                        ContentBlockParam::Compaction(block) => {
                            Some(Compaction::Claude(block.clone()))
                        }
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn response_compactions(response: &Self::Response) -> Vec<Compaction> {
        response
            .content
            .iter()
            .filter_map(|item| match item {
                ContentBlock::Compaction(block)
                    if block.content.is_some() || block.encrypted_content.is_some() =>
                {
                    let native = block.clone().into();
                    Some(Compaction::Claude(native))
                }
                _ => None,
            })
            .collect()
    }

    fn response_usage(response: &Self::Response) -> TokenUsage {
        TokenUsage::Claude(response.usage.clone())
    }
}
