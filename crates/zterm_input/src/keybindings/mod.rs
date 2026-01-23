//! Keybinding management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Action that can be triggered by a keybinding
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalAction {
    Copy,
    Paste,
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    SplitHorizontal,
    SplitVertical,
    Search,
    CommandPalette,
    ClearScreen,
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollToTop,
    ScrollToBottom,
    ZoomIn,
    ZoomOut,
    ResetZoom,
}

/// Keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    bindings: HashMap<String, TerminalAction>,
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        #[cfg(target_os = "macos")]
        {
            bindings.insert("cmd-c".to_string(), TerminalAction::Copy);
            bindings.insert("cmd-v".to_string(), TerminalAction::Paste);
            bindings.insert("cmd-t".to_string(), TerminalAction::NewTab);
            bindings.insert("cmd-w".to_string(), TerminalAction::CloseTab);
            bindings.insert("cmd-shift-]".to_string(), TerminalAction::NextTab);
            bindings.insert("cmd-shift-[".to_string(), TerminalAction::PrevTab);
            bindings.insert("cmd-d".to_string(), TerminalAction::SplitHorizontal);
            bindings.insert("cmd-shift-d".to_string(), TerminalAction::SplitVertical);
            bindings.insert("cmd-f".to_string(), TerminalAction::Search);
            bindings.insert("cmd-shift-p".to_string(), TerminalAction::CommandPalette);
            bindings.insert("cmd-k".to_string(), TerminalAction::ClearScreen);
            bindings.insert("cmd-up".to_string(), TerminalAction::ScrollUp);
            bindings.insert("cmd-down".to_string(), TerminalAction::ScrollDown);
            bindings.insert("pageup".to_string(), TerminalAction::ScrollPageUp);
            bindings.insert("pagedown".to_string(), TerminalAction::ScrollPageDown);
            bindings.insert("cmd-home".to_string(), TerminalAction::ScrollToTop);
            bindings.insert("cmd-end".to_string(), TerminalAction::ScrollToBottom);
            bindings.insert("cmd-=".to_string(), TerminalAction::ZoomIn);
            bindings.insert("cmd--".to_string(), TerminalAction::ZoomOut);
            bindings.insert("cmd-0".to_string(), TerminalAction::ResetZoom);
        }

        #[cfg(not(target_os = "macos"))]
        {
            bindings.insert("ctrl-shift-c".to_string(), TerminalAction::Copy);
            bindings.insert("ctrl-shift-v".to_string(), TerminalAction::Paste);
            bindings.insert("ctrl-shift-t".to_string(), TerminalAction::NewTab);
            bindings.insert("ctrl-shift-w".to_string(), TerminalAction::CloseTab);
            bindings.insert("ctrl-tab".to_string(), TerminalAction::NextTab);
            bindings.insert("ctrl-shift-tab".to_string(), TerminalAction::PrevTab);
            bindings.insert("ctrl-shift-d".to_string(), TerminalAction::SplitHorizontal);
            bindings.insert("ctrl-shift-e".to_string(), TerminalAction::SplitVertical);
            bindings.insert("ctrl-shift-f".to_string(), TerminalAction::Search);
            bindings.insert("ctrl-shift-p".to_string(), TerminalAction::CommandPalette);
            bindings.insert("ctrl-l".to_string(), TerminalAction::ClearScreen);
            bindings.insert("ctrl-up".to_string(), TerminalAction::ScrollUp);
            bindings.insert("ctrl-down".to_string(), TerminalAction::ScrollDown);
            bindings.insert("shift-pageup".to_string(), TerminalAction::ScrollPageUp);
            bindings.insert("shift-pagedown".to_string(), TerminalAction::ScrollPageDown);
            bindings.insert("ctrl-home".to_string(), TerminalAction::ScrollToTop);
            bindings.insert("ctrl-end".to_string(), TerminalAction::ScrollToBottom);
            bindings.insert("ctrl-=".to_string(), TerminalAction::ZoomIn);
            bindings.insert("ctrl--".to_string(), TerminalAction::ZoomOut);
            bindings.insert("ctrl-0".to_string(), TerminalAction::ResetZoom);
        }

        Self { bindings }
    }
}

impl Keybindings {
    /// Create new keybindings with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the action for a given key combination
    pub fn get_action(&self, key: &str) -> Option<&TerminalAction> {
        self.bindings.get(key)
    }

    /// Set a keybinding
    pub fn set_binding(&mut self, key: String, action: TerminalAction) {
        self.bindings.insert(key, action);
    }

    /// Remove a keybinding
    pub fn remove_binding(&mut self, key: &str) {
        self.bindings.remove(key);
    }

    /// Get all bindings for a specific action
    pub fn get_bindings_for_action(&self, action: &TerminalAction) -> Vec<&String> {
        self.bindings
            .iter()
            .filter(|(_, a)| *a == action)
            .map(|(k, _)| k)
            .collect()
    }
}
