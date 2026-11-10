use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{Candidate, Data, ModelStatus, Part, PromptFeedback, UsageMetadata};

/// Response from the model with generated content.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GenerateContentResponse {
    /// Candidate responses from the model.
    pub candidates: Option<Vec<Candidate>>,

    /// Returns the prompt's feedback related to the content filters.
    pub prompt_feedback: Option<PromptFeedback>,

    /// Output only. Metadata on the generation requests' token usage.
    pub usage_metadata: Option<UsageMetadata>,

    /// Output only. The model version used to generate the response.
    pub model_version: Option<String>,

    /// Output only. responseId is used to identify each response.
    pub response_id: Option<String>,

    /// Output only. The current model status of this model.
    pub model_status: Option<ModelStatus>,
}

impl GenerateContentResponse {
    /// Returns the text content from the first candidate's last text part, if available.
    ///
    /// This is a convenience method for simple text-only responses.
    /// For more complex responses with multiple parts or candidates, iterate over
    /// `candidates` directly.
    pub fn text(&self) -> Option<&str> {
        self.candidates
            .as_ref()?
            .first()?
            .content
            .as_ref()?
            .parts
            .as_ref()?
            .iter()
            .rfind(|part| part.as_text().is_some())?
            .as_text()
    }

    /// Returns all text content concatenated from the first candidate.
    ///
    /// This collects all text parts from the first candidate.
    pub fn all_text(&self) -> Option<Vec<String>> {
        let candidate = self.candidates.as_ref()?.first()?;
        let content = candidate.content.as_ref()?;

        let texts: Vec<String> = content
            .parts
            .as_ref()?
            .iter()
            .filter_map(|part| part.as_text().map(|s| s.to_string()))
            .collect();

        if texts.is_empty() { None } else { Some(texts) }
    }

    /// Returns all thought content concatenated from the first candidate.
    ///
    /// This collects all thought parts from the first candidate.
    pub fn all_thoughts(&self) -> Option<Vec<String>> {
        let candidate = self.candidates.as_ref()?.first()?;
        let content = candidate.content.as_ref()?;

        let thoughts: Vec<String> = content
            .parts
            .as_ref()?
            .iter()
            .filter_map(|part| part.as_thought().map(|s| s.to_string()))
            .collect();

        if thoughts.is_empty() {
            None
        } else {
            Some(thoughts)
        }
    }

    /// Returns the first candidate, if available.
    pub fn first_candidate(&self) -> Option<&Candidate> {
        self.candidates.as_ref()?.first()
    }

    /// Returns the finish reason from the first candidate, if available.
    pub fn finish_reason(&self) -> Option<&super::FinishReason> {
        self.first_candidate()?.finish_reason.as_ref()
    }

    /// Returns all parts from the first candidate.
    pub fn parts(&self) -> Option<&[Part]> {
        Some(self.first_candidate()?.content.as_ref()?.parts.as_ref()?)
    }

    /// Returns an iterator over function calls from the first candidate.
    pub fn function_calls(&self) -> impl Iterator<Item = &super::FunctionCall> {
        self.parts()
            .into_iter()
            .flatten()
            .filter_map(|part| part.as_function_call())
    }
}

impl Part {
    /// Returns the text content if this part contains text data.
    pub fn as_text(&self) -> Option<&str> {
        if self.thought.unwrap_or(false) {
            return None;
        }

        match &self.data {
            Data::Text(text) => Some(text.as_ref()),
            _ => None,
        }
    }

    /// Returns the thought text if this part is a thought from a thinking model.
    pub fn as_thought(&self) -> Option<&str> {
        if self.thought.unwrap_or(false) {
            match &self.data {
                Data::Text(text) => Some(text.as_ref()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Returns the function call if this part contains a function call.
    pub fn as_function_call(&self) -> Option<&super::FunctionCall> {
        match &self.data {
            Data::FunctionCall(fc) => Some(fc),
            _ => None,
        }
    }

    /// Returns the function response if this part contains a function response.
    pub fn as_function_response(&self) -> Option<&super::FunctionResponse> {
        match &self.data {
            Data::FunctionResponse(fr) => Some(fr),
            _ => None,
        }
    }

    /// Returns the inline data (blob) if this part contains inline data.
    pub fn as_inline_data(&self) -> Option<&super::Blob> {
        match &self.data {
            Data::InlineData(blob) => Some(blob),
            _ => None,
        }
    }

    /// Returns the file data if this part contains file data.
    pub fn as_file_data(&self) -> Option<&super::FileData> {
        match &self.data {
            Data::FileData(fd) => Some(fd),
            _ => None,
        }
    }

    /// Returns the executable code if this part contains executable code.
    pub fn as_executable_code(&self) -> Option<&super::ExecutableCode> {
        match &self.data {
            Data::ExecutableCode(ec) => Some(ec),
            _ => None,
        }
    }

    /// Returns the code execution result if this part contains one.
    pub fn as_code_execution_result(&self) -> Option<&super::CodeExecutionResult> {
        match &self.data {
            Data::CodeExecutionResult(cer) => Some(cer),
            _ => None,
        }
    }

    /// Returns true if this part is a thought from a thinking model.
    pub fn is_thought(&self) -> bool {
        self.thought.unwrap_or(false)
    }
}
