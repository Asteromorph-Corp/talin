//! Serde adapters for the Google protobuf [Duration] JSON format
//! (e.g. `"3.5s"` — seconds with up to nine fractional digits, suffixed by
//! `'s'`).

use std::time::Duration;

use serde::{Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

pub(super) struct GoogleDuration;

impl SerializeAs<Duration> for GoogleDuration {
    fn serialize_as<S: Serializer>(source: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format(source))
    }
}

impl<'de> DeserializeAs<'de, Duration> for GoogleDuration {
    fn deserialize_as<D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse(&s).map_err(serde::de::Error::custom)
    }
}

fn format(d: &Duration) -> String {
    let nanos = d.subsec_nanos();
    if nanos == 0 {
        return format!("{}s", d.as_secs());
    }
    let mut frac = format!("{nanos:09}");
    while frac.ends_with('0') {
        frac.pop();
    }
    format!("{}.{}s", d.as_secs(), frac)
}

fn parse(s: &str) -> Result<Duration, String> {
    let trimmed = s
        .strip_suffix('s')
        .ok_or_else(|| "expected Google Duration string ending in 's'".to_string())?;
    let (secs_str, nanos) = match trimmed.split_once('.') {
        Some((secs, frac)) => {
            if frac.is_empty() || frac.len() > 9 || !frac.chars().all(|c| c.is_ascii_digit()) {
                return Err("invalid fractional seconds".to_string());
            }
            let mut padded = String::from(frac);
            while padded.len() < 9 {
                padded.push('0');
            }
            let nanos = padded.parse::<u32>().map_err(|e| e.to_string())?;
            (secs, nanos)
        }
        None => (trimmed, 0u32),
    };
    let secs = secs_str.parse::<u64>().map_err(|e| e.to_string())?;
    Ok(Duration::new(secs, nanos))
}
