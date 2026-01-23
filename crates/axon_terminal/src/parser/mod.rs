//! VT/ANSI sequence parsing
//!
//! This module wraps alacritty_terminal for VT sequence parsing.

mod ansi;

pub use ansi::AnsiParser;
