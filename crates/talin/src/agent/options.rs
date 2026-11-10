/// Options for the [`Agent`](crate::agent::Agent)
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct AgentOptions {
    /// Max iterations allowed.
    pub max_iterations: Option<usize>,
    /// Max number of consecutive failures allowed.
    pub max_consecutive_fails: Option<usize>,
    /// Max number of continuation requests allowed per turn.
    pub max_continuations: Option<usize>,
}

impl AgentOptions {
    /// Constructs a new [`AgentOptions`]
    pub fn new(max_iterations: Option<usize>, max_consecutive_fails: Option<usize>) -> Self {
        Self {
            max_iterations,
            max_consecutive_fails,
            ..Self::default()
        }
    }

    /// Sets the max number of continuation requests allowed per turn.
    pub fn with_max_continuations(mut self, max_continuations: usize) -> Self {
        self.max_continuations = Some(max_continuations);
        self
    }

    /// Disables the continuation limit check.
    pub fn without_max_continuations(mut self) -> Self {
        self.max_continuations = None;
        self
    }

    /// Sets the max iterations allowed.
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    /// Disables the max iteration check.
    pub fn without_max_iterations(mut self) -> Self {
        self.max_iterations = None;
        self
    }

    /// Sets the max number of consecutive failures allowed.
    pub fn with_max_consecutive_fails(mut self, max_consecutive_fails: usize) -> Self {
        self.max_consecutive_fails = Some(max_consecutive_fails);
        self
    }

    /// Disables the consecutive failures check.
    pub fn without_max_consecutive_fails(mut self) -> Self {
        self.max_consecutive_fails = None;
        self
    }
}

impl Default for AgentOptions {
    fn default() -> Self {
        Self {
            max_iterations: Some(10),
            max_consecutive_fails: Some(3),
            max_continuations: Some(0),
        }
    }
}
