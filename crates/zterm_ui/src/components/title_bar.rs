//! Custom title bar component for window dragging
//!
//! This implementation is based on Zed's platform_title_bar pattern for reliable
//! cross-platform window dragging support.

use axon_ui::ThemeContext;
use gpui::{prelude::*, *};
use gpui_component::h_flex;

// Define simple actions for tab bar interactions
actions!(title_bar, [NewTab]);

/// Events emitted by the title bar
#[derive(Debug, Clone)]
pub enum TitleBarEvent {
    /// Request to create a new tab
    NewTab,
    /// Request to select a specific tab
    SelectTab(usize),
    /// Request to close a specific tab
    CloseTab(usize),
}

/// Height of the title bar in pixels
pub const TITLE_BAR_HEIGHT: Pixels = px(32.0);

/// Minimum width for each tab to ensure readability
const TAB_MIN_WIDTH: Pixels = px(100.0);

/// Maximum width for each tab to manage space efficiently
const TAB_MAX_WIDTH: Pixels = px(200.0);

/// Tab information for display in title bar
#[derive(Clone)]
pub struct TabInfo {
    pub id: usize,
    pub title: String,
    pub active: bool,
}

impl TabInfo {
    /// Create a new TabInfo
    pub fn new(id: usize, title: String, active: bool) -> Self {
        Self { id, title, active }
    }
}

/// Platform style enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformStyle {
    Mac,
    Linux,
    Windows,
}

impl PlatformStyle {
    pub fn platform() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
            Self::Linux
        } else {
            Self::Windows
        }
    }
}

/// Custom title bar component that supports window dragging on all platforms.
///
/// Based on Zed's PlatformTitleBar implementation for reliable Windows support.
pub struct TitleBar {
    /// Tabs to display in the title bar
    pub tabs: Vec<TabInfo>,
    should_move: bool,
    /// Scroll handle for horizontal tab scrolling
    scroll_handle: ScrollHandle,
}

impl EventEmitter<TitleBarEvent> for TitleBar {}

impl TitleBar {
    /// Create a new title bar
    pub fn new() -> Self {
        Self {
            tabs: vec![],
            should_move: false,
            scroll_handle: ScrollHandle::new(),
        }
    }

    /// Set the tabs to display
    pub fn tabs(mut self, tabs: Vec<TabInfo>) -> Self {
        self.tabs = tabs;
        self
    }

    /// Get the height of the title bar
    pub fn height() -> Pixels {
        TITLE_BAR_HEIGHT
    }

    /// Handle tab selection
    fn on_select_tab(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        cx.emit(TitleBarEvent::SelectTab(tab_index));
    }

    /// Handle tab close
    fn on_close_tab(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        cx.emit(TitleBarEvent::CloseTab(tab_index));
    }

    /// Handle new tab
    fn on_new_tab(&mut self, cx: &mut Context<Self>) {
        cx.emit(TitleBarEvent::NewTab);
    }

    /// Scroll to make the specified tab visible
    /// Call this after changing the active tab to ensure it's in view
    pub fn scroll_to_tab(&self, tab_index: usize) {
        self.scroll_handle.scroll_to_item(tab_index);
    }
}

impl Default for TitleBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for TitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let platform_style = PlatformStyle::platform();
        let tabs = self.tabs.clone();
        let is_linux = platform_style == PlatformStyle::Linux;
        let is_mac = platform_style == PlatformStyle::Mac;
        let is_fullscreen = window.is_fullscreen();
        let height = Self::height();

        // 获取主题颜色
        let theme = cx.current_theme();
        let colors = &theme.colors;
        let titlebar_bg = colors.titlebar_background.to_rgb();
        let border_color = colors.border.to_rgb();
        let tab_active_bg = colors.tab_active_background.to_rgb();
        let tab_inactive_bg = colors.tab_inactive_background.to_rgb();
        let tab_hover_bg = colors.tab_hover_background.to_rgb();
        let tab_active_indicator = colors.tab_active_indicator.to_rgb();
        let text_color = colors.text.to_rgb();
        let text_muted = colors.text_muted.to_rgb();
        let button_hover_bg = colors.button_hover_background.to_rgb();
        let button_active_bg = colors.button_active_background.to_rgb();

        // Main title bar container - follows Zed's h_flex() pattern exactly
        // window_control_area(Drag) tells Windows to return HTCAPTION for WM_NCHITTEST
        div()
            .id("title-bar")
            .flex()
            .flex_row()
            .items_center()
            .window_control_area(WindowControlArea::Drag)
            .w_full()
            .h(height)
            .bg(titlebar_bg)
            .border_b_1()
            .border_color(border_color)
            .content_stretch()
            // Mouse event handlers for non-Windows platforms
            .on_mouse_down_out(
                cx.listener(move |this, _ev: &MouseDownEvent, _window, _cx| {
                    this.should_move = false;
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |this, _ev: &MouseUpEvent, _window, _cx| {
                    this.should_move = false;
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _ev: &MouseDownEvent, _window, _cx| {
                    this.should_move = true;
                }),
            )
            .on_mouse_move(cx.listener(move |this, _ev: &MouseMoveEvent, window, _| {
                if this.should_move {
                    this.should_move = false;
                    window.start_window_move();
                }
            }))
            // Double-click handlers for non-Windows platforms
            .when(is_linux, |this: Stateful<Div>| {
                this.on_click(|event: &ClickEvent, window: &mut Window, _: &mut App| {
                    if event.click_count() == 2 {
                        window.zoom_window();
                    }
                })
            })
            .when(is_mac, |this: Stateful<Div>| {
                this.on_click(|event: &ClickEvent, window: &mut Window, _: &mut App| {
                    if event.click_count() == 2 {
                        window.titlebar_double_click();
                    }
                })
            })
            // Scrollable tabs container - follows Zed's pane.rs pattern
            // GPUI automatically converts vertical scroll to horizontal when only overflow_x_scroll is set
            .child(
                h_flex()
                    .id("tabs-scroll-container")
                    .flex_1()
                    .h_full()
                    .min_w_0() // Allow shrinking below content size
                    .items_center()
                    .overflow_x_scroll()
                    .track_scroll(&self.scroll_handle)
                    .px_2()
                    .gap_1()
                    .children(tabs.into_iter().enumerate().map(|(tab_index, tab)| {
                        let tab_id = tab.id;
                        let is_active = tab.active;
                        let tab_title = tab.title.clone();

                        // 使用主题颜色
                        let bg_color = if is_active {
                            tab_active_bg
                        } else {
                            tab_inactive_bg
                        };
                        let tab_text_color = if is_active { text_color } else { text_muted };
                        let hover_bg = tab_hover_bg;

                        div()
                            // Use NamedInteger for ~10x faster ElementId creation
                            .id(ElementId::NamedInteger("tab".into(), tab_id as u64))
                            .flex()
                            .flex_row()
                            .flex_shrink_0() // Prevent tabs from shrinking
                            .items_center()
                            .justify_between()
                            .h(px(28.0))
                            .min_w(TAB_MIN_WIDTH)
                            .max_w(TAB_MAX_WIDTH)
                            .px(px(10.0))
                            .rounded_t_md()
                            .bg(bg_color)
                            // Active tab indicator - subtle bottom border
                            .when(is_active, |el| {
                                el.border_b_2().border_color(tab_active_indicator)
                            })
                            .when(!is_active, |el| el.border_b_1().border_color(border_color))
                            .hover(|style| style.bg(hover_bg))
                            .cursor_pointer()
                            .block_mouse_except_scroll()
                            .on_mouse_down(
                                MouseButton::Left,
                                |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                },
                            )
                            .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                                this.on_select_tab(tab_index, cx);
                            }))
                            // Tab title - takes most space
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(tab_text_color)
                                    .truncate()
                                    .overflow_hidden()
                                    .child(tab_title),
                            )
                            // Close button - compact and subtle
                            .child(
                                div()
                                    // Use NamedInteger for ~10x faster ElementId creation
                                    .id(ElementId::NamedInteger("close-tab".into(), tab_id as u64))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .ml(px(6.0))
                                    .rounded(px(4.0))
                                    .text_xs()
                                    .text_color(text_muted)
                                    .hover(|style| style.bg(button_hover_bg).text_color(text_color))
                                    .active(|style| style.bg(button_active_bg))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                            window.prevent_default();
                                            cx.stop_propagation();
                                        },
                                    )
                                    .on_click(cx.listener(
                                        move |this, _: &ClickEvent, _window, cx| {
                                            cx.stop_propagation();
                                            this.on_close_tab(tab_index, cx);
                                        },
                                    ))
                                    .child("×"),
                            )
                    }))
                    // New tab button - minimal style, stays at end of tabs
                    .child(
                        div()
                            .id("new-tab-button")
                            .flex()
                            .flex_shrink_0() // Prevent button from shrinking
                            .items_center()
                            .justify_center()
                            .w(px(28.0))
                            .h(px(28.0))
                            .ml(px(4.0))
                            .rounded(px(6.0))
                            .text_color(text_muted)
                            .hover(|style| style.bg(button_hover_bg).text_color(text_color))
                            .active(|style| style.bg(button_active_bg))
                            .cursor_pointer()
                            .block_mouse_except_scroll()
                            .on_mouse_down(
                                MouseButton::Left,
                                |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                },
                            )
                            .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                                this.on_new_tab(cx);
                            }))
                            .child("+"),
                    ),
            )
            // Window controls (right side) - protected from shrinking
            .when(
                !is_fullscreen,
                |title_bar: Stateful<Div>| match platform_style {
                    PlatformStyle::Mac => title_bar,
                    PlatformStyle::Linux => title_bar.child(
                        div()
                            .flex_shrink_0() // Prevent window controls from being pushed out
                            .child(LinuxWindowControls::new()),
                    ),
                    PlatformStyle::Windows => title_bar.child(
                        div()
                            .flex_shrink_0() // Prevent window controls from being pushed out
                            .child(WindowsWindowControls::new(height)),
                    ),
                },
            )
    }
}

// =============================================================================
// Windows Window Controls - Based on Zed's implementation
// =============================================================================

#[derive(IntoElement)]
pub struct WindowsWindowControls {
    button_height: Pixels,
}

impl WindowsWindowControls {
    pub fn new(button_height: Pixels) -> Self {
        Self { button_height }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_font() -> &'static str {
        "Segoe Fluent Icons"
    }

    #[cfg(target_os = "windows")]
    fn get_font() -> &'static str {
        use windows::Wdk::System::SystemServices::RtlGetVersion;

        let mut version = unsafe { std::mem::zeroed() };
        let status = unsafe { RtlGetVersion(&mut version) };

        if status.is_ok() && version.dwBuildNumber >= 22000 {
            "Segoe Fluent Icons"
        } else {
            "Segoe MDL2 Assets"
        }
    }
}

impl RenderOnce for WindowsWindowControls {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let is_maximized = window.is_maximized();

        div()
            .id("windows-window-controls")
            .font_family(Self::get_font())
            .flex()
            .flex_row()
            .justify_center()
            .content_stretch()
            .max_h(self.button_height)
            .min_h(self.button_height)
            .child(WindowsCaptionButton::Minimize)
            .child(if is_maximized {
                WindowsCaptionButton::Restore
            } else {
                WindowsCaptionButton::Maximize
            })
            .child(WindowsCaptionButton::Close)
    }
}

#[derive(IntoElement)]
enum WindowsCaptionButton {
    Minimize,
    Restore,
    Maximize,
    Close,
}

impl WindowsCaptionButton {
    #[inline]
    fn id(&self) -> &'static str {
        match self {
            Self::Minimize => "minimize",
            Self::Restore => "restore",
            Self::Maximize => "maximize",
            Self::Close => "close",
        }
    }

    #[inline]
    fn icon(&self) -> &'static str {
        // Segoe Fluent Icons / Segoe MDL2 Assets unicode characters
        match self {
            Self::Minimize => "\u{e921}", // MinimizeWindow
            Self::Restore => "\u{e923}",  // RestoreWindow
            Self::Maximize => "\u{e922}", // MaximizeWindow
            Self::Close => "\u{e8bb}",    // ChromeClose
        }
    }

    #[inline]
    fn control_area(&self) -> WindowControlArea {
        match self {
            Self::Close => WindowControlArea::Close,
            Self::Maximize | Self::Restore => WindowControlArea::Max,
            Self::Minimize => WindowControlArea::Min,
        }
    }
}

impl RenderOnce for WindowsCaptionButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        // 获取主题颜色
        let theme = cx.current_theme();
        let colors = &theme.colors;

        let (hover_bg, hover_fg, active_bg, active_fg) = match self {
            Self::Close => {
                // 关闭按钮使用危险颜色
                let danger_bg = colors.danger.to_rgb();
                let danger_fg = colors.danger_foreground.to_rgb();
                let danger_active = colors.danger.opacity(0.6).to_rgb();
                let danger_active_fg = colors.danger_foreground.opacity(0.8).to_rgb();
                (danger_bg, danger_fg, danger_active, danger_active_fg)
            }
            _ => {
                // 最小化/最大化按钮使用主题颜色
                let hover_bg = colors.button_hover_background.to_rgb();
                let hover_fg = colors.icon.to_rgb();
                let active_bg = colors.button_active_background.to_rgb();
                let active_fg = colors.icon.to_rgb();
                (hover_bg, hover_fg, active_bg, active_fg)
            }
        };

        let icon_color = colors.icon_muted.to_rgb();

        div()
            .id(self.id())
            .flex()
            .flex_row()
            .justify_center()
            .items_center()
            // CRITICAL: Occlude prevents mouse events from reaching drag area
            // Window control buttons MUST use occlude() to block ALL mouse events
            .occlude()
            .w(px(46.0))
            .h_full()
            .text_size(px(10.0))
            .text_color(icon_color)
            .hover(move |style: StyleRefinement| style.bg(hover_bg).text_color(hover_fg))
            .active(move |style: StyleRefinement| style.bg(active_bg).text_color(active_fg))
            // CRITICAL: Register this button's control area for Windows WM_NCHITTEST
            .window_control_area(self.control_area())
            .child(self.icon())
    }
}

// =============================================================================
// Linux Window Controls
// =============================================================================

#[derive(IntoElement)]
pub struct LinuxWindowControls;

impl LinuxWindowControls {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for LinuxWindowControls {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let is_maximized = window.is_maximized();

        div()
            .id("linux-window-controls")
            .flex()
            .flex_row()
            .items_center()
            .h_full()
            .child(LinuxCaptionButton::Minimize)
            .child(if is_maximized {
                LinuxCaptionButton::Restore
            } else {
                LinuxCaptionButton::Maximize
            })
            .child(LinuxCaptionButton::Close)
    }
}

#[derive(IntoElement, Clone, Copy)]
enum LinuxCaptionButton {
    Minimize,
    Restore,
    Maximize,
    Close,
}

impl LinuxCaptionButton {
    fn id(&self) -> &'static str {
        match self {
            Self::Minimize => "linux-minimize",
            Self::Restore => "linux-restore",
            Self::Maximize => "linux-maximize",
            Self::Close => "linux-close",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Minimize => "─",
            Self::Restore => "❐",
            Self::Maximize => "□",
            Self::Close => "✕",
        }
    }
}

impl RenderOnce for LinuxCaptionButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let action = self;

        // 获取主题颜色
        let theme = cx.current_theme();
        let colors = &theme.colors;

        let (hover_bg, hover_fg) = match self {
            Self::Close => {
                // 关闭按钮使用危险颜色
                (colors.danger.to_rgb(), colors.danger_foreground.to_rgb())
            }
            _ => {
                // 最小化/最大化按钮使用主题颜色
                (
                    colors.button_hover_background.to_rgb(),
                    colors.icon.to_rgb(),
                )
            }
        };

        let icon_color = colors.icon_muted.to_rgb();

        div()
            .id(self.id())
            .flex()
            .items_center()
            .justify_center()
            .w(px(36.0))
            .h_full()
            .text_sm()
            .text_color(icon_color)
            // Window control buttons MUST use occlude() to block ALL mouse events
            .occlude()
            .hover(move |style: StyleRefinement| style.bg(hover_bg).text_color(hover_fg))
            .active(move |style: StyleRefinement| style.bg(hover_bg).opacity(0.8))
            .on_mouse_down(
                MouseButton::Left,
                |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                    window.prevent_default();
                    cx.stop_propagation();
                },
            )
            .on_click(move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                cx.stop_propagation();
                match action {
                    LinuxCaptionButton::Minimize => window.minimize_window(),
                    LinuxCaptionButton::Restore | LinuxCaptionButton::Maximize => {
                        window.zoom_window()
                    }
                    LinuxCaptionButton::Close => window.remove_window(),
                }
            })
            .child(self.icon())
    }
}

// Unit tests for public API are in tests/title_bar_tests.rs
// Private types (WindowsCaptionButton, LinuxCaptionButton) are internal implementation
// details and are implicitly tested through the public API
