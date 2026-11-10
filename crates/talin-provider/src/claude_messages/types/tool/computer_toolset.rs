use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::member::toolset_configs;
use crate::claude_messages::types::CacheControlEphemeral;

/// The computer toolset (2026-08-01).
///
/// A single tool entry (carrying no name) that declares the computer tool family. The model is
/// served the family's tool with any members disabled via [`configs`](Self::configs) removed from
/// its schema.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct ComputerToolset20260801 {
    /// The per-member configuration of this toolset.
    pub configs: Option<ComputerToolsetConfigs>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

toolset_configs! {
    /// The per-member configuration of a [`ComputerToolset20260801`].
    ComputerToolsetConfigs {
        cursor_position: ComputerCursorPositionConfig,
        double_click: ComputerDoubleClickConfig,
        hold_key: ComputerHoldKeyConfig,
        key: ComputerKeyConfig,
        left_click: ComputerLeftClickConfig,
        left_click_drag: ComputerLeftClickDragConfig,
        left_mouse_down: ComputerLeftMouseDownConfig,
        left_mouse_up: ComputerLeftMouseUpConfig,
        middle_click: ComputerMiddleClickConfig,
        mouse_move: ComputerMouseMoveConfig,
        right_click: ComputerRightClickConfig,
        screenshot: ComputerScreenshotConfig,
        scroll: ComputerScrollConfig,
        triple_click: ComputerTripleClickConfig,
        r#type: ComputerTypeConfig,
        wait: ComputerWaitConfig,
        zoom: ComputerZoomConfig,
    }
}

impl ComputerToolset20260801 {
    /// Creates a new [`ComputerToolset20260801`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the per-member [`configs`](Self::configs) of this toolset.
    pub fn with_configs(mut self, configs: ComputerToolsetConfigs) -> Self {
        self.configs = Some(configs);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}
