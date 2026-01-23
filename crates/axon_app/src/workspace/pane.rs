//! Pane management for split views

use axon_terminal::Terminal;
use axon_ui::TerminalView;
use gpui::*;

/// Layout direction for panes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneDirection {
    Horizontal,
    Vertical,
}

/// A pane that can contain a terminal or be split into multiple panes
pub struct Pane {
    /// The terminal view (if this is a leaf pane)
    terminal_view: Option<Entity<TerminalView>>,

    /// Child panes (if this is a split pane)
    children: Vec<Entity<Pane>>,

    /// Split direction (if this is a split pane)
    direction: Option<PaneDirection>,

    /// Focus handle
    focus_handle: FocusHandle,
}

impl Pane {
    /// Create a new pane with a terminal
    pub fn new(terminal: Entity<Terminal>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let terminal_view = cx.new(|cx| TerminalView::new(terminal, cx));

        Self {
            terminal_view: Some(terminal_view),
            children: vec![],
            direction: None,
            focus_handle,
        }
    }

    /// Create an empty pane
    pub fn empty(cx: &mut Context<Self>) -> Self {
        Self {
            terminal_view: None,
            children: vec![],
            direction: None,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Check if this is a leaf pane (has a terminal)
    pub fn is_leaf(&self) -> bool {
        self.terminal_view.is_some()
    }

    /// Get the terminal view if this is a leaf pane
    pub fn terminal_view(&self) -> Option<&Entity<TerminalView>> {
        self.terminal_view.as_ref()
    }

    /// Split this pane
    pub fn split(&mut self, direction: PaneDirection, new_terminal: Entity<Terminal>, cx: &mut Context<Self>) {
        if self.is_leaf() {
            // Take the current terminal view
            let current_view = self.terminal_view.take();

            // Create two child panes
            let current_pane = cx.new(|cx| {
                let mut pane = Pane::empty(cx);
                pane.terminal_view = current_view;
                pane
            });

            let new_pane = cx.new(|cx| Pane::new(new_terminal, cx));

            self.children = vec![current_pane, new_pane];
            self.direction = Some(direction);
        }
    }

    /// Close this pane
    pub fn close(&mut self) {
        self.terminal_view = None;
        self.children.clear();
        self.direction = None;
    }
}

impl Focusable for Pane {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Pane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(ref terminal_view) = self.terminal_view {
            // Leaf pane - render terminal
            div()
                .size_full()
                .child(terminal_view.clone())
        } else if !self.children.is_empty() {
            // Split pane - render children
            let direction = self.direction.unwrap_or(PaneDirection::Horizontal);

            let container = div()
                .size_full()
                .flex();

            let container = match direction {
                PaneDirection::Horizontal => container.flex_row(),
                PaneDirection::Vertical => container.flex_col(),
            };

            container.children(self.children.iter().map(|child| {
                div()
                    .flex_1()
                    .border_1()
                    .border_color(rgb(0x333333))
                    .child(child.clone())
            }))
        } else {
            // Empty pane
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child("Empty pane")
        }
    }
}
