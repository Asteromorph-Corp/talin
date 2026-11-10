/// Constrains effort on reasoning for reasoning models.
///
/// Providers support different subsets of these levels. A level a provider does
/// not support is mapped to its nearest supported one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReasoningEffort {
    /// No effort spent on reasoning.
    None,
    /// Minimal effort spent on reasoning.
    Minimal,
    /// Low effort spent on reasoning.
    Low,
    /// Moderate effort spent on reasoning.
    Medium,
    /// Significant effort spent on reasoning.
    High,
    /// Very high effort spent on reasoning.
    Xhigh,
    /// Maximum effort spent on reasoning.
    Max,
}
