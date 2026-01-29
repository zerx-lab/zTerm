//! Low-level UI elements for zTerm
//!
//! This module contains custom GPUI Element implementations
//! for high-performance terminal rendering.

pub mod terminal_element;
pub mod terminal_scrollbar;

pub use terminal_element::{
    BatchedTextRun, CursorLayout, LayoutRect, LayoutState, TerminalBounds, TerminalElement,
    TextRunStyle,
};
pub use terminal_scrollbar::{TerminalScrollHandle, ThumbState, SCROLLBAR_WIDTH};
