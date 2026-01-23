//! Main application window

use crate::app::{NewTab, CloseTab, NextTab, PrevTab};
use crate::workspace::Workspace;
use axon_ui::{TerminalTabBar, TabInfo, TitleBar};
use gpui::*;

/// The main application window
pub struct MainWindow {
    /// The workspace containing terminals
    workspace: Entity<Workspace>,

    /// Focus handle
    focus_handle: FocusHandle,
}

impl MainWindow {
    /// Create a new main window
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            workspace,
            focus_handle,
        }
    }

    fn handle_new_tab(&mut self, _: &NewTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.new_tab(cx);
        });
    }

    fn handle_close_tab(&mut self, _: &CloseTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.close_active_tab(cx);
        });
    }

    fn handle_next_tab(&mut self, _: &NextTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.next_tab(cx);
        });
    }

    fn handle_prev_tab(&mut self, _: &PrevTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.prev_tab(cx);
        });
    }
}

impl Focusable for MainWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.read(cx);

        // Get tab information
        let tabs: Vec<TabInfo> = workspace
            .tabs()
            .iter()
            .enumerate()
            .map(|(i, tab)| TabInfo {
                id: i,
                title: tab.title.clone(),
                active: i == workspace.active_tab_index(),
            })
            .collect();

        let active_terminal_view = workspace.active_terminal_view();

        div()
            .id("main-window")
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1a1a1a))
            .text_color(rgb(0xe0e0e0))
            .track_focus(&self.focus_handle)
            .key_context("MainWindow")
            .on_action(cx.listener(Self::handle_new_tab))
            .on_action(cx.listener(Self::handle_close_tab))
            .on_action(cx.listener(Self::handle_next_tab))
            .on_action(cx.listener(Self::handle_prev_tab))
            // Custom title bar
            .child(Component::new(TitleBar::new("Axon Terminal")))
            // Tab bar
            .child(Component::new(TerminalTabBar::new().tabs(tabs)))
            // Terminal content
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(if let Some(view) = active_terminal_view {
                        view.clone().into_any_element()
                    } else {
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_full()
                            .child("No terminal open")
                            .into_any_element()
                    })
            )
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
                    .child(
                        div()
                            .flex_1()
                            .child(workspace.active_working_directory().unwrap_or_default())
                    )
                    .child(
                        div()
                            .child(format!("Tab {}/{}", workspace.active_tab_index() + 1, workspace.tabs().len()))
                    )
            )
    }
}
