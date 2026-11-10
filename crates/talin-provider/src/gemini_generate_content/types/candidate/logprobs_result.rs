use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Logprobs Result.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LogprobsResult {
    /// Length = total number of decoding steps.
    pub top_candidates: Option<Vec<TopCandidates>>,

    /// Length = total number of decoding steps.
    pub chosen_candidates: Option<Vec<LogprobsCandidate>>,

    /// Sum of log probabilities for all tokens.
    pub log_probability_sum: Option<f64>,
}

/// Candidates with top log probabilities at each decoding step.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TopCandidates {
    /// Sorted by log probability in descending order.
    pub candidates: Option<Vec<LogprobsCandidate>>,
}

/// Candidate for the logprobs token and score.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LogprobsCandidate {
    /// The candidate's token string value.
    pub token: Option<String>,

    /// The candidate's token id value.
    pub token_id: Option<u32>,

    /// The candidate's log probability.
    pub log_probability: Option<f64>,
}
