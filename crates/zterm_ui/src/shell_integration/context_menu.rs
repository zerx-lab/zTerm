//! Context menu for shell integration
//!
//! This module provides utilities for building context menus
//! when right-clicking on command zones.

use zterm_terminal::shell_integration::CommandState;

/// Actions available in the context menu
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuAction {
    /// Copy selected text (普通复制)
    Copy,
    /// Paste from clipboard (粘贴)
    Paste,
    /// Copy the command text
    CopyCommand,
    /// Copy the command output
    CopyOutput,
    /// Copy both command and output
    CopyAll,
    /// Re-run the command
    RerunCommand,
    /// Open the working directory
    OpenDirectory,
    /// Send command to AI for explanation
    ExplainWithAi,
    /// Send command output to AI for analysis
    AnalyzeOutputWithAi,
    /// Send error to AI for debugging help
    DebugErrorWithAi,
    /// Custom action with identifier
    Custom(String),
}

impl ContextMenuAction {
    /// Get the display label for this action
    pub fn label(&self) -> &str {
        match self {
            ContextMenuAction::Copy => "Copy",
            ContextMenuAction::Paste => "Paste",
            ContextMenuAction::CopyCommand => "Copy Command",
            ContextMenuAction::CopyOutput => "Copy Output",
            ContextMenuAction::CopyAll => "Copy All",
            ContextMenuAction::RerunCommand => "Re-run Command",
            ContextMenuAction::OpenDirectory => "Open Directory",
            ContextMenuAction::ExplainWithAi => "Explain with AI",
            ContextMenuAction::AnalyzeOutputWithAi => "Analyze Output with AI",
            ContextMenuAction::DebugErrorWithAi => "Debug Error with AI",
            ContextMenuAction::Custom(s) => s.as_str(),
        }
    }

    /// Get a keyboard shortcut hint (if any)
    pub fn shortcut_hint(&self) -> Option<&str> {
        match self {
            ContextMenuAction::Copy => Some("Ctrl+Shift+C"),
            ContextMenuAction::Paste => Some("Ctrl+Shift+V"),
            ContextMenuAction::CopyCommand => Some("Ctrl+Shift+C"),
            ContextMenuAction::CopyOutput => Some("Ctrl+Shift+O"),
            ContextMenuAction::RerunCommand => Some("Ctrl+R"),
            _ => None,
        }
    }

    /// Check if this action requires a command text
    pub fn requires_command(&self) -> bool {
        matches!(
            self,
            ContextMenuAction::CopyCommand
                | ContextMenuAction::RerunCommand
                | ContextMenuAction::ExplainWithAi
        )
    }

    /// Check if this action requires command output
    pub fn requires_output(&self) -> bool {
        matches!(
            self,
            ContextMenuAction::CopyOutput
                | ContextMenuAction::CopyAll
                | ContextMenuAction::AnalyzeOutputWithAi
                | ContextMenuAction::DebugErrorWithAi
        )
    }
}

/// A menu item in the context menu
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// The action to perform
    pub action: ContextMenuAction,
    /// Whether the item is enabled
    pub enabled: bool,
    /// Optional icon identifier
    pub icon: Option<String>,
}

impl MenuItem {
    /// Create a new enabled menu item
    pub fn new(action: ContextMenuAction) -> Self {
        Self {
            action,
            enabled: true,
            icon: None,
        }
    }

    /// Set the enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set an icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// A separator in the menu
#[derive(Debug, Clone)]
pub struct MenuSeparator;

/// An entry in the context menu (either an item or separator)
#[derive(Debug, Clone)]
pub enum MenuEntry {
    Item(MenuItem),
    Separator,
}

/// Context for building the menu
#[derive(Debug, Clone, Default)]
pub struct MenuContext {
    /// The command text (if available)
    pub command: Option<String>,
    /// Whether output is available
    pub has_output: bool,
    /// The command state
    pub state: Option<CommandState>,
    /// Working directory (if available)
    pub working_directory: Option<String>,
    /// Whether AI features are enabled
    pub ai_enabled: bool,
}

impl MenuContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the command
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set output availability
    pub fn with_output(mut self, has_output: bool) -> Self {
        self.has_output = has_output;
        self
    }

    /// Set the command state
    pub fn with_state(mut self, state: CommandState) -> Self {
        self.state = Some(state);
        self
    }

    /// Set the working directory
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Enable AI features
    pub fn with_ai(mut self, enabled: bool) -> Self {
        self.ai_enabled = enabled;
        self
    }

    /// Check if the command failed
    pub fn is_failure(&self) -> bool {
        matches!(
            self.state,
            Some(CommandState::CommandFinished(code)) if code != 0
        )
    }

    /// Check if the command is still running
    pub fn is_running(&self) -> bool {
        matches!(self.state, Some(CommandState::CommandExecuting))
    }
}

/// Build a context menu for a command zone
pub fn build_context_menu(context: &MenuContext) -> Vec<MenuEntry> {
    let mut entries = Vec::new();

    // Copy actions
    let has_command = context.command.is_some();

    entries.push(MenuEntry::Item(
        MenuItem::new(ContextMenuAction::CopyCommand)
            .with_enabled(has_command)
            .with_icon("copy"),
    ));

    entries.push(MenuEntry::Item(
        MenuItem::new(ContextMenuAction::CopyOutput)
            .with_enabled(context.has_output)
            .with_icon("copy"),
    ));

    entries.push(MenuEntry::Item(
        MenuItem::new(ContextMenuAction::CopyAll)
            .with_enabled(has_command || context.has_output)
            .with_icon("copy"),
    ));

    entries.push(MenuEntry::Separator);

    // Command actions
    entries.push(MenuEntry::Item(
        MenuItem::new(ContextMenuAction::RerunCommand)
            .with_enabled(has_command && !context.is_running())
            .with_icon("play"),
    ));

    if context.working_directory.is_some() {
        entries.push(MenuEntry::Item(
            MenuItem::new(ContextMenuAction::OpenDirectory).with_icon("folder"),
        ));
    }

    // AI actions
    if context.ai_enabled {
        entries.push(MenuEntry::Separator);

        entries.push(MenuEntry::Item(
            MenuItem::new(ContextMenuAction::ExplainWithAi)
                .with_enabled(has_command)
                .with_icon("sparkles"),
        ));

        entries.push(MenuEntry::Item(
            MenuItem::new(ContextMenuAction::AnalyzeOutputWithAi)
                .with_enabled(context.has_output)
                .with_icon("sparkles"),
        ));

        if context.is_failure() {
            entries.push(MenuEntry::Item(
                MenuItem::new(ContextMenuAction::DebugErrorWithAi)
                    .with_enabled(context.has_output)
                    .with_icon("bug"),
            ));
        }
    }

    entries
}

/// Count the number of enabled items in a menu
pub fn count_enabled_items(entries: &[MenuEntry]) -> usize {
    entries
        .iter()
        .filter(|e| matches!(e, MenuEntry::Item(item) if item.enabled))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ContextMenuAction Tests =====

    #[test]
    fn test_action_label() {
        assert_eq!(ContextMenuAction::CopyCommand.label(), "Copy Command");
        assert_eq!(ContextMenuAction::CopyOutput.label(), "Copy Output");
        assert_eq!(
            ContextMenuAction::Custom("Test".to_string()).label(),
            "Test"
        );
    }

    #[test]
    fn test_action_shortcut_hint() {
        assert!(ContextMenuAction::CopyCommand.shortcut_hint().is_some());
        assert!(ContextMenuAction::OpenDirectory.shortcut_hint().is_none());
    }

    #[test]
    fn test_action_requires_command() {
        assert!(ContextMenuAction::CopyCommand.requires_command());
        assert!(ContextMenuAction::RerunCommand.requires_command());
        assert!(!ContextMenuAction::CopyOutput.requires_command());
    }

    #[test]
    fn test_action_requires_output() {
        assert!(ContextMenuAction::CopyOutput.requires_output());
        assert!(ContextMenuAction::DebugErrorWithAi.requires_output());
        assert!(!ContextMenuAction::CopyCommand.requires_output());
    }

    // ===== MenuItem Tests =====

    #[test]
    fn test_menu_item_new() {
        let item = MenuItem::new(ContextMenuAction::CopyCommand);
        assert!(item.enabled);
        assert!(item.icon.is_none());
    }

    #[test]
    fn test_menu_item_with_enabled() {
        let item = MenuItem::new(ContextMenuAction::CopyCommand).with_enabled(false);
        assert!(!item.enabled);
    }

    #[test]
    fn test_menu_item_with_icon() {
        let item = MenuItem::new(ContextMenuAction::CopyCommand).with_icon("copy");
        assert_eq!(item.icon.as_deref(), Some("copy"));
    }

    // ===== MenuContext Tests =====

    #[test]
    fn test_menu_context_default() {
        let ctx = MenuContext::default();
        assert!(ctx.command.is_none());
        assert!(!ctx.has_output);
        assert!(!ctx.ai_enabled);
    }

    #[test]
    fn test_menu_context_builder() {
        let ctx = MenuContext::new()
            .with_command("ls -la")
            .with_output(true)
            .with_ai(true)
            .with_working_directory("/home/user");

        assert_eq!(ctx.command.as_deref(), Some("ls -la"));
        assert!(ctx.has_output);
        assert!(ctx.ai_enabled);
        assert_eq!(ctx.working_directory.as_deref(), Some("/home/user"));
    }

    #[test]
    fn test_menu_context_is_failure() {
        let ctx = MenuContext::new().with_state(CommandState::CommandFinished(1));
        assert!(ctx.is_failure());

        let ctx = MenuContext::new().with_state(CommandState::CommandFinished(0));
        assert!(!ctx.is_failure());
    }

    #[test]
    fn test_menu_context_is_running() {
        let ctx = MenuContext::new().with_state(CommandState::CommandExecuting);
        assert!(ctx.is_running());

        let ctx = MenuContext::new().with_state(CommandState::CommandFinished(0));
        assert!(!ctx.is_running());
    }

    // ===== build_context_menu Tests =====

    #[test]
    fn test_build_menu_minimal() {
        let ctx = MenuContext::new();
        let menu = build_context_menu(&ctx);

        assert!(!menu.is_empty());
        // Copy command should be disabled
        if let MenuEntry::Item(item) = &menu[0] {
            assert!(!item.enabled);
        }
    }

    #[test]
    fn test_build_menu_with_command() {
        let ctx = MenuContext::new().with_command("ls");
        let menu = build_context_menu(&ctx);

        // Copy command should be enabled
        if let MenuEntry::Item(item) = &menu[0] {
            assert!(item.enabled);
            assert_eq!(item.action, ContextMenuAction::CopyCommand);
        }
    }

    #[test]
    fn test_build_menu_with_output() {
        let ctx = MenuContext::new().with_output(true);
        let menu = build_context_menu(&ctx);

        // Find copy output action
        let copy_output = menu.iter().find(
            |e| matches!(e, MenuEntry::Item(item) if item.action == ContextMenuAction::CopyOutput),
        );
        assert!(copy_output.is_some());
        if let MenuEntry::Item(item) = copy_output.unwrap() {
            assert!(item.enabled);
        }
    }

    #[test]
    fn test_build_menu_with_ai() {
        let ctx = MenuContext::new().with_command("ls").with_ai(true);
        let menu = build_context_menu(&ctx);

        // Should have AI actions
        let has_ai_action = menu.iter().any(|e| {
            matches!(e, MenuEntry::Item(item) if item.action == ContextMenuAction::ExplainWithAi)
        });
        assert!(has_ai_action);
    }

    #[test]
    fn test_build_menu_debug_error_on_failure() {
        let ctx = MenuContext::new()
            .with_output(true)
            .with_state(CommandState::CommandFinished(1))
            .with_ai(true);
        let menu = build_context_menu(&ctx);

        let has_debug = menu.iter().any(|e| {
            matches!(e, MenuEntry::Item(item) if item.action == ContextMenuAction::DebugErrorWithAi)
        });
        assert!(has_debug);
    }

    #[test]
    fn test_build_menu_no_debug_error_on_success() {
        let ctx = MenuContext::new()
            .with_output(true)
            .with_state(CommandState::CommandFinished(0))
            .with_ai(true);
        let menu = build_context_menu(&ctx);

        let has_debug = menu.iter().any(|e| {
            matches!(e, MenuEntry::Item(item) if item.action == ContextMenuAction::DebugErrorWithAi)
        });
        assert!(!has_debug);
    }

    #[test]
    fn test_build_menu_with_working_directory() {
        let ctx = MenuContext::new().with_working_directory("/home");
        let menu = build_context_menu(&ctx);

        let has_open_dir = menu.iter().any(|e| {
            matches!(e, MenuEntry::Item(item) if item.action == ContextMenuAction::OpenDirectory)
        });
        assert!(has_open_dir);
    }

    #[test]
    fn test_build_menu_rerun_disabled_when_running() {
        let ctx = MenuContext::new()
            .with_command("sleep 100")
            .with_state(CommandState::CommandExecuting);
        let menu = build_context_menu(&ctx);

        let rerun = menu.iter().find(|e| {
            matches!(e, MenuEntry::Item(item) if item.action == ContextMenuAction::RerunCommand)
        });
        assert!(rerun.is_some());
        if let MenuEntry::Item(item) = rerun.unwrap() {
            assert!(!item.enabled); // Should be disabled while running
        }
    }

    // ===== count_enabled_items Tests =====

    #[test]
    fn test_count_enabled_items() {
        let ctx = MenuContext::new().with_command("ls").with_output(true);
        let menu = build_context_menu(&ctx);

        let count = count_enabled_items(&menu);
        assert!(count > 0);
    }

    #[test]
    fn test_count_enabled_items_empty() {
        let entries: Vec<MenuEntry> = vec![MenuEntry::Separator];
        assert_eq!(count_enabled_items(&entries), 0);
    }
}
