//! Main application window

use crate::app::{CloseActiveTab, FocusTerminal, NewTab, NextTab, PrevTab, Quit};
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
