//! Workspace management

use gpui::*;
use tracing::debug;

/// Information about a tab
pub struct TabInfo {
    /// Tab ID
    pub id: usize,
    /// Tab title
    pub title: String,
    /// Tab content (placeholder)
    pub content: String,
}

/// Workspace containing multiple tabs
pub struct Workspace {
    /// Tabs in this workspace
    tabs: Vec<TabInfo>,

    /// Index of the active tab
    active_tab: usize,

    /// Counter for generating unique tab IDs
    next_tab_id: usize,
}

impl Workspace {
    /// Create a new workspace with an initial tab
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut workspace = Self {
            tabs: vec![],
            active_tab: 0,
            next_tab_id: 1,
        };

        // Create initial tab
        workspace.new_tab(cx);

        workspace
    }

    /// Create a new tab
    pub fn new_tab(&mut self, cx: &mut Context<Self>) {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab_info = TabInfo {
            id: tab_id,
            title: format!("Tab {}", tab_id),
            content: format!("Content for tab {}", tab_id),
        };

        self.tabs.push(tab_info);
        self.active_tab = self.tabs.len() - 1;

        debug!("Created new tab {}", tab_id);
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

    /// Get all tabs as TabInfo for UI display
    pub fn get_tab_infos(&self) -> Vec<zterm_ui::TabInfo> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| zterm_ui::TabInfo::new(tab.id, tab.title.clone(), i == self.active_tab))
            .collect()
    }

    /// Get the active tab content (placeholder)
    pub fn active_tab_content(&self) -> Option<&str> {
        self.tabs.get(self.active_tab).map(|t| t.content.as_str())
    }
}
