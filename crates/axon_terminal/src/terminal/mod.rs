//! Terminal entity and management

mod event;
mod state;

pub use event::TerminalEvent;
pub use state::{
    IndexedCell, Terminal, TerminalBounds, TerminalContent, TerminalEventListener, TerminalSize,
};
