//! Terminal events

use crate::shell_integration::ShellEvent;

/// Events emitted by the terminal
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Terminal output received
    Output(Vec<u8>),

    /// Bell character received
    Bell,

    /// Terminal title changed
    TitleChanged(String),

    /// Cursor position changed
    CursorMoved { x: usize, y: usize },

    /// Terminal was scrolled
    Scrolled { lines: i32 },

    /// Terminal was resized
    Resized { cols: usize, rows: usize },

    /// Process exited
    ProcessExited { exit_code: Option<i32> },

    /// Error occurred
    Error(String),

    /// Shell integration event
    ShellIntegration(ShellEvent),
}
