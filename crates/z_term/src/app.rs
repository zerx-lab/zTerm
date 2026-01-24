//! Application initialization and management

use crate::window::MainWindow;
use crate::workspace::Workspace;
use zterm_common::AppSettings;
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
        FocusTerminal,
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

    /// Watch for configuration changes and rebind keybindings when needed
    fn watch_config_changes(cx: &mut App) {
        // Track the last seen change counter
        let mut last_counter = cx
            .try_global::<AppSettings>()
            .map(|s| s.change_counter)
            .unwrap_or(0);

        cx.spawn(async move |cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(200))
                    .await;

                // Check if config has changed
                let current_counter = cx.update(|cx| {
                    cx.try_global::<AppSettings>()
                        .map(|s| s.change_counter)
                        .unwrap_or(0)
                });

                if current_counter != last_counter {
                    last_counter = current_counter;
                    tracing::info!("Config changed (counter: {}), rebinding keybindings...", current_counter);
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

    /// Convert config keybinding format ("ctrl+t") to GPUI format ("ctrl-t")
    fn normalize_keybinding(key: &str) -> String {
        key.replace('+', "-").to_lowercase()
    }

    /// Set up global key bindings from configuration
    fn setup_keybindings(cx: &mut App) {
        let config = zterm_common::Config::global();
        let kb = &config.keybindings;

        // System keybindings (not configurable)
        let mut bindings = vec![
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-q", Quit, None),
            #[cfg(not(target_os = "macos"))]
            KeyBinding::new("alt-f4", Quit, None),
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-n", NewWindow, None),
            #[cfg(not(target_os = "macos"))]
            KeyBinding::new("ctrl-shift-n", NewWindow, None),
        ];

        // Configurable keybindings from config file
        bindings.push(KeyBinding::new(
            &Self::normalize_keybinding(&kb.new_tab),
            NewTab,
            Some("MainWindow"),
        ));
        bindings.push(KeyBinding::new(
            &Self::normalize_keybinding(&kb.close_tab),
            CloseActiveTab,
            Some("MainWindow"),
        ));
        bindings.push(KeyBinding::new(
            &Self::normalize_keybinding(&kb.next_tab),
            NextTab,
            Some("MainWindow"),
        ));
        bindings.push(KeyBinding::new(
            &Self::normalize_keybinding(&kb.prev_tab),
            PrevTab,
            Some("MainWindow"),
        ));

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
            let main_window = cx.new(|cx| MainWindow::new(workspace, cx));

            // Show window after content is ready to avoid transparent flash
            window.activate_window();

            main_window
        })
        .unwrap();
    }
}
