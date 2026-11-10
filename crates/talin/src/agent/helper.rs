/// Helper macro to call a strategy method and map error into an `AgentError::StrategyError`.
macro_rules! use_strategy {
    ($strategy:expr, $method:ident ( $($args:tt)* )) => {
        $strategy
            .$method($($args)*)
            .await
            .map_err(|e| AgentError::Strategy(stringify!($method), Box::new(e)))
    };
}
pub(super) use use_strategy;
