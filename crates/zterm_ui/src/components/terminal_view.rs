//! Terminal view component

use crate::components::{ContextMenuState, ContextMenuView};
use crate::elements::{ScrollbarElement, ScrollbarState, Selection, TerminalElement};
use crate::shell_integration::ContextMenuAction;
use crate::theme::TerminalTheme;
use gpui::*;
use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;
use zterm_common::AppSettings;
use zterm_terminal::{SelectionSide, SelectionType, Terminal, TerminalEvent};

// Terminal-specific actions (defined here so zterm_ui doesn't depend on z_term)
actions!(
    terminal,
    [
        Copy,
        Paste,
        Search,
        ScrollUp,
        ScrollDown,
        ScrollPageUp,
        ScrollPageDown,
        ScrollToTop,
        ScrollToBottom,
    ]
);

/// Input batching interval in milliseconds (matches Zed's approach)
/// Keyboard events within this window are batched into a single PTY write
const INPUT_BATCH_INTERVAL_MS: u64 = 4;

/// IME (Input Method Editor) state for handling Chinese/Japanese/Korean input
#[derive(Clone)]
pub struct ImeState {
    /// The text currently being composed (pre-edit text)
    pub marked_text: String,
}

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
    /// Y offset for bottom alignment (content starts at bounds.origin.y + y_offset)
    pub y_offset: Rc<Cell<Option<Pixels>>>,
}

impl Default for SharedBounds {
    fn default() -> Self {
        Self {
            bounds: Rc::new(Cell::new(None)),
            cell_width: Rc::new(Cell::new(None)),
            line_height: Rc::new(Cell::new(None)),
            y_offset: Rc::new(Cell::new(None)),
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

    /// Current selection type (Simple, Semantic, Lines)
    /// Used to determine behavior on mouse up
    current_selection_type: Option<SelectionType>,

    /// Cached cell width (measured from text system, used for fallback)
    cell_width: Option<Pixels>,

    /// Shared bounds with TerminalElement for accurate mouse position calculation
    shared_bounds: SharedBounds,

    /// IME state for input method composition
    pub(crate) ime_state: Option<ImeState>,

    /// Scrollbar state entity
    scrollbar_state: Entity<ScrollbarState>,

    /// Pending keyboard input buffer for batching (reduces PTY writes during key repeat)
    pending_input: Vec<u8>,

    /// Timer task for flushing pending input
    input_flush_task: Option<Task<()>>,

    /// Context menu state for right-click menu
    context_menu_state: ContextMenuState,

    /// Context menu Entity 和订阅
    context_menu: Option<(Entity<ContextMenuView>, Point<Pixels>, Subscription)>,
}

impl TerminalView {
    /// Create a new terminal view
    pub fn new(terminal: Entity<Terminal>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        // Load theme from configuration
        let config = AppSettings::global_config(cx);
        let theme = TerminalTheme::from_config(&config);

        // Subscribe to terminal events
        cx.subscribe(&terminal, Self::on_terminal_event).detach();

        // Subscribe to global settings changes for hot-reload
        cx.observe_global::<AppSettings>(Self::on_settings_changed)
            .detach();

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
            current_selection_type: None,
            cell_width: None, // Will be measured on first use
            shared_bounds: SharedBounds::default(),
            ime_state: None,
            scrollbar_state,
            pending_input: Vec::with_capacity(64), // Pre-allocate for typical input
            input_flush_task: None,
            context_menu_state: ContextMenuState::new(),
            context_menu: None,
        }
    }

    /// Handle settings changes (hot-reload)
    fn on_settings_changed(&mut self, cx: &mut Context<Self>) {
        let config = AppSettings::global_config(cx);

        // Only update if config actually changed
        if self.theme.matches_config(&config) {
            return;
        }

        self.theme.update_from_config(&config);

        // Clear cached cell width so it gets remeasured with new font
        self.cell_width = None;

        tracing::info!("Terminal theme updated from config");
        cx.notify();
    }

    /// Measure cell width using the text system
    fn measure_cell_width(&mut self, cx: &App) -> Pixels {
        if let Some(width) = self.cell_width {
            return width;
        }

        let text_system = cx.text_system();
        let font = Font {
            family: self.theme.font_family.clone(),
            ..Default::default()
        };

        let font_size = px(self.theme.font_size);
        let font_id = text_system.resolve_font(&font);

        if let Ok(advance) = text_system.advance(font_id, font_size, 'm') {
            self.cell_width = Some(advance.width);
            return advance.width;
        }

        // Fallback to estimated width
        let fallback = px(self.theme.font_size * 0.6);
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
                self.current_selection_type = None;
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
    /// Uses input batching to reduce PTY writes during key repeat (long press)
    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;

        // Convert keystroke to terminal input
        if let Some(input) = self.keystroke_to_input(keystroke) {
            // Auto-scroll to bottom when user types (only if not already at bottom)
            if self.scroll_offset != 0 {
                self.scroll_to_bottom(cx);
            }

            // Add to pending buffer instead of immediate send
            self.pending_input.extend(input.as_bytes());

            // Schedule flush if not already scheduled
            self.schedule_input_flush(cx);
        }
    }

    /// Schedule a flush of pending input after INPUT_BATCH_INTERVAL_MS
    /// Multiple key events within the interval are batched into one PTY write
    fn schedule_input_flush(&mut self, cx: &mut Context<Self>) {
        // If already scheduled, let the existing timer handle it
        if self.input_flush_task.is_some() {
            return;
        }

        // Schedule flush after batch interval
        self.input_flush_task = Some(cx.spawn(async move |this, cx: &mut AsyncApp| {
            // Wait for batch interval
            smol::Timer::after(Duration::from_millis(INPUT_BATCH_INTERVAL_MS)).await;

            // Flush pending input
            let _ = this.update(cx, |view, cx| {
                view.flush_pending_input(cx);
            });
        }));
    }

    /// Flush all pending input to the PTY in a single write
    fn flush_pending_input(&mut self, cx: &mut Context<Self>) {
        self.input_flush_task = None;

        if self.pending_input.is_empty() {
            return;
        }

        // Take ownership of pending input and send all at once
        let input = std::mem::take(&mut self.pending_input);

        // Prevent unbounded capacity growth: only reserve if capacity is reasonable
        // Typical keyboard input is small, so we cap at 1KB
        const MAX_INPUT_CAPACITY: usize = 1024;
        if self.pending_input.capacity() < MAX_INPUT_CAPACITY {
            self.pending_input.reserve(64);
        } else {
            // Reset to smaller capacity if it grew too large
            self.pending_input = Vec::with_capacity(64);
        }

        self.terminal.update(cx, |terminal, _| {
            terminal.write_owned(input);
        });
    }

    /// Convert a keystroke to terminal input bytes
    /// NOTE: Only handles special keys. Regular character input is handled by InputHandler (IME)
    fn keystroke_to_input(&self, keystroke: &Keystroke) -> Option<String> {
        // Handle Ctrl key combinations
        if keystroke.modifiers.control {
            if let Some(c) = keystroke.key.chars().next() {
                let key_lower = c.to_ascii_lowercase();

                // Skip Ctrl+Shift combinations - these are handled by app actions
                // Copy (ctrl+shift+c), Paste (ctrl+shift+v), Search (ctrl+shift+f), etc.
                if keystroke.modifiers.shift {
                    return None; // Let these bubble up to action handlers
                }

                // Skip certain Ctrl combinations that should be handled by app actions
                // Ctrl+W: close tab, Ctrl+T: new tab
                if key_lower == 'w' || key_lower == 't' {
                    return None; // Let these bubble up to app-level actions
                }

                // Skip number keys with Ctrl - used for tab switching (Ctrl+1-9)
                // and zoom (Ctrl+0)
                if c.is_ascii_digit() {
                    return None;
                }

                // Skip Ctrl+=, Ctrl+- for zoom
                if c == '=' || c == '-' {
                    return None;
                }

                // Ctrl+A through Ctrl+Z (except the ones we skipped above)
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

        // Handle right mouse button for context menu
        if event.button == MouseButton::Right {
            self.show_context_menu(event.position, window, cx);
            return;
        }

        // Only handle left mouse button for selection
        if event.button != MouseButton::Left {
            return;
        }

        // Context menu will be automatically hidden by on_mouse_down_out

        // Determine selection type based on click count
        let selection_type = match event.click_count {
            0 => return, // This is a release
            1 => SelectionType::Simple,
            2 => SelectionType::Semantic, // Double-click: select word
            3 => SelectionType::Lines,    // Triple-click: select line
            _ => return,                  // Ignore further clicks
        };

        // Start selection in both local state (for UI) and alacritty (for copy)
        if let Some((col, row, side)) = self.position_and_side_from_mouse(event.position, cx) {
            // Update local state for rendering
            if let Some(pos) = self.position_from_mouse(event.position, cx) {
                self.selection_start = Some(pos);
                self.selection_end = Some(pos);
            }
            self.is_selecting = true;
            self.current_selection_type = Some(selection_type);

            // Start selection in alacritty terminal
            self.terminal.update(cx, |terminal, _| {
                terminal.start_selection(col, row, side, selection_type);
            });

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
            // Update both local state (for UI) and alacritty (for copy)
            if let Some((col, row, side)) = self.position_and_side_from_mouse(event.position, cx) {
                // Update local state for rendering
                if let Some(pos) = self.position_from_mouse(event.position, cx) {
                    self.selection_end = Some(pos);
                }

                // Update selection in alacritty terminal
                self.terminal.update(cx, |terminal, _| {
                    terminal.update_selection(col, row, side);
                });

                cx.notify();
            }
        }
    }

    /// Handle mouse up to finish selection
    fn on_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.button != MouseButton::Left {
            return;
        }

        self.is_selecting = false;

        // For Simple selection: clear if start equals end (no drag happened)
        // For Semantic/Lines/Block selection: alacritty auto-expands, so don't clear
        let should_clear = match self.current_selection_type {
            Some(SelectionType::Simple) | None => self.selection_start == self.selection_end,
            Some(SelectionType::Semantic)
            | Some(SelectionType::Lines)
            | Some(SelectionType::Block) => {
                // Don't clear - alacritty has already expanded the selection
                false
            }
        };

        if should_clear {
            self.selection_start = None;
            self.selection_end = None;
            self.current_selection_type = None;
            // Also clear alacritty selection
            self.terminal.update(cx, |terminal, _| {
                terminal.clear_selection();
            });
        }
        // Always notify to ensure selection is rendered
        cx.notify();
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
            .unwrap_or(self.theme.font_size * 0.6);
        let cell_height: f32 = self
            .shared_bounds
            .line_height
            .get()
            .map(|h| h.into())
            .unwrap_or(self.theme.font_size * self.theme.line_height);

        // Get y_offset for bottom alignment
        let y_offset: f32 = self
            .shared_bounds
            .y_offset
            .get()
            .map(|o| o.into())
            .unwrap_or(0.0);

        // Convert window coordinates to element-relative coordinates
        // Account for y_offset (content starts at bounds.origin.y + y_offset)
        let x: f32 = position.x.into();
        let y: f32 = position.y.into();
        let origin_x: f32 = bounds.origin.x.into();
        let origin_y: f32 = bounds.origin.y.into();

        let relative_x = (x - origin_x).max(0.0);
        let relative_y = (y - origin_y - y_offset).max(0.0);

        // Use floor() for more intuitive selection behavior
        // (clicking in the middle of a cell selects that cell)
        let col = (relative_x / cell_width).floor() as usize;
        let row = (relative_y / cell_height).floor() as usize;

        // Clamp to grid bounds
        let col = col.min((size.cols as usize).saturating_sub(1));
        let row = row.min((size.rows as usize).saturating_sub(1));

        Some(GridPosition { col, row })
    }

    /// Convert mouse position to grid position with side (Left/Right based on cell position)
    ///
    /// This is used for selection to properly track which side of a cell the selection
    /// starts/ends on (like Zed/alacritty does).
    fn position_and_side_from_mouse(
        &self,
        position: Point<Pixels>,
        cx: &Context<Self>,
    ) -> Option<(usize, i32, SelectionSide)> {
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
            .unwrap_or(self.theme.font_size * 0.6);
        let cell_height: f32 = self
            .shared_bounds
            .line_height
            .get()
            .map(|h| h.into())
            .unwrap_or(self.theme.font_size * self.theme.line_height);

        // Get y_offset for bottom alignment
        let y_offset: f32 = self
            .shared_bounds
            .y_offset
            .get()
            .map(|o| o.into())
            .unwrap_or(0.0);

        // Convert window coordinates to element-relative coordinates
        let x: f32 = position.x.into();
        let y: f32 = position.y.into();
        let origin_x: f32 = bounds.origin.x.into();
        let origin_y: f32 = bounds.origin.y.into();

        let relative_x = (x - origin_x).max(0.0);
        let relative_y = (y - origin_y - y_offset).max(0.0);

        // Calculate column and determine side based on position within cell
        let col = (relative_x / cell_width).floor() as usize;
        let cell_x = relative_x % cell_width;
        let half_cell_width = cell_width / 2.0;
        let side = if cell_x > half_cell_width {
            SelectionSide::Right
        } else {
            SelectionSide::Left
        };

        // Calculate row (as signed integer for alacritty coordinates)
        let row = (relative_y / cell_height).floor() as i32;

        // Clamp column to grid bounds
        let col = col.min((size.cols as usize).saturating_sub(1));

        Some((col, row, side))
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
        self.current_selection_type = None;
        cx.notify();
    }

    /// Show context menu at the specified position
    fn show_context_menu(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        // 关闭旧菜单（如果存在）
        self.context_menu = None;

        // 检查是否有选择的文本
        let has_selection = self.get_selection().is_some();

        // 创建菜单
        let terminal_view = cx.entity().clone();
        let menu = cx.new(|cx| {
            ContextMenuView::new(cx)
                .item("复制", ContextMenuAction::Copy, has_selection)
                .item("粘贴", ContextMenuAction::Paste, true)
                .on_action(move |action, window, menu_cx| {
                    terminal_view.update(menu_cx, |view, view_cx| {
                        view.handle_context_menu_action(action, window, view_cx);
                    });
                })
        });

        // 设置焦点
        window.focus(&menu.focus_handle(cx), cx);

        // 订阅关闭事件
        let subscription = cx.subscribe_in(&menu, window, |this, _, _: &DismissEvent, window, cx| {
            this.context_menu = None;
            window.focus(&this.focus_handle, cx);
            cx.notify();
        });

        self.context_menu = Some((menu, position, subscription));
        cx.notify();
    }

    /// 处理上下文菜单操作
    fn handle_context_menu_action(
        &mut self,
        action: ContextMenuAction,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            ContextMenuAction::Copy => {
                if let Some(text) = self.terminal.read(cx).selection_text() {
                    if !text.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                        tracing::info!("从右键菜单复制了 {} 字符到剪贴板", text.len());
                    }
                }
            }
            ContextMenuAction::Paste => {
                if let Some(item) = cx.read_from_clipboard() {
                    if let Some(text) = item.text() {
                        tracing::info!("从右键菜单粘贴了 {} 字符", text.len());
                        self.commit_text(&text, cx);
                    }
                }
            }
            _ => {
                tracing::warn!("不支持的上下文菜单操作: {:?}", action);
            }
        }
    }

    /// Check if context menu is visible
    pub fn is_context_menu_visible(&self) -> bool {
        self.context_menu_state.is_visible()
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
    /// Uses input batching for consistency with keyboard input
    pub(crate) fn commit_text(&mut self, text: &str, cx: &mut Context<Self>) {
        if !text.is_empty() {
            // Auto-scroll to bottom when user types (only if not already at bottom)
            if self.scroll_offset != 0 {
                self.scroll_to_bottom(cx);
            }

            // Add to pending buffer and schedule flush
            self.pending_input.extend(text.as_bytes());
            self.schedule_input_flush(cx);
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

    // === Action handlers ===

    fn handle_copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        tracing::debug!("Copy action triggered");

        // Get selection text from alacritty terminal (selection is synced via mouse events)
        if let Some(text) = self.terminal.read(cx).selection_text() {
            if !text.is_empty() {
                cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                tracing::info!("Copied {} chars to clipboard", text.len());
                return;
            }
        }

        tracing::debug!("No text selected to copy");
    }

    fn handle_paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        tracing::debug!("Paste action triggered");

        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                tracing::info!("Pasting {} chars from clipboard", text.len());
                self.commit_text(&text, cx);
            }
        } else {
            tracing::debug!("No text in clipboard to paste");
        }
    }

    fn handle_search(&mut self, _: &Search, _window: &mut Window, _cx: &mut Context<Self>) {
        // TODO: Implement search UI
        tracing::info!("Search not yet implemented");
    }

    fn handle_scroll_up(&mut self, _: &ScrollUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.scroll_lines(1, cx);
    }

    fn handle_scroll_down(&mut self, _: &ScrollDown, _window: &mut Window, cx: &mut Context<Self>) {
        self.scroll_lines(-1, cx);
    }

    fn handle_scroll_page_up(
        &mut self,
        _: &ScrollPageUp,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let page_size = {
            let terminal = self.terminal.read(cx);
            terminal.size().rows as i32
        };
        self.scroll_lines(page_size, cx);
    }

    fn handle_scroll_page_down(
        &mut self,
        _: &ScrollPageDown,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let page_size = {
            let terminal = self.terminal.read(cx);
            terminal.size().rows as i32
        };
        self.scroll_lines(-page_size, cx);
    }

    fn handle_scroll_to_top(
        &mut self,
        _: &ScrollToTop,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.scroll_to_top(cx);
    }

    fn handle_scroll_to_bottom(
        &mut self,
        _: &ScrollToBottom,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.scroll_to_bottom(cx);
    }

    /// Scroll by a number of lines (positive = up, negative = down)
    fn scroll_lines(&mut self, delta: i32, cx: &mut Context<Self>) {
        let max_scroll = {
            let terminal = self.terminal.read(cx);
            terminal.content().history_size
        };

        let new_offset = if delta > 0 {
            self.scroll_offset.saturating_add(delta as usize)
        } else {
            self.scroll_offset.saturating_sub((-delta) as usize)
        };

        self.scroll_offset = new_offset.min(max_scroll);

        self.terminal.update(cx, |terminal, _| {
            terminal.scroll(delta);
        });

        cx.notify();
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

        // Get current selection from alacritty's SelectionRange
        // This ensures double-click (word) and triple-click (line) selections work correctly
        let selection = content.selection.map(|sel| {
            let display_offset = content.display_offset as i32;
            // Convert terminal coordinates to visual coordinates
            let start_row = (sel.start.line.0 + display_offset).max(0) as usize;
            let end_row = (sel.end.line.0 + display_offset).max(0) as usize;
            Selection {
                start_col: sel.start.column.0,
                start_row,
                end_col: sel.end.column.0,
                end_row,
            }
        });

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

        let mut container = div()
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
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            // Terminal action handlers
            .on_action(cx.listener(Self::handle_copy))
            .on_action(cx.listener(Self::handle_paste))
            .on_action(cx.listener(Self::handle_search))
            .on_action(cx.listener(Self::handle_scroll_up))
            .on_action(cx.listener(Self::handle_scroll_down))
            .on_action(cx.listener(Self::handle_scroll_page_up))
            .on_action(cx.listener(Self::handle_scroll_page_down))
            .on_action(cx.listener(Self::handle_scroll_to_top))
            .on_action(cx.listener(Self::handle_scroll_to_bottom))
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
            .child(scrollbar);

        // 渲染上下文菜单（如果存在）
        // 使用 deferred 来确保菜单在正确的窗口坐标系中渲染
        if let Some((menu, position, _)) = &self.context_menu {
            let menu_clone = menu.clone();
            let position = *position;

            container = container.child(
                deferred(
                    anchored()
                        .position(position)
                        .child(menu_clone)
                )
            );
        }

        container
    }
}
