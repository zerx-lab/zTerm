//! Axon Terminal - Terminal Core
//!
//! This crate provides the core terminal functionality including:
//! - PTY management (cross-platform)
//! - VT/ANSI sequence parsing
//! - Screen buffer management
//! - Terminal state machine

pub mod buffer;
pub mod parser;
pub mod platform;
pub mod terminal;

pub use terminal::{Terminal, TerminalEvent, TerminalSize};
