//! Application initialization and management

use crate::window::MainWindow;
use crate::workspace::Workspace;
use zterm_terminal::TerminalSize;
use gpui::*;
use gpui_component::theme::Theme;

actions!(
    zterm,
    [
        Quit,
        NewWindow,
        NewTab,
        CloseActiveTab,
        NextTab,
        PrevTab,
        SplitHorizontal,
        SplitVertical,
        ToggleFullscreen,
        ZoomIn,
        ZoomOut,
        ResetZoom,
    ]
);

/// Main application state
pub struct ZTermApp;

impl ZTermApp {
    /// Initialize the application
    pub fn init(cx: &mut App) {
        // Initialize theme (required for gpui_component)
        cx.set_global(Theme::default());

        // Register actions
        Self::register_actions(cx);

        // Set up global key bindings
        Self::setup_keybindings(cx);

        // Handle window close - quit when last window is closed
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
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

    /// Set up global key bindings
    fn setup_keybindings(cx: &mut App) {
        cx.bind_keys([
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-q", Quit, None),
            #[cfg(not(target_os = "macos"))]
            KeyBinding::new("alt-f4", Quit, None),
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-n", NewWindow, None),
            #[cfg(not(target_os = "macos"))]
            KeyBinding::new("ctrl-shift-n", NewWindow, None),
            // New tab
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-t", NewTab, Some("MainWindow")),
            #[cfg(not(target_os = "macos"))]
            KeyBinding::new("ctrl-t", NewTab, Some("MainWindow")),
            // Close active tab
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-w", CloseActiveTab, Some("MainWindow")),
            #[cfg(not(target_os = "macos"))]
            KeyBinding::new("ctrl-w", CloseActiveTab, Some("MainWindow")),
        ]);
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
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            is_movable: true,
            window_background: WindowBackgroundAppearance::Opaque,
            app_id: Some("zterm".to_string()),
            window_min_size: Some(size(px(400.0), px(300.0))),
            window_decorations: Some(WindowDecorations::Client),
            ..Default::default()
        };

        cx.open_window(window_options, |_window, cx| {
            // Create the workspace with initial terminal
            let workspace = cx.new(|cx| {
                let terminal_size = TerminalSize::default();
                Workspace::new(terminal_size, cx)
            });

            // Create and return the main window view
            cx.new(|cx| MainWindow::new(workspace, cx))
        })
        .unwrap();
    }
}
