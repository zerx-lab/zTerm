//! zTerm - Terminal Core
//!
//! This crate provides the core terminal functionality including:
//! - PTY management using alacritty's tty module
//! - Terminal emulation using alacritty_terminal
//! - Event batching for smooth UI updates
//! - Shell integration via OSC 133/633

// Re-export alacritty_terminal for use by rendering code
pub use alacritty_terminal;

pub mod shell_integration;
pub mod terminal;

// Keep these for backwards compatibility, but they're now unused
pub mod buffer;
pub mod parser;
pub mod platform;

pub use terminal::{
    IndexedCell, OscEvent, PtyEventLoop, PtyMsg, PtyNotifier, Terminal, TerminalBounds,
    TerminalContent, TerminalEvent, TerminalSize, ZoneInfo,
};
