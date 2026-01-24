//! UI Components for zTerm

mod context_menu;
mod context_menu_view;
mod tab_bar;
mod terminal_view;
pub mod title_bar;

pub use context_menu::{ContextMenuItem, ContextMenuState};
pub use context_menu_view::{ContextMenuView, MenuItemData};
pub use tab_bar::TerminalTabBar;
pub use terminal_view::{
    Copy, GridPosition, ImeState, Paste, ScrollDown, ScrollPageDown, ScrollPageUp,
    ScrollToBottom, ScrollToTop, ScrollUp, Search, SharedBounds, TerminalView,
};
pub use title_bar::{TabInfo, TitleBar};
