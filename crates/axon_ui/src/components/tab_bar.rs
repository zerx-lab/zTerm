//! Terminal tab bar component

use gpui::*;

/// Tab information
#[derive(Clone)]
pub struct TabInfo {
    pub id: usize,
    pub title: String,
    pub active: bool,
    pub shell_name: String,
}

/// Terminal tab bar component
pub struct TerminalTabBar {
    tabs: Vec<TabInfo>,
}

impl TerminalTabBar {
    /// Create a new tab bar
    pub fn new() -> Self {
        Self { tabs: vec![] }
    }

    /// Set the tabs
    pub fn tabs(mut self, tabs: Vec<TabInfo>) -> Self {
        self.tabs = tabs;
        self
    }
}

impl Default for TerminalTabBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for TerminalTabBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tabs = self.tabs.clone();

        div()
            .id("tab-bar")
            .flex()
            .flex_row()
            .items_center()
            .h(px(36.0))
            .w_full()
            .bg(rgb(0x1e1e1e))
            .border_b_1()
            .border_color(rgb(0x333333))
            .px_2()
            .children(tabs.into_iter().map(|tab| {
                let bg_color = if tab.active {
                    rgb(0x2d2d2d)
                } else {
                    rgb(0x1e1e1e)
                };

                div()
                    .id(ElementId::Name(format!("tab-{}", tab.id).into()))
                    .flex()
                    .items_center()
                    .px_3()
                    .py_1()
                    .mx_px()
                    .rounded_t_md()
                    .bg(bg_color)
                    .hover(|style| style.bg(rgb(0x3d3d3d)))
                    .cursor_pointer()
                    .child(
                        div()
                            .text_sm()
                            .text_color(if tab.active {
                                rgb(0xffffff)
                            } else {
                                rgb(0x888888)
                            })
                            .child(tab.title),
                    )
            }))
            .child(
                // New tab button
                div()
                    .id("new-tab-button")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(28.0))
                    .h(px(28.0))
                    .ml_2()
                    .rounded_md()
                    .hover(|style| style.bg(rgb(0x3d3d3d)))
                    .cursor_pointer()
                    .child(div().text_sm().text_color(rgb(0x888888)).child("+")),
            )
    }
}

// Note: GPUI component tests require #[gpui::test] and TestAppContext.
// Basic unit tests are in a separate test file to avoid macro expansion issues.
