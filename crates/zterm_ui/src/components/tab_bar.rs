//! Terminal tab bar component

use axon_ui::ThemeContext;
use crate::components::title_bar::TabInfo;
use gpui::prelude::*;
use gpui::*;
use gpui_component::h_flex;

/// Terminal tab bar component
pub struct TerminalTabBar {
    tabs: Vec<TabInfo>,
    scroll_handle: Option<ScrollHandle>,
}

impl TerminalTabBar {
    /// Create a new tab bar
    pub fn new() -> Self {
        Self {
            tabs: vec![],
            scroll_handle: None,
        }
    }

    /// Set the tabs
    pub fn tabs(mut self, tabs: Vec<TabInfo>) -> Self {
        self.tabs = tabs;
        self
    }

    /// Enable scroll tracking
    pub fn track_scroll(mut self, scroll_handle: &ScrollHandle) -> Self {
        self.scroll_handle = Some(scroll_handle.clone());
        self
    }
}

impl Default for TerminalTabBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for TerminalTabBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tabs = self.tabs.clone();
        let scroll_handle = self.scroll_handle.clone();

        // 获取主题颜色
        let theme = cx.current_theme();
        let colors = &theme.colors;
        let tab_bar_bg = colors.tab_bar_background.to_rgb();
        let border_color = colors.border.to_rgb();
        let tab_active_bg = colors.tab_active_background.to_rgb();
        let tab_inactive_bg = colors.tab_inactive_background.to_rgb();
        let tab_hover_bg = colors.tab_hover_background.to_rgb();
        let text_color = colors.text.to_rgb();
        let text_muted = colors.text_muted.to_rgb();
        let button_hover_bg = colors.button_hover_background.to_rgb();

        div()
            .id("tab-bar")
            .flex()
            .flex_row()
            .items_center()
            .h(px(36.0))
            .w_full()
            .bg(tab_bar_bg)
            .border_b_1()
            .border_color(border_color)
            // Scrollable tabs container
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .overflow_x_hidden()
                    .child(
                        h_flex()
                            .id("tabs-container")
                            .h_full()
                            .overflow_x_scroll()
                            .when_some(scroll_handle, |this, handle| this.track_scroll(&handle))
                            .children(tabs.into_iter().map(|tab| {
                                let bg_color = if tab.active {
                                    tab_active_bg
                                } else {
                                    tab_inactive_bg
                                };
                                let tab_text_color = if tab.active {
                                    text_color
                                } else {
                                    text_muted
                                };

                                div()
                                    .id(ElementId::Name(format!("tab-{}", tab.id).into()))
                                    .flex()
                                    .flex_shrink_0()
                                    .items_center()
                                    .px_3()
                                    .py_1()
                                    .mx_px()
                                    .rounded_t_md()
                                    .bg(bg_color)
                                    .hover(|style| style.bg(tab_hover_bg))
                                    .cursor_pointer()
                                    // Min/Max width constraints - prevents tabs from being too wide or too narrow
                                    .min_w(px(80.0))
                                    .max_w(px(200.0))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .justify_center()
                                            .min_w_0() // Allow text truncation
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(tab_text_color)
                                                    .truncate()
                                                    .child(tab.title),
                                            ),
                                    )
                            })),
                    ),
            )
            // New tab button
            .child(
                div()
                    .id("new-tab-button")
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .w(px(28.0))
                    .h(px(28.0))
                    .ml_2()
                    .mr_2()
                    .rounded_md()
                    .hover(|style| style.bg(button_hover_bg))
                    .cursor_pointer()
                    .child(div().text_sm().text_color(text_muted).child("+")),
            )
    }
}

// Note: GPUI component tests require #[gpui::test] and TestAppContext.
// Basic unit tests are in a separate test file to avoid macro expansion issues.
