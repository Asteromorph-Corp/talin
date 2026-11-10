/// Asks the model to give its best final answer, ignoring all previous instructions and stopping
/// the use of any tools.
pub(crate) const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. \
You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

/// Asks the model to finish an output it ran out of tokens to write.
pub(crate) const CONTINUE_PARTIAL: &str = "Your previous response was cut off because it reached the output token limit. \
Continue from exactly where you left off. Do not repeat anything you have already written, and do not acknowledge this message.";
