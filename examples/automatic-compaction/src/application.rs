use talin::llm::{ClaudeMessages, Llm, LlmPreset, OpenAIResponses};

use crate::agents::{RecallAgent, RememberAgent};
use crate::config::Config;
use crate::facts::Recall;
use crate::scenario::{self, Outcome};

pub async fn openai(
    llm: OpenAIResponses,
    config: Config,
    facts: Recall,
    followups: usize,
) -> Outcome {
    let trace = scenario::trace(
        llm.model_key().model,
        &config.openai,
        config.openai.compaction.threshold,
        &facts,
        &config.archive,
    );
    let strategy = (config.openai.compaction.clone(), trace.clone());
    let remember = RememberAgent::new(LlmPreset::new(llm.clone(), strategy.clone())).await;
    let recall = RecallAgent::new(LlmPreset::new(llm, strategy)).await;
    scenario::run(
        remember,
        recall,
        config.openai,
        facts,
        config.archive,
        trace,
        followups,
    )
    .await
}

pub async fn claude(
    llm: ClaudeMessages,
    config: Config,
    facts: Recall,
    followups: usize,
) -> Outcome {
    let trace = scenario::trace(
        llm.model_key().model,
        &config.claude,
        config.claude.compaction.threshold,
        &facts,
        &config.archive,
    );
    let strategy = (config.claude.compaction.clone(), trace.clone());
    let remember = RememberAgent::new(LlmPreset::new(llm.clone(), strategy.clone())).await;
    let recall = RecallAgent::new(LlmPreset::new(llm, strategy)).await;
    scenario::run(
        remember,
        recall,
        config.claude,
        facts,
        config.archive,
        trace,
        followups,
    )
    .await
}
