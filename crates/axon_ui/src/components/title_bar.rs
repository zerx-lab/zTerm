//! Custom title bar component for window dragging
//!
//! This implementation is based on Zed's platform_title_bar pattern for reliable
//! cross-platform window dragging support.

use gpui::{prelude::*, *};

/// Height of the title bar in pixels
pub const TITLE_BAR_HEIGHT: Pixels = px(32.0);

/// Tab information for display in title bar
#[derive(Clone)]
pub struct TabInfo {
    pub id: usize,
    pub title: String,
    pub active: bool,
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
            .on_mouse_down_out(cx.listener(move |this, _ev: &MouseDownEvent, _window, _cx| {
                this.should_move = false;
            }))
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
            // Content layout - tabs area (no occlude on container)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .h_full()
                    .items_center()
                    .overflow_x_hidden()
                    .px_2()
                    .gap_1()
                    .children(tabs.into_iter().map(|tab| {
                        let is_active = tab.active;
                        TabItem::new(tab.id, tab.title, is_active)
                    }))
                    .child(NewTabButton::new()),
            )
            // Spacer
            .child(div().flex_1())
            // Window controls (right side)
            .when(!is_fullscreen, |title_bar: Stateful<Div>| {
                match platform_style {
                    PlatformStyle::Mac => title_bar,
                    PlatformStyle::Linux => {
                        title_bar.child(LinuxWindowControls::new())
                    }
                    PlatformStyle::Windows => {
                        title_bar.child(WindowsWindowControls::new(height))
                    }
                }
            })
    }
}

/// Tab item component
#[derive(IntoElement)]
struct TabItem {
    id: usize,
    title: String,
    active: bool,
}

impl TabItem {
    fn new(id: usize, title: String, active: bool) -> Self {
        Self { id, title, active }
    }
}

impl RenderOnce for TabItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let bg_color = if self.active {
            rgb(0x2d2d2d)
        } else {
            rgb(0x1e1e1e)
        };
        let text_color = if self.active {
            rgb(0xffffff)
        } else {
            rgb(0x888888)
        };

        div()
            .id(ElementId::Name(format!("tab-{}", self.id).into()))
            .flex()
            .items_center()
            .h(px(28.0))
            .px_3()
            .rounded_md()
            .bg(bg_color)
            .hover(|style| style.bg(rgb(0x3d3d3d)))
            .cursor_pointer()
            // Occlude blocks mouse events from passing through to drag area
            // This is correct - clicking on tabs should NOT trigger window drag
            .occlude()
            .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .child(
                div()
                    .text_sm()
                    .text_color(text_color)
                    .max_w(px(120.0))
                    .truncate()
                    .child(self.title),
            )
    }
}

/// New tab button component
#[derive(IntoElement)]
struct NewTabButton;

impl NewTabButton {
    fn new() -> Self {
        Self
    }
}

impl RenderOnce for NewTabButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id("new-tab-button")
            .flex()
            .items_center()
            .justify_center()
            .w(px(28.0))
            .h(px(28.0))
            .rounded_md()
            .hover(|style| style.bg(rgb(0x3d3d3d)))
            .cursor_pointer()
            // Occlude blocks mouse events from passing through
            // This is correct - clicking on new tab button should NOT trigger window drag
            .occlude()
            .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .child("+"),
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
            Self::Minimize => "\u{e921}",  // MinimizeWindow
            Self::Restore => "\u{e923}",   // RestoreWindow
            Self::Maximize => "\u{e922}",  // MaximizeWindow
            Self::Close => "\u{e8bb}",     // ChromeClose
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
            _ => (
                rgb(0x3d3d3d),
                rgb(0xcccccc),
                rgb(0x4d4d4d),
                rgb(0xcccccc),
            ),
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
            .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                window.prevent_default();
                cx.stop_propagation();
            })
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
