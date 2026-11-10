use std::sync::Arc;

use async_trait::async_trait;
use diffy::{Patch, apply as apply_patch};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::strategy::buffered_final_answer_strategy::FinalAnswerBuffer;
use crate::strategy::buffered_final_answer_strategy::hashline::{apply_hashline, hashline_view};
use crate::tools::{Function, Tool};

/// How the model edits the [accumulated answer](super::BufferedFinalAnswerStrategy).
///
/// Each variant exposes a single `edit_final_answer` tool; only the input format differs.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FinalAnswerEdit {
    /// Substring replacement: `{ old, new }` replaces every occurrence of `old` with `new`, or
    /// appends `new` when `old` is empty.
    Replace,
    /// Unified-diff patch: `{ patch }` applies one or more `@@` hunks to the buffer, located by
    /// their context.
    Patch,
    /// [Hashline](https://github.com/kebbbnnn/hashline) partial update: `{ start, end, content }`
    /// replaces the inclusive line range addressed by `linenum:hash` anchors. The buffer is
    /// presented with hashline anchors so the model can reference lines.
    Hashline,
}

impl FinalAnswerEdit {
    /// Renders the buffer as the model should see it: plain text, except in
    /// [`Hashline`](FinalAnswerEdit::Hashline) mode where each line carries a `linenum:hash`
    /// anchor.
    pub(super) fn render_buffer(&self, content: &str) -> String {
        match self {
            FinalAnswerEdit::Hashline => hashline_view(content),
            _ => content.to_owned(),
        }
    }

    /// Builds this mode's `edit_final_answer` tool over the shared buffer.
    pub(super) fn tool<B: FinalAnswerBuffer + 'static>(self, content: Arc<B>) -> Tool {
        match self {
            FinalAnswerEdit::Replace => Tool::from(ReplaceEdit(content)),
            FinalAnswerEdit::Patch => Tool::from(PatchEdit(content)),
            FinalAnswerEdit::Hashline => Tool::from(HashlineEdit(content)),
        }
    }
}

pub(super) const EDIT_FINAL_ANSWER: &str = "edit_final_answer";

/// Reports a successful edit back to the model.
fn updated(len: usize) -> String {
    format!("Updated answer ({len} chars).")
}

/// The input to the [`ReplaceEdit`] tool.
#[derive(Deserialize, JsonSchema)]
struct ReplaceInput {
    /// The text to replace. If empty, `new` is appended to the end of the answer.
    old: String,
    /// The text to insert in place of `old`.
    new: String,
}

/// Edits the buffer by substring replacement, or appends when `old` is empty.
struct ReplaceEdit<B>(Arc<B>);

#[async_trait]
impl<B: FinalAnswerBuffer + 'static> Function for ReplaceEdit<B> {
    type Input = ReplaceInput;
    type Output = String;

    fn name(&self) -> String {
        EDIT_FINAL_ANSWER.to_owned()
    }

    fn description(&self) -> String {
        "Edit the accumulated answer by replacing every occurrence of `old` with `new`. \
         Leave `old` empty to append `new` to the end."
            .to_owned()
    }

    async fn call(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
        let mut content = self.0.read().await?;
        if input.old.is_empty() {
            content.push_str(&input.new);
        } else if content.contains(&input.old) {
            content = content.replace(&input.old, &input.new);
        } else {
            return Err("`old` text was not found in the accumulated answer.".into());
        }
        let len = content.len();
        self.0.write(content).await?;
        Ok(updated(len))
    }
}

/// The input to the [`PatchEdit`] tool.
#[derive(Deserialize, JsonSchema)]
struct PatchInput {
    /// A unified diff to apply to the answer in the format:
    /// ```text
    /// @@ -start_line,start_length +end_line,end_length @@
    ///  context line
    /// -old line
    /// +new line
    /// ```
    patch: String,
}

/// Edits the buffer by applying a unified diff.
struct PatchEdit<B>(Arc<B>);

#[async_trait]
impl<B: FinalAnswerBuffer + 'static> Function for PatchEdit<B> {
    type Input = PatchInput;
    type Output = String;

    fn name(&self) -> String {
        EDIT_FINAL_ANSWER.to_owned()
    }

    fn description(&self) -> String {
        "Edit the accumulated answer by applying a unified diff.".to_owned()
    }

    async fn call(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
        let patch = Patch::from_str(&input.patch)?;
        let content = apply_patch(&self.0.read().await?, &patch)?;
        let len = content.len();
        self.0.write(content).await?;
        Ok(updated(len))
    }
}

/// The input to the [`HashlineEdit`] tool.
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
struct HashlineInput {
    /// The `linenum:hash` anchor of the first line to replace, e.g. `13:b7c2`.
    start: String,
    /// The `linenum:hash` anchor of the last line to replace (inclusive).
    end: String,
    /// The replacement text for the range.
    content: String,
}

/// Edits the buffer by replacing an inclusive line range addressed by hashline anchors.
struct HashlineEdit<B>(Arc<B>);

#[async_trait]
impl<B: FinalAnswerBuffer + 'static> Function for HashlineEdit<B> {
    type Input = HashlineInput;
    type Output = String;

    fn name(&self) -> String {
        EDIT_FINAL_ANSWER.to_owned()
    }

    fn description(&self) -> String {
        "Edit the accumulated answer by replacing inclusive range of lines.\
        Address the lines with `linenum:hash` anchors, taken from the hashline view of the answer."
            .to_owned()
    }

    async fn call(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
        let content = apply_hashline(
            &self.0.read().await?,
            &input.start,
            &input.end,
            &input.content,
        )?;
        let len = content.len();
        self.0.write(content).await?;
        Ok(updated(len))
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;

    use super::*;
    use crate::strategy::buffered_final_answer_strategy::InMemoryBuffer;

    fn shared(text: &str) -> Arc<InMemoryBuffer> {
        Arc::new(InMemoryBuffer(Mutex::new(text.to_owned())))
    }

    #[tokio::test]
    async fn replace_appends_when_old_empty() {
        let content = shared("hello");
        let tool = ReplaceEdit(Arc::clone(&content));

        tool.call(ReplaceInput {
            old: String::new(),
            new: " world".to_owned(),
        })
        .await
        .unwrap();

        assert_eq!(content.read().await.unwrap(), "hello world");
    }

    #[tokio::test]
    async fn replace_replaces_occurrences() {
        let content = shared("a a a");
        let tool = ReplaceEdit(Arc::clone(&content));

        tool.call(ReplaceInput {
            old: "a".to_owned(),
            new: "b".to_owned(),
        })
        .await
        .unwrap();

        assert_eq!(content.read().await.unwrap(), "b b b");
    }

    #[tokio::test]
    async fn replace_errors_on_missing_old() {
        let tool = ReplaceEdit(shared("hello"));

        let result = tool
            .call(ReplaceInput {
                old: "absent".to_owned(),
                new: "x".to_owned(),
            })
            .await;

        assert!(result.is_err());
    }
}
