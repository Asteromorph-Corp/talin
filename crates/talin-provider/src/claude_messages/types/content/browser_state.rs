use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::CacheControlEphemeral;
use crate::utils::tagged_enum;

/// The caller's browser state after a browser toolset member call.
///
/// Carries the full inventory of open tabs, which tab is active, and any side effects — tabs
/// opened, download state changes — the call produced.
///
/// At most one per tool result, and only on a non-error result answering a browser toolset member
/// tool use. The server renders the model-visible text from it; the model never sees the raw
/// fields.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BrowserStateBlockParam {
    /// All tabs open in the browser after this call — the full inventory, not a delta.
    ///
    /// May be empty. Whenever it is not, exactly one entry is
    /// [`active`](BrowserStateTabEntry::active).
    pub tabs: Vec<BrowserStateTabEntry>,

    /// The tabs opened and download state changes during this call.
    ///
    /// "Nothing to report" is expressed by omitting the field, never by an empty list.
    pub state_changes: Option<Vec<BrowserStateChange>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

/// One open browser tab reported in a [`BrowserStateBlockParam`]'s inventory.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BrowserStateTabEntry {
    /// The caller-assigned identifier for this tab, unique within the inventory.
    pub tab_id: String,

    /// The title of the page the tab is showing. May be empty.
    pub title: String,

    /// The URL of the page the tab is showing. May be empty.
    pub url: String,

    /// Whether this tab is the active tab after this call.
    ///
    /// Whenever the inventory is not empty, exactly one entry is marked.
    #[cfg_attr(test, schema(nullable = false))]
    pub active: Option<bool>,
}

tagged_enum! {
    /// A side effect a browser toolset member call produced.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum BrowserStateChange {
        /// A tab this call's execution opened that remains open at its end.
        TabOpened(BrowserStateChangeTabOpened),
        /// A file download that started during this call.
        DownloadStarted(BrowserStateChangeDownloadStarted),
        /// A file download that finished during this call.
        DownloadCompleted(BrowserStateChangeDownloadCompleted),
        /// A file download that failed — or was cancelled — during this call.
        DownloadFailed(BrowserStateChangeDownloadFailed),
    }
}

/// A tab this call's execution opened that remains open at its end — the creation delta of the
/// tab inventory, not an event log.
///
/// Carries only the tab id; the tab's title and URL live on its
/// [`tabs`](BrowserStateBlockParam::tabs) entry, which must carry the same tab id. A tab opened
/// during a failed call gets no deferred change; it simply appears in the next result's inventory.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BrowserStateChangeTabOpened {
    /// The id of the opened tab, present in [`tabs`](BrowserStateBlockParam::tabs).
    pub tab_id: String,
}

/// A file download that started during this call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BrowserStateChangeDownloadStarted {
    /// The caller-assigned identifier for this download, stable across the state changes
    /// reporting it.
    pub download_id: String,

    /// The final post-redirect URL the download was served from.
    pub url: String,
}

/// A file download that finished during this call.
///
/// Reported with the same download id as the download that started it — or without a prior
/// start, when the download finished during the call that started it. At most one state change
/// per download id per result.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BrowserStateChangeDownloadCompleted {
    /// The caller-assigned identifier for this download, stable across the state changes
    /// reporting it.
    pub download_id: String,

    /// The final post-redirect URL the download was served from.
    pub url: String,

    /// Where the executor saved the file, on the executor's filesystem.
    ///
    /// Only included when another tool in the same environment can read the file at that path.
    pub path: Option<String>,

    /// The completed download's size.
    #[cfg_attr(
        test,
        schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema)
    )]
    pub size_bytes: Option<u64>,
}

/// A file download that failed — or was cancelled — during this call.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BrowserStateChangeDownloadFailed {
    /// The caller-assigned identifier for this download, stable across the state changes
    /// reporting it.
    pub download_id: String,

    /// The final post-redirect URL the download was served from.
    pub url: String,

    /// The failure or cancellation detail, when known.
    pub error: Option<String>,
}

impl BrowserStateBlockParam {
    /// Creates a new [`BrowserStateBlockParam`] from the given tab inventory.
    pub fn new(tabs: Vec<BrowserStateTabEntry>) -> Self {
        Self {
            tabs,
            state_changes: None,
            cache_control: None,
        }
    }

    /// Sets the [`state_changes`](Self::state_changes) this call produced.
    pub fn with_state_changes(mut self, state_changes: Vec<BrowserStateChange>) -> Self {
        self.state_changes = Some(state_changes);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl BrowserStateTabEntry {
    /// Creates a new [`BrowserStateTabEntry`] from the given tab id, title, and URL.
    pub fn new(
        tab_id: impl Into<String>,
        title: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            tab_id: tab_id.into(),
            title: title.into(),
            url: url.into(),
            active: None,
        }
    }

    /// Sets whether this tab is the [`active`](Self::active) one after the call.
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }
}

impl BrowserStateChangeTabOpened {
    /// Creates a new [`BrowserStateChangeTabOpened`] from the given tab id.
    pub fn new(tab_id: impl Into<String>) -> Self {
        Self {
            tab_id: tab_id.into(),
        }
    }
}

impl BrowserStateChangeDownloadStarted {
    /// Creates a new [`BrowserStateChangeDownloadStarted`] from the given download id and URL.
    pub fn new(download_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            download_id: download_id.into(),
            url: url.into(),
        }
    }
}

impl BrowserStateChangeDownloadCompleted {
    /// Creates a new [`BrowserStateChangeDownloadCompleted`] from the given download id and URL.
    pub fn new(download_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            download_id: download_id.into(),
            url: url.into(),
            path: None,
            size_bytes: None,
        }
    }

    /// Sets the [`path`](Self::path) the executor saved the file at.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Sets the completed download's [`size_bytes`](Self::size_bytes).
    pub fn with_size_bytes(mut self, size_bytes: u64) -> Self {
        self.size_bytes = Some(size_bytes);
        self
    }
}

impl BrowserStateChangeDownloadFailed {
    /// Creates a new [`BrowserStateChangeDownloadFailed`] from the given download id and URL.
    pub fn new(download_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            download_id: download_id.into(),
            url: url.into(),
            error: None,
        }
    }

    /// Sets the failure or cancellation detail in [`error`](Self::error).
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }
}
