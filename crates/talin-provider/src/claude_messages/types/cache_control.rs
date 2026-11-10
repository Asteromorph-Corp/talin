use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::{string_enum, tagged_enum};

tagged_enum! {
    /// Cache control configuration for content items.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum CacheControlEphemeral {
        /// An ephemeral cache with a specified TTL.
        Ephemeral(CacheControlEphemeralInner),
    }
}

/// An ephemeral cache with a specified TTL.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct CacheControlEphemeralInner {
    /// The time-to-live for the cache. Defaults to `"5m"` if not specified.
    #[cfg_attr(test, schema(nullable = false))]
    pub ttl: Option<CacheTtl>,
}

string_enum! {
    /// A time-to-live for an ephemeral cache.
    pub enum CacheTtl {
        /// Five minutes.
        #[serde(rename = "5m")]
        FiveMinutes,

        /// One hour.
        #[serde(rename = "1h")]
        OneHour,
    }
}

impl CacheControlEphemeralInner {
    /// Creates a new [`CacheControlEphemeralInner`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`ttl`](Self::ttl) for the ephemeral cache.
    pub fn with_ttl(mut self, ttl: CacheTtl) -> Self {
        self.ttl = Some(ttl);
        self
    }
}
