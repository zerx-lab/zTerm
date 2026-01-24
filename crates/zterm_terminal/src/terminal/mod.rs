//! Terminal entity and management

mod event;
mod pty_loop;
mod state;

pub use event::TerminalEvent;
pub use pty_loop::{Msg as PtyMsg, Notifier as PtyNotifier, OscEvent, PtyEventLoop};
pub use state::{
    IndexedCell, Terminal, TerminalBounds, TerminalContent, TerminalEventListener, TerminalSize,
    ZoneInfo,
};
