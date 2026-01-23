//! Custom title bar component for window dragging
//!
//! This implementation is based on Zed's platform_title_bar pattern for reliable
//! cross-platform window dragging support.

use gpui::{prelude::*, *};
use gpui_component::h_flex;
use std::path::Path;

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
    pub shell_name: String,
    /// Working directory path
    pub working_directory: String,
}

impl TabInfo {
    /// Get a display-friendly directory name
    /// Shows last directory component, or ~ for home directory
    pub fn display_directory(&self) -> String {
        let path = &self.working_directory;

        // Check if it's home directory
        if let Some(home) = dirs::home_dir() {
            if path == home.to_string_lossy().as_ref() {
                return "~".to_string();
            }
            // Check if it starts with home directory
            if let Ok(stripped) = Path::new(path).strip_prefix(&home) {
                if stripped.as_os_str().is_empty() {
                    return "~".to_string();
                }
                // Return ~/last_component format for subdirectories
                if let Some(last) = stripped.file_name() {
                    return format!("~/{}", last.to_string_lossy());
                }
            }
        }

        // Return last path component
        Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone())
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
}

impl EventEmitter<TitleBarEvent> for TitleBar {}

impl TitleBar {
    /// Create a new title bar
    pub fn new() -> Self {
        Self {
            tabs: vec![],
            should_move: false,
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
    fn on_select_tab(&mut self, tab_id: usize, cx: &mut Context<Self>) {
        cx.emit(TitleBarEvent::SelectTab(tab_id));
    }

    /// Handle tab close
    fn on_close_tab(&mut self, tab_id: usize, cx: &mut Context<Self>) {
        cx.emit(TitleBarEvent::CloseTab(tab_id));
    }

    /// Handle new tab
    fn on_new_tab(&mut self, cx: &mut Context<Self>) {
        cx.emit(TitleBarEvent::NewTab);
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
            .bg(rgb(0x1e1e1e))
            .border_b_1()
            .border_color(rgb(0x333333))
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
            // Content layout - tabs area with proper overflow handling
            // Outer container: overflow_x_hidden to clip overflowing content
            // Inner container: overflow_x_scroll for scrollable tabs
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .overflow_x_hidden()
                    .min_w_0() // Allow shrinking below content size
                    .child(
                        h_flex()
                            .id("tabs-scroll-container")
                            .h_full()
                            .items_center()
                            .overflow_x_scroll()
                            .px_2()
                            .gap_1()
                            .children(tabs.into_iter().map(|tab| {
                                let tab_id = tab.id;
                                let is_active = tab.active;
                                let display_dir = tab.display_directory();

                                // Colors inspired by modern terminals (Warp, iTerm2, Windows Terminal)
                                let bg_color = if is_active {
                                    rgb(0x2d2d2d)
                                } else {
                                    rgb(0x252525)
                                };
                                let text_color = if is_active {
                                    rgb(0xe8e8e8)
                                } else {
                                    rgb(0x808080)
                                };
                                let hover_bg = if is_active {
                                    rgb(0x363636)
                                } else {
                                    rgb(0x2d2d2d)
                                };

                                div()
                                    .id(ElementId::Name(format!("tab-{}", tab_id).into()))
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
                                        el.border_b_2().border_color(rgb(0x4a9eff))
                                    })
                                    .when(!is_active, |el| {
                                        el.border_b_1().border_color(rgb(0x3a3a3a))
                                    })
                                    .hover(|style| style.bg(hover_bg))
                                    .cursor_pointer()
                                    .occlude()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                            window.prevent_default();
                                            cx.stop_propagation();
                                        },
                                    )
                                    .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                                        this.on_select_tab(tab_id, cx);
                                    }))
                                    // Directory path - takes most space
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .text_color(text_color)
                                            .truncate()
                                            .overflow_hidden()
                                            .child(display_dir),
                                    )
                                    // Close button - compact and subtle
                                    .child(
                                        div()
                                            .id(ElementId::Name(
                                                format!("close-tab-{}", tab_id).into(),
                                            ))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(18.0))
                                            .h(px(18.0))
                                            .ml(px(6.0))
                                            .rounded(px(4.0))
                                            .text_xs()
                                            .text_color(rgb(0x606060))
                                            .hover(|style| {
                                                style.bg(rgb(0x4a4a4a)).text_color(rgb(0xd0d0d0))
                                            })
                                            .active(|style| style.bg(rgb(0x5a5a5a)))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                |_: &MouseDownEvent,
                                                 window: &mut Window,
                                                 cx: &mut App| {
                                                    window.prevent_default();
                                                    cx.stop_propagation();
                                                },
                                            )
                                            .on_click(cx.listener(
                                                move |this, _: &ClickEvent, _window, cx| {
                                                    cx.stop_propagation();
                                                    this.on_close_tab(tab_id, cx);
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
                                    .text_color(rgb(0x606060))
                                    .hover(|style| style.bg(rgb(0x2d2d2d)).text_color(rgb(0xa0a0a0)))
                                    .active(|style| style.bg(rgb(0x3d3d3d)))
                                    .cursor_pointer()
                                    .occlude()
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
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (hover_bg, hover_fg, active_bg, active_fg) = match self {
            Self::Close => (
                rgb(0xe81123),
                rgb(0xffffff),
                rgba(0xe8112399),
                rgba(0xffffffcc),
            ),
            _ => (rgb(0x3d3d3d), rgb(0xcccccc), rgb(0x4d4d4d), rgb(0xcccccc)),
        };

        div()
            .id(self.id())
            .flex()
            .flex_row()
            .justify_center()
            .items_center()
            // CRITICAL: Occlude prevents mouse events from reaching drag area
            .occlude()
            .w(px(46.0))
            .h_full()
            .text_size(px(10.0))
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
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let action = self;
        let (hover_bg, hover_fg) = match self {
            Self::Close => (rgb(0xe81123), rgb(0xffffff)),
            _ => (rgb(0x3d3d3d), rgb(0xcccccc)),
        };

        div()
            .id(self.id())
            .flex()
            .items_center()
            .justify_center()
            .w(px(36.0))
            .h_full()
            .text_sm()
            .text_color(rgb(0xcccccc))
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

// Unit tests are in tests/title_bar_tests.rs
