//! zTerm - Terminal Core
//!
//! This crate provides the core terminal functionality including:
//! - PTY management using alacritty's tty module
//! - Terminal emulation using alacritty_terminal
//! - Event batching for smooth UI updates

// Re-export alacritty_terminal for use by rendering code
pub use alacritty_terminal;

// Re-export selection types for mouse selection
pub use alacritty_terminal::index::Direction as SelectionSide;
pub use alacritty_terminal::selection::SelectionType;

pub mod terminal;

// Keep these for backwards compatibility, but they're now unused
pub mod buffer;
pub mod parser;
pub mod platform;

pub use terminal::{
    IndexedCell, Terminal, TerminalBounds, TerminalContent, TerminalEvent, TerminalSize,
};
