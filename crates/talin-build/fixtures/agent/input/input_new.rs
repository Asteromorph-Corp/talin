impl<'a> MyAgentInput<'a> {
    /// Create a new instance of the input struct.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audio: impl Into<std::borrow::Cow<'a, [::talin::schemas::AudioContent]>>,
        document: impl Into<std::borrow::Cow<'a, [::talin::schemas::DocumentContent]>>,
        image: impl Into<std::borrow::Cow<'a, [::talin::schemas::ImageContent]>>,
        input: impl Into<String>,
        video: impl Into<std::borrow::Cow<'a, [::talin::schemas::VideoContent]>>,
    ) -> Self {
        Self {
            _env_my_env_var: std::env::var("MY_ENV_VAR")
                .unwrap_or_else(|e| panic!("Failed to get env var '{}': {}", "MY_ENV_VAR", e)),
            _preset_current_date: ::talin::__private::chrono::Local::now()
                .format("%Y-%m-%d")
                .to_string(),
            audio: audio.into(),
            document: document.into(),
            image: image.into(),
            input: input.into(),
            video: video.into(),
        }
    }
}
