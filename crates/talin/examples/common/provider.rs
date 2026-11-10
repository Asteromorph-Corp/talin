use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum Provider {
    GenericChat,
    OpenaiChat,
    ClaudeMessages,
    GeminiInteractions,
    GeminiGenerateContent,
    OpenaiResponses,
    XaiResponses,
}
