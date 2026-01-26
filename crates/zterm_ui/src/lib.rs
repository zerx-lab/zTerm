//! zTerm - UI Components
//!
//! This crate provides the UI components for zTerm, built on GPUI
//! and gpui-component library.

#![recursion_limit = "256"]

pub mod components;
pub mod elements;

pub use components::title_bar::{
    LinuxWindowControls, NewTab, PlatformStyle, TITLE_BAR_HEIGHT, TitleBarEvent,
    WindowsWindowControls,
};
pub use components::{TabInfo, TerminalTabBar, TitleBar};
