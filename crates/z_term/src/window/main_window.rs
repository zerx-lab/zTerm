//! Main application window

use crate::app::{
    CloseActiveTab, CommandPalette, FocusTerminal, GotoTab1, GotoTab2, GotoTab3, GotoTab4,
    GotoTab5, GotoTab6, GotoTab7, GotoTab8, GotoTab9, NewTab, NextTab, PrevTab, Quit, ResetZoom,
    SplitHorizontal, SplitVertical, ToggleFullscreen, ZoomIn, ZoomOut,
};
use crate::workspace::Workspace;
use zterm_ui::{TabInfo, TitleBar, TitleBarEvent};
use gpui::*;

/// The main application window
pub struct MainWindow {
    /// The workspace containing terminals
    workspace: Entity<Workspace>,

    /// Title bar entity
    title_bar: Entity<TitleBar>,
}

impl MainWindow {
    /// Create a new main window
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let title_bar = cx.new(|_| TitleBar::new());

        // Subscribe to title bar events
        cx.subscribe(&title_bar, Self::handle_title_bar_event)
            .detach();

        Self {
            workspace,
            title_bar,
        }
    }

    /// Handle events from the title bar
    fn handle_title_bar_event(
        &mut self,
        _title_bar: Entity<TitleBar>,
        event: &TitleBarEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            TitleBarEvent::NewTab => {
                self.workspace.update(cx, |ws, cx| {
                    ws.new_tab(cx);
                });
                // Scroll to show the new tab (last one)
                let active_index = self.workspace.read(cx).active_tab_index();
                self.title_bar.update(cx, |title_bar, _| {
                    title_bar.scroll_to_tab(active_index);
                });
                // Focus the new terminal
                self.focus_active_terminal_deferred(cx);
            }
            TitleBarEvent::SelectTab(tab_index) => {
                let tab_index = *tab_index;
                self.workspace.update(cx, |ws, cx| {
                    ws.set_active_tab(tab_index, cx);
                });
                // Scroll to show the selected tab
                self.title_bar.update(cx, |title_bar, _| {
                    title_bar.scroll_to_tab(tab_index);
                });
                // Focus the selected terminal
                self.focus_active_terminal_deferred(cx);
            }
            TitleBarEvent::CloseTab(tab_index) => {
                let tab_index = *tab_index;
                let should_close_window = self
                    .workspace
                    .update(cx, |ws, cx| ws.close_tab(tab_index, cx));

                if should_close_window {
                    // Dispatch Quit action to close the app
                    cx.defer(|cx| {
                        cx.dispatch_action(&Quit);
                    });
                } else {
                    // Focus the new active terminal
                    self.focus_active_terminal_deferred(cx);
                }
            }
        }
    }

    /// Focus the active terminal view (deferred version for use without Window)
    fn focus_active_terminal_deferred(&self, cx: &mut Context<Self>) {
        // Dispatch FocusTerminal action to focus the terminal
        // This allows us to get the Window parameter in the action handler
        cx.defer(|cx| {
            cx.dispatch_action(&FocusTerminal);
        });
    }

    fn handle_focus_terminal(
        &mut self,
        _: &FocusTerminal,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_active_terminal(window, cx);
    }

    /// Focus the active terminal view
    fn focus_active_terminal(&self, window: &mut Window, cx: &mut Context<Self>) {
        // Clone the terminal view entity to avoid borrow issues
        let terminal_view = self.workspace.read(cx).active_terminal_view().cloned();

        if let Some(terminal_view) = terminal_view {
            terminal_view.update(cx, |view, cx| {
                window.focus(view.focus_handle_ref(), cx);
            });
        }
    }

    fn handle_new_tab(&mut self, _: &NewTab, window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.new_tab(cx);
        });
        // Scroll to show the new tab (last one)
        let active_index = self.workspace.read(cx).active_tab_index();
        self.title_bar.update(cx, |title_bar, _| {
            title_bar.scroll_to_tab(active_index);
        });
        // Focus the new terminal
        self.focus_active_terminal(window, cx);
    }

    fn handle_close_active_tab(
        &mut self,
        _: &CloseActiveTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let should_close_window = self.workspace.update(cx, |ws, cx| ws.close_active_tab(cx));

        if should_close_window {
            window.remove_window();
        } else {
            // Refocus the new active terminal view
            self.focus_active_terminal(window, cx);
        }
    }

    fn handle_next_tab(&mut self, _: &NextTab, window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.next_tab(cx);
        });
        // Scroll to show the active tab
        let active_index = self.workspace.read(cx).active_tab_index();
        self.title_bar.update(cx, |title_bar, _| {
            title_bar.scroll_to_tab(active_index);
        });
        // Focus the new active terminal
        self.focus_active_terminal(window, cx);
    }

    fn handle_prev_tab(&mut self, _: &PrevTab, window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.prev_tab(cx);
        });
        // Scroll to show the active tab
        let active_index = self.workspace.read(cx).active_tab_index();
        self.title_bar.update(cx, |title_bar, _| {
            title_bar.scroll_to_tab(active_index);
        });
        // Focus the new active terminal
        self.focus_active_terminal(window, cx);
    }

    fn handle_toggle_fullscreen(
        &mut self,
        _: &ToggleFullscreen,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        window.toggle_fullscreen();
    }

    fn handle_split_horizontal(
        &mut self,
        _: &SplitHorizontal,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        // TODO: Implement horizontal split when pane system is ready
        tracing::info!("Split horizontal not yet implemented");
    }

    fn handle_split_vertical(
        &mut self,
        _: &SplitVertical,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        // TODO: Implement vertical split when pane system is ready
        tracing::info!("Split vertical not yet implemented");
    }

    fn handle_zoom_in(&mut self, _: &ZoomIn, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement zoom in
        tracing::info!("Zoom in not yet implemented");
    }

    fn handle_zoom_out(&mut self, _: &ZoomOut, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement zoom out
        tracing::info!("Zoom out not yet implemented");
    }

    fn handle_reset_zoom(&mut self, _: &ResetZoom, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement reset zoom
        tracing::info!("Reset zoom not yet implemented");
    }

    fn handle_command_palette(
        &mut self,
        _: &CommandPalette,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        // TODO: Implement command palette
        tracing::info!("Command palette not yet implemented");
    }

    /// Go to a specific tab by index (0-based)
    fn goto_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let tab_count = self.workspace.read(cx).tabs().len();
        if index < tab_count {
            self.workspace.update(cx, |ws, cx| {
                ws.set_active_tab(index, cx);
            });
            self.title_bar.update(cx, |title_bar, _| {
                title_bar.scroll_to_tab(index);
            });
            self.focus_active_terminal(window, cx);
        }
    }

    fn handle_goto_tab_1(&mut self, _: &GotoTab1, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(0, window, cx);
    }

    fn handle_goto_tab_2(&mut self, _: &GotoTab2, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(1, window, cx);
    }

    fn handle_goto_tab_3(&mut self, _: &GotoTab3, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(2, window, cx);
    }

    fn handle_goto_tab_4(&mut self, _: &GotoTab4, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(3, window, cx);
    }

    fn handle_goto_tab_5(&mut self, _: &GotoTab5, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(4, window, cx);
    }

    fn handle_goto_tab_6(&mut self, _: &GotoTab6, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(5, window, cx);
    }

    fn handle_goto_tab_7(&mut self, _: &GotoTab7, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(6, window, cx);
    }

    fn handle_goto_tab_8(&mut self, _: &GotoTab8, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(7, window, cx);
    }

    fn handle_goto_tab_9(&mut self, _: &GotoTab9, window: &mut Window, cx: &mut Context<Self>) {
        // Ctrl+9 goes to the last tab (like browsers)
        let last_index = self.workspace.read(cx).tabs().len().saturating_sub(1);
        self.goto_tab(last_index, window, cx);
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get tab information from workspace
        let (tabs, active_tab_index, tab_count, working_dir) = {
            let workspace = self.workspace.read(cx);
            let tabs: Vec<TabInfo> = workspace
                .tabs()
                .iter()
                .enumerate()
                .map(|(i, tab)| {
                    let terminal = tab.terminal.read(cx);
                    let shell_name = terminal.shell_name();
                    let working_directory =
                        terminal.working_directory().to_string_lossy().to_string();
                    // Use TabInfo::new for pre-computed display_directory (better perf)
                    TabInfo::new(
                        i,
                        tab.title.clone(),
                        i == workspace.active_tab_index(),
                        shell_name,
                        working_directory,
                    )
                })
                .collect();
            let active_tab_index = workspace.active_tab_index();
            let tab_count = workspace.tabs().len();
            let working_dir = workspace.active_working_directory().unwrap_or_default();
            (tabs, active_tab_index, tab_count, working_dir)
        };

        // Update title bar tabs
        self.title_bar.update(cx, |title_bar, _| {
            title_bar.tabs = tabs;
        });

        // Get active terminal view separately after the mutable borrow is done
        let active_terminal_view = self.workspace.read(cx).active_terminal_view();

        div()
            .id("main-window")
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1a1a1a))
            .text_color(rgb(0xe0e0e0))
            // NOTE: 不要在窗口根节点 track_focus。
            // 在 Windows 上，自绘 titlebar 依赖 WM_NCHITTEST -> HTCAPTION 的默认行为来触发系统拖拽。
            // 而 `track_focus` 会在 MouseDown 时自动 focus 并调用 window.prevent_default()，
            // 进而导致 WM_NCLBUTTONDOWN 被 GPUI 标记为 handled，阻止系统开始拖拽。
            .key_context("MainWindow")
            .on_action(cx.listener(Self::handle_new_tab))
            .on_action(cx.listener(Self::handle_close_active_tab))
            .on_action(cx.listener(Self::handle_next_tab))
            .on_action(cx.listener(Self::handle_prev_tab))
            .on_action(cx.listener(Self::handle_focus_terminal))
            .on_action(cx.listener(Self::handle_toggle_fullscreen))
            .on_action(cx.listener(Self::handle_split_horizontal))
            .on_action(cx.listener(Self::handle_split_vertical))
            .on_action(cx.listener(Self::handle_zoom_in))
            .on_action(cx.listener(Self::handle_zoom_out))
            .on_action(cx.listener(Self::handle_reset_zoom))
            .on_action(cx.listener(Self::handle_command_palette))
            // Tab switching (Ctrl+1-9)
            .on_action(cx.listener(Self::handle_goto_tab_1))
            .on_action(cx.listener(Self::handle_goto_tab_2))
            .on_action(cx.listener(Self::handle_goto_tab_3))
            .on_action(cx.listener(Self::handle_goto_tab_4))
            .on_action(cx.listener(Self::handle_goto_tab_5))
            .on_action(cx.listener(Self::handle_goto_tab_6))
            .on_action(cx.listener(Self::handle_goto_tab_7))
            .on_action(cx.listener(Self::handle_goto_tab_8))
            .on_action(cx.listener(Self::handle_goto_tab_9))
            // Title bar with integrated tabs (like Warp Terminal)
            .child(self.title_bar.clone())
            // Terminal content
            .child(div().flex_1().overflow_hidden().child(
                if let Some(view) = active_terminal_view {
                    view.clone().into_any_element()
                } else {
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size_full()
                        .child("No terminal open")
                        .into_any_element()
                },
            ))
            // Status bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(24.0))
                    .px_2()
                    .bg(rgb(0x252525))
                    .border_t_1()
                    .border_color(rgb(0x333333))
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(div().flex_1().child(working_dir))
                    .child(div().child(format!("Tab {}/{}", active_tab_index + 1, tab_count))),
            )
    }
}
