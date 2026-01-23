//! Custom title bar component for window dragging
//!
//! This module provides a cross-platform title bar that works on Windows, macOS, and Linux.
//! The implementation is based on gpui-component's TitleBar pattern.

use gpui::{prelude::*, *};

/// Height of the title bar in pixels
pub const TITLE_BAR_HEIGHT: Pixels = px(32.0);

/// Custom title bar component that supports window dragging on all platforms.
///
/// # Implementation Notes
///
/// Window dragging is implemented differently per platform:
/// - **Windows**: Uses `WindowControlArea::Drag` which registers the area for `WM_NCHITTEST`
///   returning `HTCAPTION`, allowing native window dragging behavior.
/// - **macOS/Linux**: Uses `start_window_move()` API triggered on mouse move after mouse down.
///
/// Window control buttons use `WindowControlArea::Min/Max/Close` on Windows to integrate
/// with the native window chrome behavior.
#[derive(IntoElement)]
pub struct TitleBar;

impl TitleBar {
    /// Create a new title bar
    pub fn new(_title: impl Into<SharedString>) -> Self {
        // Title is no longer displayed for minimal UI
        Self
    }
}

/// State for tracking window drag
struct TitleBarState {
    should_move: bool,
}

impl Render for TitleBarState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_maximized = window.is_maximized();

        // Use window state to track drag state across renders
        let state = window.use_state(cx, |_, _| TitleBarState { should_move: false });

        div()
            .flex_shrink_0()
            .child(
                div()
                    .id("title-bar")
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(TITLE_BAR_HEIGHT)
                    .w_full()
                    .bg(rgb(0x1e1e1e))
                    // Reset drag state when mouse leaves
                    .on_mouse_down_out(window.listener_for(&state, |state, _, _, _| {
                        state.should_move = false;
                    }))
                    // Start tracking potential drag on mouse down
                    .on_mouse_down(
                        MouseButton::Left,
                        window.listener_for(&state, |state, _, _, _| {
                            state.should_move = true;
                        }),
                    )
                    // Reset drag state on mouse up
                    .on_mouse_up(
                        MouseButton::Left,
                        window.listener_for(&state, |state, _, _, _| {
                            state.should_move = false;
                        }),
                    )
                    // Start window move on mouse move (for macOS/Linux)
                    .on_mouse_move(window.listener_for(&state, |state, _, window, _| {
                        if state.should_move {
                            state.should_move = false;
                            window.start_window_move();
                        }
                    }))
                    // Double-click to maximize/restore
                    .on_click(|event: &ClickEvent, window: &mut Window, _: &mut App| {
                        if event.click_count() == 2 {
                            window.zoom_window();
                        }
                    })
                    // Drag region - this is where Windows WM_NCHITTEST will detect drag area
                    .child(
                        div()
                            .id("title-bar-drag-region")
                            .flex()
                            .flex_1()
                            .h_full()
                            .items_center()
                            // Register this area as draggable for Windows WM_NCHITTEST
                            .window_control_area(WindowControlArea::Drag),
                    )
                    // Window control buttons (right side)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .flex_shrink_0()
                            .h_full()
                            .child(WindowControlButton::new(WindowAction::Minimize))
                            .child(WindowControlButton::new(if is_maximized {
                                WindowAction::Restore
                            } else {
                                WindowAction::Maximize
                            }))
                            .child(WindowControlButton::new(WindowAction::Close)),
                    ),
            )
    }
}

/// Window button action types
#[derive(Clone, Copy, PartialEq)]
enum WindowAction {
    Minimize,
    Restore,
    Maximize,
    Close,
}

impl WindowAction {
    fn id(&self) -> &'static str {
        match self {
            Self::Minimize => "window-minimize",
            Self::Restore => "window-restore",
            Self::Maximize => "window-maximize",
            Self::Close => "window-close",
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

    fn hover_bg(&self) -> u32 {
        match self {
            Self::Close => 0xe81123,
            _ => 0x3d3d3d,
        }
    }

    fn control_area(&self) -> WindowControlArea {
        match self {
            Self::Minimize => WindowControlArea::Min,
            Self::Restore | Self::Maximize => WindowControlArea::Max,
            Self::Close => WindowControlArea::Close,
        }
    }
}

/// Window control button component
#[derive(IntoElement)]
struct WindowControlButton {
    action: WindowAction,
}

impl WindowControlButton {
    fn new(action: WindowAction) -> Self {
        Self { action }
    }
}

impl RenderOnce for WindowControlButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let action = self.action;
        let hover_bg = action.hover_bg();
        let hover_fg = if action == WindowAction::Close {
            0xffffff
        } else {
            0xcccccc
        };
        let is_windows = cfg!(target_os = "windows");
        let is_linux = cfg!(target_os = "linux");

        div()
            .id(action.id())
            .flex()
            .items_center()
            .justify_center()
            .w(px(46.0))
            .h_full()
            .flex_shrink_0()
            .text_sm()
            .text_color(rgb(0xcccccc))
            .hover(move |style| style.bg(rgb(hover_bg)).text_color(rgb(hover_fg)))
            .active(move |style| style.bg(rgb(hover_bg)).opacity(0.8))
            // Register this button's control area for Windows WM_NCHITTEST
            .when(is_windows, |this: Stateful<Div>| {
                this.window_control_area(action.control_area())
            })
            // For Linux, handle clicks manually
            .when(is_linux, |this: Stateful<Div>| {
                this.on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .on_click(move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                    cx.stop_propagation();
                    match action {
                        WindowAction::Minimize => window.minimize_window(),
                        WindowAction::Restore | WindowAction::Maximize => window.zoom_window(),
                        WindowAction::Close => window.remove_window(),
                    }
                })
            })
            .child(action.icon())
    }
}

// Note: GPUI component tests require #[gpui::test] and TestAppContext.
// Basic unit tests are in a separate test file to avoid macro expansion issues.
