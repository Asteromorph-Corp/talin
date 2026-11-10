use std::borrow::Cow;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Detailed token usage with per-model, per-kind billing breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(into = "Vec<UsageModelEntry>", from = "Vec<UsageModelEntry>")]
pub struct Usage(pub IndexMap<ModelKey, IndexMap<Cow<'static, str>, UsageLine>>);

/// An identifier for a model and its provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelKey {
    /// The name of the model.
    pub model: String,
    /// The model provider.
    pub provider: Cow<'static, str>,
}

/// The usage lines for a single model, keyed by kind (e.g. input or output tokens).
pub type UsageLines = IndexMap<Cow<'static, str>, UsageLine>;

/// A single usage line.
#[derive(Debug, Clone)]
pub struct UsageLine {
    /// The number of units used.
    pub count: u32,
    /// USD price per magnitude units (e.g. per million tokens)
    pub price: Option<f64>,
    /// Unit description (e.g. "requests", "images", etc.)
    pub unit: Cow<'static, str>,
    /// Unit magnitude (e.g. 1 for per-request, 1_000_000 for per-million tokens)
    pub magnitude: u32,
}

impl Usage {
    /// Returns true if there are no usage lines recorded.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add a line for a given model/provider and kind. Accumulates count if the entry already
    /// exists.
    pub fn add(&mut self, key: ModelKey, kind: &'static str, line: UsageLine) {
        let existing = self
            .0
            .entry(key)
            .or_default()
            .entry(Cow::Borrowed(kind))
            .or_insert(UsageLine { count: 0, ..line });
        existing.count += line.count;
    }

    /// Merge another `Usage` into this one, summing counts for matching keys.
    pub fn merge(mut self, other: &Usage) -> Usage {
        self.merge_inplace(other);
        self
    }

    /// Merges another `Usage` into this one in place, summing counts for matching keys.
    pub fn merge_inplace(&mut self, other: &Usage) {
        for (model_key, lines) in &other.0 {
            let entry = self.0.entry(model_key.clone()).or_default();
            for (kind, line) in lines {
                let existing = entry.entry(kind.clone()).or_insert(UsageLine {
                    count: 0,
                    ..line.clone()
                });
                existing.count += line.count;
            }
        }
    }
}

impl ModelKey {
    /// Creates a [`ModelKey`] for a model served by the given provider.
    pub fn new(model: String, provider: &'static str) -> Self {
        ModelKey {
            model,
            provider: Cow::Borrowed(provider),
        }
    }
}

impl UsageLine {
    /// Creates a new [`UsageLine`].
    pub fn new(count: u32, price: Option<f64>, unit: &'static str, magnitude: u32) -> Self {
        UsageLine {
            count,
            price,
            unit: Cow::Borrowed(unit),
            magnitude,
        }
    }

    /// Creates a new [`UsageLine`] for token usage with the price per million tokens.
    pub fn per_million_tokens(count: u32, price: Option<f64>) -> Self {
        Self::new(count, price, "tokens", 1_000_000)
    }

    /// Returns the cost in USD, or `None` if the price is unknown.
    pub fn cost(&self) -> Option<f64> {
        Some(self.count as f64 * self.price? / self.magnitude as f64)
    }
}

/// Format a dollar amount in scientific notation (e.g. "2.0 x 10^-5 $").
fn format_cost(value: f64) -> String {
    if value == 0.0 {
        return "$0".to_string();
    }
    let exp = value.abs().log10().floor() as i32;
    if exp != 0 {
        format!("${value:.1e}")
    } else {
        format!("${value:.1}")
    }
}

/// Format a magnitude as a human-readable string (e.g. 1_000_000 -> "1M", 1_000 -> "1K", 1 -> "1").
fn format_magnitude(magnitude: u32) -> String {
    match magnitude {
        1_000_000 => "1M".to_string(),
        1_000 => "1K".to_string(),
        other => other.to_string(),
    }
}

/// Intermediate representation for JSON-compatible serialization of [`Usage`].
#[derive(Serialize, Deserialize)]
struct UsageModelEntry {
    model: String,
    provider: String,
    lines: Vec<UsageLineEntry>,
}

/// Intermediate representation for JSON-compatible serialization of [`UsageLine`].
#[derive(Serialize, Deserialize)]
struct UsageLineEntry {
    kind: String,
    count: u32,
    price: Option<f64>,
    unit: String,
    magnitude: u32,
}

impl From<Usage> for Vec<UsageModelEntry> {
    fn from(usage: Usage) -> Self {
        usage
            .0
            .into_iter()
            .map(|(key, lines)| UsageModelEntry {
                model: key.model,
                provider: key.provider.into_owned(),
                lines: lines
                    .into_iter()
                    .map(|(kind, line)| UsageLineEntry {
                        kind: kind.into_owned(),
                        count: line.count,
                        price: line.price,
                        unit: line.unit.into_owned(),
                        magnitude: line.magnitude,
                    })
                    .collect(),
            })
            .collect()
    }
}

impl From<Vec<UsageModelEntry>> for Usage {
    fn from(entries: Vec<UsageModelEntry>) -> Self {
        let entries = entries.into_iter().map(|entry| {
            let key = ModelKey {
                model: entry.model,
                provider: Cow::Owned(entry.provider),
            };
            let lines: UsageLines = entry
                .lines
                .into_iter()
                .map(|line| {
                    let usage_line = UsageLine {
                        count: line.count,
                        price: line.price,
                        unit: Cow::Owned(line.unit),
                        magnitude: line.magnitude,
                    };
                    (Cow::Owned(line.kind), usage_line)
                })
                .collect();
            (key, lines)
        });
        Usage(IndexMap::from_iter(entries))
    }
}

impl std::fmt::Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first_model = true;
        for (model_key, lines) in &self.0 {
            if !first_model {
                writeln!(f)?;
            }
            first_model = false;

            writeln!(f, "- {} ({})", model_key.model, model_key.provider)?;
            let mut subtotal = Some(0.0);
            for (kind, line) in lines {
                match line.price {
                    Some(price) => {
                        let cost = line.count as f64 * price / line.magnitude as f64;
                        if let Some(subtotal) = &mut subtotal {
                            *subtotal += cost;
                        }
                        writeln!(
                            f,
                            "  - {kind}: {} {} x ${} per {} {} = {}",
                            line.count,
                            line.unit,
                            price,
                            format_magnitude(line.magnitude),
                            line.unit,
                            format_cost(cost),
                        )?;
                    }
                    None => {
                        subtotal = None;
                        writeln!(
                            f,
                            "  - {kind}: {} {} (price unknown)",
                            line.count, line.unit,
                        )?;
                    }
                }
            }
            if let Some(subtotal) = subtotal {
                write!(f, "  - Subtotal: {}", format_cost(subtotal))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_usage_display() {
        let mut usage = Usage::default();
        let key = ModelKey {
            model: "gpt-4o-mini".to_owned(),
            provider: "OpenAI".into(),
        };
        usage.add(
            key.clone(),
            "Input tokens (not cached)",
            UsageLine::per_million_tokens(100, Some(0.2)),
        );
        usage.add(
            key.clone(),
            "Input tokens (cached)",
            UsageLine::per_million_tokens(20, Some(0.1)),
        );
        usage.add(
            key,
            "Output tokens",
            UsageLine::per_million_tokens(30, Some(1.0)),
        );

        let output = format!("{usage}");
        let expected = indoc! {"
            - gpt-4o-mini (OpenAI)
              - Input tokens (not cached): 100 tokens x $0.2 per 1M tokens = $2.0e-5
              - Input tokens (cached): 20 tokens x $0.1 per 1M tokens = $2.0e-6
              - Output tokens: 30 tokens x $1 per 1M tokens = $3.0e-5
              - Subtotal: $5.2e-5"
        };
        assert_eq!(output, expected);
    }

    #[test]
    fn test_usage_merge() {
        let key = ModelKey {
            model: "gpt-4o-mini".into(),
            provider: "OpenAI".into(),
        };

        let mut a = Usage::default();
        a.add(
            key.clone(),
            "Input tokens",
            UsageLine::per_million_tokens(100, Some(0.2)),
        );

        let mut b = Usage::default();
        b.add(
            key.clone(),
            "Input tokens",
            UsageLine::per_million_tokens(50, Some(0.2)),
        );
        b.add(
            key.clone(),
            "Output tokens",
            UsageLine::per_million_tokens(30, Some(1.0)),
        );

        let merged = a.merge(&b);

        let lines = &merged.0[&key];
        assert_eq!(lines["Input tokens"].count, 150);
        assert_eq!(lines["Output tokens"].count, 30);
    }

    #[test]
    fn test_usage_serde_roundtrip() {
        let mut usage = Usage::default();
        let key = ModelKey {
            model: "gpt-4o-mini".to_owned(),
            provider: "OpenAI".into(),
        };
        usage.add(
            key.clone(),
            "Input tokens",
            UsageLine::per_million_tokens(100, Some(0.2)),
        );
        usage.add(
            key,
            "Output tokens",
            UsageLine::per_million_tokens(30, None),
        );

        let json = serde_json::to_string_pretty(&usage).unwrap();
        let deserialized: Usage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.0.len(), 1);
        let (key, lines) = deserialized.0.into_iter().next().unwrap();
        assert_eq!(key.model, "gpt-4o-mini");
        assert_eq!(key.provider, "OpenAI");
        assert_eq!(lines["Input tokens"].count, 100);
        assert_eq!(lines["Input tokens"].price, Some(0.2));
        assert_eq!(lines["Input tokens"].unit, "tokens");
        assert_eq!(lines["Input tokens"].magnitude, 1_000_000);
        assert_eq!(lines["Output tokens"].count, 30);
        assert_eq!(lines["Output tokens"].price, None);
    }

    #[test]
    fn test_format_cost() {
        // Zero
        assert_eq!(format_cost(0.0), "$0");
        // Large values (exp > 0)
        assert_eq!(format_cost(12.5), "$1.2e1");
        assert_eq!(format_cost(100.0), "$1.0e2");
        // Around $1 (exp == 0, no scientific notation)
        assert_eq!(format_cost(1.0), "$1.0");
        assert_eq!(format_cost(5.5), "$5.5");
        // Sub-dollar (exp < 0)
        assert_eq!(format_cost(0.05), "$5.0e-2");
        assert_eq!(format_cost(0.00002), "$2.0e-5");
        assert_eq!(format_cost(0.000052), "$5.2e-5");
        assert_eq!(format_cost(0.000002), "$2.0e-6");
    }
}
