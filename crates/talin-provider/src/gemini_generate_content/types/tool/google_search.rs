use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use time::OffsetDateTime;

/// GoogleSearch tool type. Tool to support Google Search in Model. Powered by Google.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GoogleSearch {
    /// Optional. Filter search results to a specific time range. If customers set a start time,
    /// they must set an end time (and vice versa).
    pub time_range_filter: Option<Interval>,

    /// Optional. The types of searches to enable (e.g. web search, image search).
    pub search_types: Option<SearchTypes>,
}

impl GoogleSearch {
    /// Creates a new empty GoogleSearch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the time range filter for this tool.
    pub fn with_time_range_filter(mut self, time_range_filter: Interval) -> Self {
        self.time_range_filter = Some(time_range_filter);
        self
    }

    /// Sets the types of searches to enable.
    pub fn with_search_types(mut self, search_types: SearchTypes) -> Self {
        self.search_types = Some(search_types);
        self
    }
}

/// Represents a time interval, encoded as a Timestamp start (inclusive) and a Timestamp end
/// (exclusive).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Interval {
    /// Optional. Inclusive start of the interval.
    pub start_time: Option<OffsetDateTime>,
    /// Optional. Exclusive end of the interval.
    pub end_time: Option<OffsetDateTime>,
}

impl Interval {
    /// Creates a new empty Interval.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the inclusive start of the interval.
    pub fn with_start_time(mut self, start_time: OffsetDateTime) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Sets the exclusive end of the interval.
    pub fn with_end_time(mut self, end_time: OffsetDateTime) -> Self {
        self.end_time = Some(end_time);
        self
    }
}

/// The kinds of searches that can be performed by the `GoogleSearch` tool.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SearchTypes {
    /// Optional. Standard web search for grounding and related configurations.
    pub web_search: Option<WebSearch>,
    /// Optional. Image search for grounding and related configurations.
    pub image_search: Option<ImageSearch>,
}

impl SearchTypes {
    /// Creates a new empty SearchTypes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the standard web search configuration.
    pub fn with_web_search(mut self, web_search: WebSearch) -> Self {
        self.web_search = Some(web_search);
        self
    }

    /// Sets the image search configuration.
    pub fn with_image_search(mut self, image_search: ImageSearch) -> Self {
        self.image_search = Some(image_search);
        self
    }
}

/// Standard web search for grounding.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebSearch {}

/// Image search for grounding.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImageSearch {}
