//! Workspace management

use gpui::*;
use tracing::debug;
use zterm_terminal::{Terminal, TerminalSize};
use zterm_ui::TerminalView;

/// Information about a tab
pub struct TabInfo {
    /// Tab title
    pub title: String,
    /// The terminal view
    pub terminal_view: Entity<TerminalView>,
    /// The underlying terminal
    pub terminal: Entity<Terminal>,
}

/// Workspace containing multiple terminal tabs
pub struct Workspace {
    /// Tabs in this workspace
    tabs: Vec<TabInfo>,

    /// Index of the active tab
    active_tab: usize,

    /// Default terminal size
    terminal_size: TerminalSize,

    /// Counter for generating unique tab IDs
    next_tab_id: usize,
}

impl Workspace {
    /// Create a new workspace with an initial terminal
    pub fn new(terminal_size: TerminalSize, cx: &mut Context<Self>) -> Self {
        let mut workspace = Self {
            tabs: vec![],
            active_tab: 0,
            terminal_size,
            next_tab_id: 1,
        };

        // Create initial tab
        workspace.new_tab(cx);

        workspace
    }

    /// Create a new tab with a terminal
    pub fn new_tab(&mut self, cx: &mut Context<Self>) {
        let terminal = cx.new(|cx| Terminal::new(None, None, self.terminal_size, cx));

        let terminal_view = cx.new(|cx| TerminalView::new(terminal.clone(), cx));

        // Get shell name for the tab title
        let shell_name = {
            let term = terminal.read(cx);
            term.shell_name()
        };

        // Generate tab title with shell name using unique counter
        let tab_number = self.next_tab_id;
        self.next_tab_id += 1;
        let title = format!("{} ({})", shell_name, tab_number);

        let tab_info = TabInfo {
            title,
            terminal_view,
            terminal,
        };

        self.tabs.push(tab_info);
        self.active_tab = self.tabs.len() - 1;

        debug!("Created new tab {}", tab_number);
        cx.notify();
    }

    /// Close the active tab
    /// Returns true if this was the last tab (window should close)
    pub fn close_active_tab(&mut self, cx: &mut Context<Self>) -> bool {
        if self.tabs.len() <= 1 {
            // This is the last tab, signal to close window
            return true;
        }

        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }

        debug!("Closed tab, active is now {}", self.active_tab);
        cx.notify();
        false
    }

    /// Close a specific tab
    /// Returns true if this was the last tab (window should close)
    pub fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        if self.tabs.len() <= 1 {
            // This is the last tab, signal to close window
            return true;
        }

        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.active_tab > index {
            self.active_tab -= 1;
        }

        cx.notify();
        false
    }

    /// Switch to the next tab
    pub fn next_tab(&mut self, cx: &mut Context<Self>) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
        cx.notify();
    }

    /// Switch to the previous tab
    pub fn prev_tab(&mut self, cx: &mut Context<Self>) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = if self.active_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab - 1
        };
        cx.notify();
    }

    /// Set the active tab by index
    pub fn set_active_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.tabs.len() {
            self.active_tab = index;
            cx.notify();
        }
    }

    /// Get the active tab index
    pub fn active_tab_index(&self) -> usize {
        self.active_tab
    }

    /// Get all tabs
    pub fn tabs(&self) -> &[TabInfo] {
        &self.tabs
    }

    /// Get the active terminal view
    pub fn active_terminal_view(&self) -> Option<&Entity<TerminalView>> {
        self.tabs.get(self.active_tab).map(|t| &t.terminal_view)
    }

    /// Get the active terminal
    #[allow(dead_code)]
    pub fn active_terminal(&self) -> Option<&Entity<Terminal>> {
        self.tabs.get(self.active_tab).map(|t| &t.terminal)
    }

    /// Get the working directory of the active terminal
    pub fn active_working_directory(&self) -> Option<String> {
        // TODO: Get from terminal
        Some("~".to_string())
    }

    /// Resize all terminals
    #[allow(dead_code)]
    pub fn resize_terminals(&mut self, size: TerminalSize, cx: &mut Context<Self>) {
        self.terminal_size = size;
        for tab in &self.tabs {
            tab.terminal.update(cx, |terminal, cx| {
                terminal.resize(size, cx);
            });
        }
    }
}
