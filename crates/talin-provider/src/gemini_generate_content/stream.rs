use futures::StreamExt;

use crate::error::StreamError;
use crate::gemini_generate_content::types::{
    ApiErrorDetails, Blob, Candidate, CitationMetadata, Data, GenerateContentResponse,
    GroundingMetadata, LogprobsResult, Part, RetrievalMetadata, UrlContextMetadata, UsageMetadata,
};
use crate::stream::{Merge, PartialResponse, PreferNew, PreferOld, ResponseStream, or_partial};

/// Assemble a complete [`GenerateContentResponse`] from a stream of partial
/// [`GenerateContentResponse`]s.
pub(super) async fn assemble(
    mut stream: ResponseStream<GenerateContentResponse, ApiErrorDetails>,
) -> Result<GenerateContentResponse, PartialResponse<GenerateContentResponse, ApiErrorDetails>> {
    let mut accumulated: Option<GenerateContentResponse> = None;

    while let Some(result) = stream.next().await {
        let chunk = or_partial!(result, ?accumulated);
        accumulated.merge(chunk);
    }

    accumulated.ok_or_else(|| PartialResponse::new(None, StreamError::UnexpectedClose))
}

/// [`GenerateContentResponse`] are merged by merging the fields.
impl Merge<Self> for GenerateContentResponse {
    fn merge(&mut self, rhs: Self) {
        // Merge candidates by index
        if let Some(rhs_candidates) = rhs.candidates {
            if let Some(candidates) = &mut self.candidates {
                for (rhs_idx, rhs_candidate) in rhs_candidates.into_iter().enumerate() {
                    let idx = rhs_candidate.index.map(|i| i as usize).unwrap_or(rhs_idx);

                    // Extend candidates array if needed
                    while candidates.len() <= idx {
                        candidates.push(Candidate {
                            content: None,
                            finish_reason: None,
                            safety_ratings: None,
                            citation_metadata: None,
                            token_count: None,
                            grounding_attributions: None,
                            grounding_metadata: None,
                            avg_logprobs: None,
                            logprobs_result: None,
                            url_context_metadata: None,
                            index: None,
                            finish_message: None,
                        });
                    }
                    candidates[idx].merge(rhs_candidate);
                }
            } else {
                self.candidates = Some(rhs_candidates);
            }
        }

        // Merge usage metadata
        self.usage_metadata.merge(rhs.usage_metadata);
        self.model_version.merge(PreferOld(rhs.model_version));
        self.prompt_feedback.merge(PreferOld(rhs.prompt_feedback));
        self.response_id.merge(PreferNew(rhs.response_id));
    }
}

/// Merges two [`Candidate`] values by merging their fields appropriately.
impl Merge<Self> for Candidate {
    fn merge(&mut self, rhs: Self) {
        // Merge content
        if let Some(rhs_content) = rhs.content {
            if let Some(content) = &mut self.content
                && let Some(parts) = &mut content.parts
                && let Some(rhs_parts) = rhs_content.parts
            {
                for rhs_part in rhs_parts {
                    if let Some(last_part) = parts.last_mut()
                        && is_mergeable(last_part, &rhs_part)
                    {
                        last_part.data.merge(rhs_part.data);
                    } else {
                        parts.push(rhs_part);
                    }
                }
            } else {
                self.content = Some(rhs_content);
            }
        }

        self.finish_reason.merge(PreferNew(rhs.finish_reason));
        self.safety_ratings.merge(PreferNew(rhs.safety_ratings));
        self.citation_metadata.merge(rhs.citation_metadata);
        self.token_count.merge(rhs.token_count);
        self.grounding_attributions
            .merge(rhs.grounding_attributions);
        self.grounding_metadata.merge(rhs.grounding_metadata);
        self.avg_logprobs.merge(PreferNew(rhs.avg_logprobs));
        self.logprobs_result.merge(rhs.logprobs_result);
        self.url_context_metadata.merge(rhs.url_context_metadata);
        self.index.merge(PreferNew(rhs.index));
        self.finish_message.merge(PreferNew(rhs.finish_message));
    }
}

/// [`Data`] is merged only when both values are of the same type.
impl Merge<Self> for Data {
    fn merge(&mut self, rhs: Self) {
        match (self, rhs) {
            (Data::Text(text), Data::Text(rhs_text)) => text.merge(rhs_text),
            (Data::InlineData(blob), Data::InlineData(rhs_blob)) => blob.merge(rhs_blob),
            // Do not merge other data types.
            _ => {}
        }
    }
}

/// [`Blob`]s are merged only when both values have the same MIME type.
impl Merge<Self> for Blob {
    fn merge(&mut self, rhs: Self) {
        if rhs.mime_type == self.mime_type {
            self.data.extend(rhs.data);
        }
    }
}

impl Merge<Self> for CitationMetadata {
    fn merge(&mut self, rhs: Self) {
        self.citation_sources.merge(rhs.citation_sources);
    }
}

impl Merge<Self> for GroundingMetadata {
    fn merge(&mut self, rhs: Self) {
        self.grounding_chunks.merge(rhs.grounding_chunks);
        self.grounding_supports.merge(rhs.grounding_supports);
        self.web_search_queries.merge(rhs.web_search_queries);
        self.search_entry_point
            .merge(PreferNew(rhs.search_entry_point));
        self.retrieval_metadata.merge(rhs.retrieval_metadata);
        self.google_maps_widget_context_token
            .merge(PreferNew(rhs.google_maps_widget_context_token));
    }
}

impl Merge<Self> for RetrievalMetadata {
    fn merge(&mut self, rhs: Self) {
        self.google_search_dynamic_retrieval_score
            .merge(PreferNew(rhs.google_search_dynamic_retrieval_score));
    }
}

impl Merge<Self> for LogprobsResult {
    fn merge(&mut self, rhs: Self) {
        self.top_candidates.merge(rhs.top_candidates);
        self.chosen_candidates.merge(rhs.chosen_candidates);
        self.log_probability_sum.merge(rhs.log_probability_sum);
    }
}

/// Merges two [`UsageMetadata`] values by summing token counts and merging token details.
impl Merge<Self> for UsageMetadata {
    fn merge(&mut self, rhs: Self) {
        // Sum up token counts using Merge<u32> trait
        self.prompt_token_count.merge(rhs.prompt_token_count);
        self.cached_content_token_count
            .merge(rhs.cached_content_token_count);
        self.candidates_token_count
            .merge(rhs.candidates_token_count);
        self.tool_use_prompt_token_count
            .merge(rhs.tool_use_prompt_token_count);
        self.thoughts_token_count.merge(rhs.thoughts_token_count);
        self.total_token_count.merge(rhs.total_token_count);

        // Merge token details
        self.prompt_tokens_details.merge(rhs.prompt_tokens_details);
        self.cache_tokens_details.merge(rhs.cache_tokens_details);
        self.candidates_tokens_details
            .merge(rhs.candidates_tokens_details);
        self.tool_use_prompt_tokens_details
            .merge(rhs.tool_use_prompt_tokens_details);
    }
}

impl Merge<Self> for UrlContextMetadata {
    fn merge(&mut self, rhs: Self) {
        self.url_metadata.extend(rhs.url_metadata);
    }
}

/// Checks if two [`Part`]s are mergeable.
fn is_mergeable(part: &Part, other: &Part) -> bool {
    !part.thought.unwrap_or(false)
        && !other.thought.unwrap_or(false)
        && part.thought_signature == other.thought_signature
        && part.part_metadata == other.part_metadata
}
