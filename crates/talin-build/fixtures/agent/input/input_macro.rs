#[macro_export]
macro_rules! my_agent_input {
    (@value $_field:ident, $val:expr) => {
        $val
    };
    (@value $field:ident) => {
        $field
    };
    ($($field:ident $(: $val:expr)?),* $()?) => {
        MyAgentInput::new($($field : my_agent_input!(@ value $field $(, $val)?))*)
    };
}
