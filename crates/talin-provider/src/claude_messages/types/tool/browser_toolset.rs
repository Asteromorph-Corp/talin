use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::member::toolset_configs;
use crate::claude_messages::types::CacheControlEphemeral;

/// The browser toolset (2026-08-01).
///
/// A single tool entry — carrying no name — that declares the browser tool family.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct BrowserToolset20260801 {
    /// The per-member configuration of this toolset.
    pub configs: Option<BrowserToolsetConfigs>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

toolset_configs! {
    /// The per-member configuration of a [`BrowserToolset20260801`].
    BrowserToolsetConfigs {
        close_tab: BrowserCloseTabConfig,
        double_click: BrowserDoubleClickConfig,
        file_upload: BrowserFileUploadConfig,
        find: BrowserFindConfig,
        form_input: BrowserFormInputConfig,
        get_page_text: BrowserGetPageTextConfig,
        hold_key: BrowserHoldKeyConfig,
        hover: BrowserHoverConfig,
        javascript_exec: BrowserJavascriptExecConfig,
        key: BrowserKeyConfig,
        left_click: BrowserLeftClickConfig,
        left_click_drag: BrowserLeftClickDragConfig,
        left_mouse_down: BrowserLeftMouseDownConfig,
        left_mouse_up: BrowserLeftMouseUpConfig,
        list_tabs: BrowserListTabsConfig,
        middle_click: BrowserMiddleClickConfig,
        mouse_move: BrowserMouseMoveConfig,
        navigate: BrowserNavigateConfig,
        new_tab: BrowserNewTabConfig,
        read_console: BrowserReadConsoleConfig,
        read_network: BrowserReadNetworkConfig,
        read_page: BrowserReadPageConfig,
        right_click: BrowserRightClickConfig,
        screenshot: BrowserScreenshotConfig,
        scroll: BrowserScrollConfig,
        scroll_to: BrowserScrollToConfig,
        switch_tab: BrowserSwitchTabConfig,
        triple_click: BrowserTripleClickConfig,
        r#type: BrowserTypeConfig,
        wait: BrowserWaitConfig,
        zoom: BrowserZoomConfig,
    }
}

impl BrowserToolset20260801 {
    /// Creates a new [`BrowserToolset20260801`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the per-member [`configs`](Self::configs) of this toolset.
    pub fn with_configs(mut self, configs: BrowserToolsetConfigs) -> Self {
        self.configs = Some(configs);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}
