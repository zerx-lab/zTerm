//! Axon Terminal - UI Components
//!
//! This crate provides the UI components for Axon Terminal, built on GPUI
//! and gpui-component library.

#![recursion_limit = "256"]

pub mod components;
pub mod elements;
pub mod theme;

pub use components::title_bar::{NewTab, TitleBarEvent};
pub use components::{TabInfo, TerminalTabBar, TerminalView, TitleBar};
pub use theme::TerminalTheme;
