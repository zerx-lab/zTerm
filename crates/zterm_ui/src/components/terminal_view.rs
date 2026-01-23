//! Terminal view component

use crate::elements::{ScrollbarElement, ScrollbarState, Selection, TerminalElement};
use crate::theme::TerminalTheme;
use zterm_terminal::{Terminal, TerminalEvent};
use gpui::*;
use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

/// IME (Input Method Editor) state for handling Chinese/Japanese/Korean input
#[derive(Clone)]
pub struct ImeState {
    /// The text currently being composed (pre-edit text)
    pub marked_text: String,
}

/// Terminal font family (must match main_window.rs)
const TERMINAL_FONT_FAMILY: &str = "Consolas";
/// Terminal font size (must match main_window.rs)
const TERMINAL_FONT_SIZE: f32 = 14.0;

/// Position in terminal grid (column, row)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub col: usize,
    pub row: usize,
}

/// Shared bounds for coordinate conversion between TerminalElement and TerminalView
#[derive(Clone)]
pub struct SharedBounds {
    /// The actual bounds of TerminalElement in window coordinates
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// Cell width measured during paint
    pub cell_width: Rc<Cell<Option<Pixels>>>,
    /// Line height measured during paint
    pub line_height: Rc<Cell<Option<Pixels>>>,
}

impl Default for SharedBounds {
    fn default() -> Self {
        Self {
            bounds: Rc::new(Cell::new(None)),
            cell_width: Rc::new(Cell::new(None)),
            line_height: Rc::new(Cell::new(None)),
        }
    }
}

/// The main terminal view component
pub struct TerminalView {
    /// The terminal entity
    terminal: Entity<Terminal>,

    /// Focus handle for keyboard input
    focus_handle: FocusHandle,

    /// Current scroll offset (in lines from bottom, 0 = at bottom)
    scroll_offset: usize,

    /// Terminal theme
    theme: TerminalTheme,

    /// Selection start position (if selecting)
    selection_start: Option<GridPosition>,

    /// Selection end position (if selecting)
    selection_end: Option<GridPosition>,

    /// Whether we are currently dragging to select
    is_selecting: bool,

    /// Cached cell width (measured from text system, used for fallback)
    cell_width: Option<Pixels>,

    /// Shared bounds with TerminalElement for accurate mouse position calculation
    shared_bounds: SharedBounds,

    /// IME state for input method composition
    pub(crate) ime_state: Option<ImeState>,

    /// Scrollbar state entity
    scrollbar_state: Entity<ScrollbarState>,
}

impl TerminalView {
    /// Create a new terminal view
    pub fn new(terminal: Entity<Terminal>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let theme = TerminalTheme::default();

        // Subscribe to terminal events
        cx.subscribe(&terminal, Self::on_terminal_event).detach();

        // Create scrollbar state entity
        let scrollbar_state = cx.new(|_| ScrollbarState::new());

        Self {
            terminal,
            focus_handle,
            scroll_offset: 0,
            theme,
            selection_start: None,
            selection_end: None,
            is_selecting: false,
            cell_width: None, // Will be measured on first use
            shared_bounds: SharedBounds::default(),
            ime_state: None,
            scrollbar_state,
        }
    }

    /// Measure cell width using the text system
    fn measure_cell_width(&mut self, cx: &App) -> Pixels {
        if let Some(width) = self.cell_width {
            return width;
        }

        let text_system = cx.text_system();
        let font = Font {
            family: TERMINAL_FONT_FAMILY.into(),
            ..Default::default()
        };

        let font_size = px(TERMINAL_FONT_SIZE);
        let font_id = text_system.resolve_font(&font);

        if let Ok(advance) = text_system.advance(font_id, font_size, 'm') {
            self.cell_width = Some(advance.width);
            return advance.width;
        }

        // Fallback to estimated width
        let fallback = px(TERMINAL_FONT_SIZE * 0.6);
        self.cell_width = Some(fallback);
        fallback
    }

    /// Handle terminal events
    fn on_terminal_event(
        &mut self,
        _terminal: Entity<Terminal>,
        event: &TerminalEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            TerminalEvent::TitleChanged(_) => {
                cx.notify();
            }
            TerminalEvent::Resized { .. } => {
                // Clear selection when terminal is resized to prevent selection artifacts
                self.selection_start = None;
                self.selection_end = None;
                self.is_selecting = false;
                // Force full re-render on resize
                cx.notify();
            }
            TerminalEvent::ProcessExited { .. } => {
                cx.notify();
            }
            _ => {}
        }
    }

    /// Handle key down events
    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;

        // Convert keystroke to terminal input
        if let Some(input) = self.keystroke_to_input(keystroke) {
            // Auto-scroll to bottom when user types (only if not already at bottom)
            if self.scroll_offset != 0 {
                self.scroll_to_bottom(cx);
            }

            self.terminal.update(cx, |terminal, _| {
                terminal.write(input.as_bytes());
            });
        }
    }

    /// Convert a keystroke to terminal input bytes
    /// NOTE: Only handles special keys. Regular character input is handled by InputHandler (IME)
    fn keystroke_to_input(&self, keystroke: &Keystroke) -> Option<String> {
        // Handle Ctrl key combinations
        if keystroke.modifiers.control {
            if let Some(c) = keystroke.key.chars().next() {
                // Skip certain Ctrl combinations that should be handled by app actions
                // Ctrl+W: close tab, Ctrl+T: new tab, Ctrl+Tab: switch tab
                let key_lower = c.to_ascii_lowercase();
                if key_lower == 'w' || key_lower == 't' {
                    return None; // Let these bubble up to app-level actions
                }

                // Ctrl+A through Ctrl+Z
                if c.is_ascii_lowercase() {
                    let ctrl_char = (c as u8 - b'a' + 1) as char;
                    return Some(ctrl_char.to_string());
                }
                if c.is_ascii_uppercase() {
                    let ctrl_char = (c as u8 - b'A' + 1) as char;
                    return Some(ctrl_char.to_string());
                }
            }
        }

        // Only handle special keys here - regular characters are handled by InputHandler
        match keystroke.key.as_str() {
            "enter" => Some("\r".to_string()),
            "tab" => Some("\t".to_string()),
            "backspace" => Some("\x7f".to_string()),
            "escape" => Some("\x1b".to_string()),
            "up" => Some("\x1b[A".to_string()),
            "down" => Some("\x1b[B".to_string()),
            "right" => Some("\x1b[C".to_string()),
            "left" => Some("\x1b[D".to_string()),
            "home" => Some("\x1b[H".to_string()),
            "end" => Some("\x1b[F".to_string()),
            "pageup" => Some("\x1b[5~".to_string()),
            "pagedown" => Some("\x1b[6~".to_string()),
            "delete" => Some("\x1b[3~".to_string()),
            "insert" => Some("\x1b[2~".to_string()),
            "f1" => Some("\x1bOP".to_string()),
            "f2" => Some("\x1bOQ".to_string()),
            "f3" => Some("\x1bOR".to_string()),
            "f4" => Some("\x1bOS".to_string()),
            "f5" => Some("\x1b[15~".to_string()),
            "f6" => Some("\x1b[17~".to_string()),
            "f7" => Some("\x1b[18~".to_string()),
            "f8" => Some("\x1b[19~".to_string()),
            "f9" => Some("\x1b[20~".to_string()),
            "f10" => Some("\x1b[21~".to_string()),
            "f11" => Some("\x1b[23~".to_string()),
            "f12" => Some("\x1b[24~".to_string()),
            // Don't handle regular characters here - they go through InputHandler
            _ => None,
        }
    }

    /// Handle scroll wheel events
    fn on_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Calculate scroll delta in lines
        let line_delta: i32 = match event.delta {
            ScrollDelta::Lines(lines) => lines.y as i32,
            ScrollDelta::Pixels(pixels) => {
                let line_height = self.theme.font_size * self.theme.line_height;
                let delta_f32: f32 = pixels.y.into();
                (delta_f32 / line_height) as i32
            }
        };

        // Get max scroll from terminal content
        let max_scroll = {
            let terminal = self.terminal.read(cx);
            terminal.content().history_size
        };

        // Scrolling up (positive delta) increases offset, scrolling down decreases
        let new_offset = if line_delta > 0 {
            self.scroll_offset.saturating_add(line_delta as usize)
        } else {
            self.scroll_offset.saturating_sub((-line_delta) as usize)
        };

        self.scroll_offset = new_offset.min(max_scroll);

        // Update terminal scroll
        self.terminal.update(cx, |terminal, _| {
            terminal.scroll(line_delta);
        });

        cx.notify();
    }

    /// Scroll to bottom (newest content)
    pub fn scroll_to_bottom(&mut self, cx: &mut Context<Self>) {
        self.scroll_offset = 0;

        // Sync terminal display offset
        self.terminal.update(cx, |terminal, _| {
            terminal.set_scroll_offset(0);
        });

        cx.notify();
    }

    /// Scroll to top (oldest content in scrollback)
    pub fn scroll_to_top(&mut self, cx: &mut Context<Self>) {
        let scroll_offset = {
            let terminal = self.terminal.read(cx);
            terminal.content().history_size
        };
        self.scroll_offset = scroll_offset;

        // Sync terminal display offset
        self.terminal.update(cx, |terminal, _| {
            terminal.set_scroll_offset(scroll_offset);
        });

        cx.notify();
    }

    /// Handle mouse down for focus and selection start
    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);

        // Start selection
        if let Some(pos) = self.position_from_mouse(event.position, cx) {
            self.selection_start = Some(pos);
            self.selection_end = Some(pos);
            self.is_selecting = true;
            cx.notify();
        }
    }

    /// Handle mouse move for selection update
    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            if let Some(pos) = self.position_from_mouse(event.position, cx) {
                self.selection_end = Some(pos);
                cx.notify();
            }
        }
    }

    /// Handle mouse up to finish selection
    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_selecting = false;

        // Clear selection if start equals end (no actual selection)
        if self.selection_start == self.selection_end {
            self.selection_start = None;
            self.selection_end = None;
            cx.notify();
        }
    }

    /// Handle scrollbar scroll event
    fn on_scrollbar_scroll(&mut self, new_offset: usize, cx: &mut Context<Self>) {
        self.scroll_offset = new_offset;

        // Update terminal scroll state
        self.terminal.update(cx, |terminal, _| {
            terminal.set_scroll_offset(new_offset);
        });

        cx.notify();
    }

    /// Convert mouse position to grid position
    ///
    /// Mouse position is in window coordinates. We need to:
    /// 1. Get the actual bounds of TerminalElement (stored during paint)
    /// 2. Calculate position relative to the element's origin
    /// 3. Convert to grid coordinates using actual cell dimensions
    fn position_from_mouse(
        &self,
        position: Point<Pixels>,
        cx: &Context<Self>,
    ) -> Option<GridPosition> {
        let terminal = self.terminal.read(cx);
        let size = terminal.size();

        // Get actual bounds from TerminalElement (set during paint phase)
        let bounds = self.shared_bounds.bounds.get()?;

        // Get actual cell dimensions from TerminalElement (set during paint phase)
        let cell_width: f32 = self
            .shared_bounds
            .cell_width
            .get()
            .map(|w| w.into())
            .unwrap_or(TERMINAL_FONT_SIZE * 0.6);
        let cell_height: f32 = self
            .shared_bounds
            .line_height
            .get()
            .map(|h| h.into())
            .unwrap_or(TERMINAL_FONT_SIZE * 1.4);

        // Convert window coordinates to element-relative coordinates
        let x: f32 = position.x.into();
        let y: f32 = position.y.into();
        let origin_x: f32 = bounds.origin.x.into();
        let origin_y: f32 = bounds.origin.y.into();

        let relative_x = (x - origin_x).max(0.0);
        let relative_y = (y - origin_y).max(0.0);

        // Use floor() for more intuitive selection behavior
        // (clicking in the middle of a cell selects that cell)
        let col = (relative_x / cell_width).floor() as usize;
        let row = (relative_y / cell_height).floor() as usize;

        // Clamp to grid bounds
        let col = col.min((size.cols as usize).saturating_sub(1));
        let row = row.min((size.rows as usize).saturating_sub(1));

        Some(GridPosition { col, row })
    }

    /// Get the current selection as normalized (start, end) where start <= end
    pub fn get_selection(&self) -> Option<Selection> {
        match (self.selection_start, self.selection_end) {
            (Some(start), Some(end)) if start != end => {
                // Normalize so start is before end
                let (start, end) = if (start.row, start.col) <= (end.row, end.col) {
                    (start, end)
                } else {
                    (end, start)
                };
                Some(Selection {
                    start_col: start.col,
                    start_row: start.row,
                    end_col: end.col,
                    end_row: end.row,
                })
            }
            _ => None,
        }
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.selection_start = None;
        self.selection_end = None;
        self.is_selecting = false;
        cx.notify();
    }

    /// Get the terminal entity
    pub fn terminal(&self) -> &Entity<Terminal> {
        &self.terminal
    }

    /// Set the marked (pre-edit) text from IME composition
    pub(crate) fn set_marked_text(&mut self, text: String, cx: &mut Context<Self>) {
        if text.is_empty() {
            return self.clear_marked_text(cx);
        }
        self.ime_state = Some(ImeState { marked_text: text });
        cx.notify();
    }

    /// Get the range of marked text (in UTF-16 code units)
    pub(crate) fn marked_text_range(&self) -> Option<Range<usize>> {
        self.ime_state
            .as_ref()
            .map(|state| 0..state.marked_text.encode_utf16().count())
    }

    /// Clear the marked (pre-edit) text state
    pub(crate) fn clear_marked_text(&mut self, cx: &mut Context<Self>) {
        if self.ime_state.is_some() {
            self.ime_state = None;
            cx.notify();
        }
    }

    /// Commit (send) the given text to the PTY
    pub(crate) fn commit_text(&mut self, text: &str, cx: &mut Context<Self>) {
        if !text.is_empty() {
            // Auto-scroll to bottom when user types (only if not already at bottom)
            if self.scroll_offset != 0 {
                self.scroll_to_bottom(cx);
            }

            self.terminal.update(cx, |term, _| {
                term.write_str(text);
            });
        }
    }

    /// Get the focus handle reference
    pub fn focus_handle_ref(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Get the theme reference
    pub fn theme_ref(&self) -> &TerminalTheme {
        &self.theme
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Measure cell width on first render
        let _ = self.measure_cell_width(cx);

        let content = {
            let terminal = self.terminal.read(cx);
            terminal.content().clone()
        };
        let theme = self.theme.clone();
        let scroll_offset = self.scroll_offset;

        // Calculate scrollbar dimensions from content
        let total_lines = content.total_lines;
        let visible_lines = content.screen_lines;
        let max_scroll = content.history_size;

        // Get current selection
        let selection = self.get_selection();

        // Create scrollbar element with callback
        let terminal_view = cx.entity().clone();
        let scrollbar = ScrollbarElement::new(
            self.scrollbar_state.clone(),
            total_lines,
            visible_lines,
            scroll_offset,
            max_scroll,
        )
        .on_scroll(move |new_offset, _window, cx| {
            terminal_view.update(cx, |view, view_cx| {
                view.on_scrollbar_scroll(new_offset, view_cx);
            });
        });

        div()
            .id("terminal-view")
            .flex()
            .flex_row()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family(theme.font_family.clone())
            .text_size(px(theme.font_size))
            .track_focus(&self.focus_handle)
            .key_context("Terminal")
            .on_key_down(cx.listener(Self::on_key_down))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .child(
                // Terminal content with 5px padding
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .pt(px(5.0))
                    .pb(px(5.0))
                    .pl(px(5.0))
                    .pr(px(5.0))
                    .child(TerminalElement::new(
                        self.terminal.clone(),
                        cx.entity().clone(),
                        self.focus_handle.clone(),
                        theme.clone(),
                        selection,
                        self.shared_bounds.clone(),
                    )),
            )
            .child(scrollbar)
    }
}
