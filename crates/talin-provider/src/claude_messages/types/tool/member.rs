//! Declaring a toolset's per-member configuration from the member list alone.
//!
//! Upstream gives every member tool of a toolset its own configuration schema, and every one of
//! them currently carries the same two fields. They are separate schemas, so upstream can add a
//! field to one member without touching the rest; sharing a single type between them here would
//! make that change impossible to express. So each is declared — from the member list, to keep the
//! duplication from being written out by hand.

/// Declares the per-member configuration of a toolset, and a configuration type per member.
///
/// ```ignore
/// toolset_configs! {
///     /// The per-member configuration of a [`BrowserToolset20260801`].
///     BrowserToolsetConfigs {
///         close_tab: BrowserCloseTabConfig,
///         navigate: BrowserNavigateConfig,
///     }
/// }
/// ```
macro_rules! toolset_configs {
    (
        $(#[$meta:meta])*
        $configs:ident { $($member:ident: $config:ident,)* }
    ) => {
        $(#[$meta])*
        #[skip_serializing_none]
        #[cfg_attr(test, derive(utoipa::ToSchema))]
        #[derive(Serialize, Deserialize, Debug, Default, Clone)]
        #[non_exhaustive]
        pub struct $configs {
            $(
                #[doc = concat!("`", stringify!($member), "`'s configuration overrides.")]
                pub $member: Option<$config>,
            )*
        }

        impl $configs {
            #[doc = concat!("Creates a new [`", stringify!($configs), "`], overriding nothing.")]
            ///
            /// Each member's configuration is a public field, left `None` to keep that member's
            /// defaults.
            pub fn new() -> Self {
                Self::default()
            }
        }

        $(
            #[doc = concat!("`", stringify!($member), "`'s configuration overrides.")]
            #[skip_serializing_none]
            #[cfg_attr(test, derive(utoipa::ToSchema))]
            #[derive(Serialize, Deserialize, Debug, Default, Clone)]
            #[non_exhaustive]
            pub struct $config {
                /// Whether this member is offered to the model.
                ///
                /// The default is per member, per the toolset's documentation. A member whose
                /// value resolves to `false` is withheld from the served schema.
                pub enabled: Option<bool>,

                /// Whether to defer loading this member.
                ///
                /// Must resolve to the same value on every enabled member of the toolset.
                pub defer_loading: Option<bool>,
            }

            impl $config {
                #[doc = concat!("Creates a new [`", stringify!($config), "`], overriding nothing.")]
                pub fn new() -> Self {
                    Self::default()
                }

                /// Sets whether this member is [`enabled`](Self::enabled).
                pub fn with_enabled(mut self, enabled: bool) -> Self {
                    self.enabled = Some(enabled);
                    self
                }

                /// Sets whether to [`defer_loading`](Self::defer_loading) this member.
                pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
                    self.defer_loading = Some(defer_loading);
                    self
                }
            }
        )*
    };
}

pub(super) use toolset_configs;
