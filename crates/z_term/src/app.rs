//! Application initialization and management

use crate::window::MainWindow;
use crate::workspace::Workspace;
use gpui::*;
use gpui_component::theme::Theme;
use zterm_common::AppSettings;
use zterm_terminal::TerminalSize;

// Terminal-specific actions are defined in zterm_ui::components::terminal_view
// and re-exported here for convenience
pub use zterm_ui::{
    Copy, Paste, ScrollDown, ScrollPageDown, ScrollPageUp, ScrollToBottom, ScrollToTop, ScrollUp,
    Search,
};

actions!(
    zterm,
    [
        // System
        Quit,
        NewWindow,
        // Tab management
        NewTab,
        CloseActiveTab,
        NextTab,
        PrevTab,
        // Window operations
        ToggleFullscreen,
        // Split pane
        SplitHorizontal,
        SplitVertical,
        // Zoom
        ZoomIn,
        ZoomOut,
        ResetZoom,
        // Other
        FocusTerminal, // Internal action, not configurable
        CommandPalette,
        // Tab switching (Ctrl+1-9)
        GotoTab1,
        GotoTab2,
        GotoTab3,
        GotoTab4,
        GotoTab5,
        GotoTab6,
        GotoTab7,
        GotoTab8,
        GotoTab9,
    ]
);

/// Main application state
pub struct ZTermApp;

impl ZTermApp {
    /// Initialize the application
    pub fn init(cx: &mut App) {
        // Initialize theme (required for gpui_component)
        cx.set_global(Theme::default());

        // Initialize our theme system
        axon_ui::ThemeManager::init(cx);

        // Load theme from config
        Self::load_theme_from_config(cx);

        // Register actions
        Self::register_actions(cx);

        // Set up global key bindings
        Self::setup_keybindings(cx);

        // Watch for config changes and rebind keys
        Self::watch_config_changes(cx);

        // Handle window close - quit when last window is closed
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
    }

    /// Load theme from configuration
    fn load_theme_from_config(cx: &mut App) {
        let config = zterm_common::Config::global();
        let theme_name = &config.ui.theme;

        tracing::info!("Loading theme from config: {}", theme_name);
        if !axon_ui::ThemeManager::set_theme_by_name(theme_name, cx) {
            tracing::warn!("Failed to load theme '{}', using default", theme_name);
        }
    }

    /// Watch for configuration changes and rebind keybindings when needed
    fn watch_config_changes(cx: &mut App) {
        // Track the last seen change counter and theme name
        let mut last_counter = cx
            .try_global::<AppSettings>()
            .map(|s| s.change_counter)
            .unwrap_or(0);

        let mut last_theme = zterm_common::Config::global().ui.theme.clone();

        cx.spawn(async move |cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(200))
                    .await;

                // Check if config has changed
                let (current_counter, current_theme) = cx.update(|cx| {
                    let counter = cx
                        .try_global::<AppSettings>()
                        .map(|s| s.change_counter)
                        .unwrap_or(0);
                    let theme = zterm_common::Config::global().ui.theme.clone();
                    (counter, theme)
                });

                if current_counter != last_counter {
                    last_counter = current_counter;

                    // Check if theme changed
                    if current_theme != last_theme {
                        last_theme = current_theme.clone();
                        tracing::info!("Theme changed to: {}", current_theme);
                        cx.update(|cx| {
                            Self::load_theme_from_config(cx);
                        });
                    }

                    tracing::info!(
                        "Config changed (counter: {}), rebinding keybindings...",
                        current_counter
                    );
                    cx.update(|cx| {
                        Self::setup_keybindings(cx);
                    });
                }
            }
        })
        .detach();
    }

    /// Register global actions
    fn register_actions(cx: &mut App) {
        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        cx.on_action(|_: &NewWindow, cx| {
            Self::open_main_window(cx);
        });
    }

    /// Set up global key bindings from configuration
    fn setup_keybindings(cx: &mut App) {
        use zterm_common::{ConfigurableAction, KeybindingsConfig};

        let config = zterm_common::Config::global();
        let kb = &config.keybindings;

        // Helper to create normalized keybinding
        let norm = |key: &str| KeybindingsConfig::normalize_keybinding(key);

        // Global keybindings (no context required)
        let mut bindings = vec![
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::Quit)),
                Quit,
                None,
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::NewWindow)),
                NewWindow,
                None,
            ),
        ];

        // MainWindow context keybindings (Tab management, Window operations, Zoom)
        bindings.extend([
            // Tab management
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::NewTab)),
                NewTab,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::CloseTab)),
                CloseActiveTab,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::NextTab)),
                NextTab,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::PrevTab)),
                PrevTab,
                Some("MainWindow"),
            ),
            // Window operations
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ToggleFullscreen)),
                ToggleFullscreen,
                Some("MainWindow"),
            ),
            // Split pane
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::SplitHorizontal)),
                SplitHorizontal,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::SplitVertical)),
                SplitVertical,
                Some("MainWindow"),
            ),
            // Zoom
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ZoomIn)),
                ZoomIn,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ZoomOut)),
                ZoomOut,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ResetZoom)),
                ResetZoom,
                Some("MainWindow"),
            ),
            // Other
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::CommandPalette)),
                CommandPalette,
                Some("MainWindow"),
            ),
            // Tab switching (Ctrl+1-9)
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab1)),
                GotoTab1,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab2)),
                GotoTab2,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab3)),
                GotoTab3,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab4)),
                GotoTab4,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab5)),
                GotoTab5,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab6)),
                GotoTab6,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab7)),
                GotoTab7,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab8)),
                GotoTab8,
                Some("MainWindow"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::GotoTab9)),
                GotoTab9,
                Some("MainWindow"),
            ),
        ]);

        // Terminal context keybindings (Terminal operations, Scrolling)
        bindings.extend([
            // Terminal operations
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::Copy)),
                Copy,
                Some("Terminal"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::Paste)),
                Paste,
                Some("Terminal"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::Search)),
                Search,
                Some("Terminal"),
            ),
            // Scrolling
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ScrollUp)),
                ScrollUp,
                Some("Terminal"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ScrollDown)),
                ScrollDown,
                Some("Terminal"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ScrollPageUp)),
                ScrollPageUp,
                Some("Terminal"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ScrollPageDown)),
                ScrollPageDown,
                Some("Terminal"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ScrollToTop)),
                ScrollToTop,
                Some("Terminal"),
            ),
            KeyBinding::new(
                &norm(kb.get_keybinding(ConfigurableAction::ScrollToBottom)),
                ScrollToBottom,
                Some("Terminal"),
            ),
        ]);

        cx.bind_keys(bindings);
    }

    /// Open the main application window
    pub fn open_main_window(cx: &mut App) {
        let config = zterm_common::Config::global();

        let window_options = WindowOptions {
            titlebar: None, // We use custom title bar
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(
                    px(config.ui.window_width as f32),
                    px(config.ui.window_height as f32),
                ),
                cx,
            ))),
            focus: false,
            show: false, // Don't show immediately, wait for content to be ready
            kind: WindowKind::Normal,
            is_movable: true,
            window_background: WindowBackgroundAppearance::Opaque,
            app_id: Some("zterm".to_string()),
            window_min_size: Some(size(px(400.0), px(300.0))),
            window_decorations: Some(WindowDecorations::Client),
            ..Default::default()
        };

        cx.open_window(window_options, |window, cx| {
            // Create the workspace with initial terminal
            let workspace = cx.new(|cx| {
                let terminal_size = TerminalSize::default();
                Workspace::new(terminal_size, cx)
            });

            // Create and return the main window view
            let main_window = cx.new(|cx| MainWindow::new(workspace.clone(), cx));

            // Focus the terminal view so keyboard input works immediately
            if let Some(terminal_view) = workspace.read(cx).active_terminal_view() {
                let focus_handle = terminal_view.read(cx).focus_handle_ref().clone();
                window.focus(&focus_handle, cx);
            }

            // Show window after content is ready to avoid transparent flash
            window.activate_window();

            main_window
        })
        .unwrap();
    }
}
