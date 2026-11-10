#[derive(
    Debug,
    Default,
    ::talin::__private::serde::Serialize,
    ::talin::__private::serde::Deserialize,
    ::talin::__private::schemars::JsonSchema,
)]
#[serde(crate = "::talin::__private::serde")]
#[schemars(crate = "::talin::__private::schemars")]
#[serde(deny_unknown_fields)]
pub struct MyToolInput {
    #[schemars(with = "i32")]
    pub limit: u8,
    ///The search query.
    pub query: String,
}
