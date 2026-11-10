#[derive(::talin::chain::ChainInput, ::talin::chain::Ctor)]
#[serde(crate = "::talin::__private::serde")]
#[chain(crate = "::talin")]
pub struct MyAgentInput<'a> {
    #[chain(into = "text")]
    pub _env_my_env_var: String,
    #[chain(into = "text")]
    pub _preset_current_date: String,
    #[chain(into = "media")]
    pub audio: std::borrow::Cow<'a, [::talin::schemas::AudioContent]>,
    #[chain(into = "media")]
    pub document: std::borrow::Cow<'a, [::talin::schemas::DocumentContent]>,
    #[chain(into = "media")]
    pub image: std::borrow::Cow<'a, [::talin::schemas::ImageContent]>,
    #[chain(into = "text")]
    pub input: String,
    #[chain(into = "media")]
    pub video: std::borrow::Cow<'a, [::talin::schemas::VideoContent]>,
}
