//! zTerm - UI Components
//!
//! This crate provides the UI components for zTerm, built on GPUI
//! and gpui-component library.

#![recursion_limit = "256"]

pub mod components;
pub mod elements;
pub mod shell_integration;
pub mod theme;

pub use components::title_bar::{
    LinuxWindowControls, NewTab, PlatformStyle, TitleBarEvent, WindowsWindowControls,
    TITLE_BAR_HEIGHT,
};
pub use components::{
    ContextMenuItem, ContextMenuState, Copy, GridPosition, ImeState, Paste, ScrollDown,
    ScrollPageDown, ScrollPageUp, ScrollToBottom, ScrollToTop, ScrollUp, Search, SharedBounds,
    TabInfo, TerminalTabBar, TerminalView, TitleBar,
};
pub use elements::{ScrollbarElement, ScrollbarState, Selection, ThumbState};
pub use shell_integration::ContextMenuAction;
pub use theme::TerminalTheme;
