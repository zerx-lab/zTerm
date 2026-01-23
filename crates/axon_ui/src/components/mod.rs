//! UI Components for Axon Terminal

mod tab_bar;
mod terminal_view;
mod title_bar;

pub use tab_bar::TerminalTabBar;
pub use terminal_view::{SharedBounds, TerminalView};
pub use title_bar::{TabInfo, TitleBar};
