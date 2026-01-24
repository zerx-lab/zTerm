//! zTerm - UI Components
//!
//! This crate provides the UI components for zTerm, built on GPUI
//! and gpui-component library.

#![recursion_limit = "256"]
#![allow(clippy::type_complexity)]
#![allow(clippy::new_without_default)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::clone_on_copy)]

pub mod components;
pub mod elements;
pub mod shell_integration;
pub mod theme;

pub use components::title_bar::{
    LinuxWindowControls, NewTab, PlatformStyle, TITLE_BAR_HEIGHT, TitleBarEvent,
    WindowsWindowControls,
};
pub use components::{
    ContextMenuItem, ContextMenuState, Copy, GridPosition, ImeState, Paste, ScrollDown,
    ScrollPageDown, ScrollPageUp, ScrollToBottom, ScrollToTop, ScrollUp, Search, SharedBounds,
    TabInfo, TerminalTabBar, TerminalView, TitleBar,
};
pub use elements::{ScrollbarElement, ScrollbarState, Selection, ThumbState};
pub use shell_integration::ContextMenuAction;
pub use theme::TerminalTheme;
