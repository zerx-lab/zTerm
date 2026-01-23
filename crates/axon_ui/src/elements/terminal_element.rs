//! Terminal rendering element using low-level Element API
//!
//! This implementation follows Zed's approach for optimal performance:
//! - Uses Element trait instead of RenderOnce
//! - Batches text with same styling into single draw calls
//! - Uses direct paint calls instead of creating div elements
//! - Resizes terminal based on actual bounds during prepaint

use crate::components::{SharedBounds, TerminalView};
use crate::theme::TerminalTheme;
use axon_terminal::alacritty_terminal::term::cell::Flags;
use axon_terminal::alacritty_terminal::vte::ansi::{Color as AnsiColor, CursorShape};
use axon_terminal::{IndexedCell, Terminal, TerminalBounds, TerminalContent};
use gpui::*;
use std::ops::Range;
use unicode_width::UnicodeWidthChar;

/// Selection range in the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub start_col: usize,
    pub start_row: usize,
    pub end_col: usize,
    pub end_row: usize,
}

impl Selection {
    /// Check if a cell at (col, row) is within the selection
    pub fn contains(&self, col: usize, row: usize) -> bool {
        if row < self.start_row || row > self.end_row {
            return false;
        }
        if row == self.start_row && row == self.end_row {
            col >= self.start_col && col <= self.end_col
        } else if row == self.start_row {
            col >= self.start_col
        } else if row == self.end_row {
            col <= self.end_col
        } else {
            true
        }
    }
}

/// A batched text run with same styling
pub struct StyledRun {
    text: String,
    line: i32,
    start_col: usize,
    /// Number of characters in this run (for merge checking, like zed)
    cell_count: usize,
    /// Number of terminal columns this run spans (accounts for wide chars like CJK)
    col_span: usize,
    fg_color: Rgba,
    bg_color: Option<Rgba>,
    bold: bool,
    italic: bool,
    underline: bool,
}

/// Layout state computed during prepaint
pub struct LayoutState {
    /// Batched text runs for efficient rendering
    text_runs: Vec<StyledRun>,
    /// Cursor position and style
    cursor_line: i32,
    cursor_col: usize,
    /// Cursor shape from terminal (Hidden when TUI apps hide cursor)
    cursor_shape: CursorShape,
    /// Cell dimensions
    cell_width: Pixels,
    line_height: Pixels,
    /// Theme colors
    background: Rgba,
    cursor_color: Rgba,
    /// Cursor bounds for IME positioning
    cursor_bounds: Option<Bounds<Pixels>>,
    /// Base text style for IME rendering
    base_text_style: TextStyle,
}

/// Terminal element for rendering terminal content
pub struct TerminalElement {
    terminal: Entity<Terminal>,
    terminal_view: Entity<TerminalView>,
    focus: FocusHandle,
    theme: TerminalTheme,
    selection: Option<Selection>,
    /// Shared bounds for mouse position calculation
    shared_bounds: Option<SharedBounds>,
}

impl TerminalElement {
    /// Create with all necessary components for full IME support
    pub fn new(
        terminal: Entity<Terminal>,
        terminal_view: Entity<TerminalView>,
        focus: FocusHandle,
        theme: TerminalTheme,
        selection: Option<Selection>,
        shared_bounds: SharedBounds,
    ) -> Self {
        Self {
            terminal,
            terminal_view,
            focus,
            theme,
            selection,
            shared_bounds: Some(shared_bounds),
        }
    }

    /// Convert an alacritty color to GPUI color
    fn color_to_gpui(&self, color: &AnsiColor, is_foreground: bool) -> Rgba {
        match color {
            AnsiColor::Named(named) => {
                use axon_terminal::alacritty_terminal::vte::ansi::NamedColor;
                let idx = match named {
                    NamedColor::Black => 0,
                    NamedColor::Red => 1,
                    NamedColor::Green => 2,
                    NamedColor::Yellow => 3,
                    NamedColor::Blue => 4,
                    NamedColor::Magenta => 5,
                    NamedColor::Cyan => 6,
                    NamedColor::White => 7,
                    NamedColor::BrightBlack => 8,
                    NamedColor::BrightRed => 9,
                    NamedColor::BrightGreen => 10,
                    NamedColor::BrightYellow => 11,
                    NamedColor::BrightBlue => 12,
                    NamedColor::BrightMagenta => 13,
                    NamedColor::BrightCyan => 14,
                    NamedColor::BrightWhite => 15,
                    NamedColor::Foreground => return self.theme.foreground,
                    NamedColor::Background => return self.theme.background,
                    NamedColor::Cursor => return self.theme.cursor_color,
                    _ => {
                        return if is_foreground {
                            self.theme.foreground
                        } else {
                            self.theme.background
                        };
                    }
                };
                self.theme.ansi_colors[idx]
            }
            AnsiColor::Spec(rgb) => {
                rgba((rgb.r as u32) << 24 | (rgb.g as u32) << 16 | (rgb.b as u32) << 8 | 0xff)
            }
            AnsiColor::Indexed(idx) => {
                if *idx < 16 {
                    self.theme.ansi_colors[*idx as usize]
                } else if *idx < 232 {
                    let idx = *idx - 16;
                    let r = (idx / 36) % 6;
                    let g = (idx / 6) % 6;
                    let b = idx % 6;
                    let r = if r > 0 { r * 40 + 55 } else { 0 };
                    let g = if g > 0 { g * 40 + 55 } else { 0 };
                    let b = if b > 0 { b * 40 + 55 } else { 0 };
                    rgba((r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | 0xff)
                } else {
                    let gray = (*idx - 232) * 10 + 8;
                    rgba((gray as u32) << 24 | (gray as u32) << 16 | (gray as u32) << 8 | 0xff)
                }
            }
        }
    }

    /// Get foreground color for a cell
    fn get_fg_color(&self, cell: &axon_terminal::alacritty_terminal::term::cell::Cell) -> Rgba {
        if cell.flags.contains(Flags::INVERSE) {
            self.color_to_gpui(&cell.bg, false)
        } else {
            self.color_to_gpui(&cell.fg, true)
        }
    }

    /// Get background color for a cell
    fn get_bg_color(
        &self,
        cell: &axon_terminal::alacritty_terminal::term::cell::Cell,
    ) -> Option<Rgba> {
        let bg = if cell.flags.contains(Flags::INVERSE) {
            self.color_to_gpui(&cell.fg, true)
        } else {
            self.color_to_gpui(&cell.bg, false)
        };

        if bg != self.theme.background {
            Some(bg)
        } else {
            None
        }
    }

    /// Build batched text runs from cells
    ///
    /// `cursor_visible` indicates whether the cursor should be rendered.
    /// When false (e.g., TUI apps hide cursor), we don't apply special cursor styling to cells.
    fn build_text_runs(&self, content: &TerminalContent, cursor_visible: bool) -> Vec<StyledRun> {
        let mut runs = Vec::new();

        // Group cells by line
        let mut lines: std::collections::BTreeMap<i32, Vec<&IndexedCell>> =
            std::collections::BTreeMap::new();

        for cell in &content.cells {
            lines.entry(cell.point.line.0).or_default().push(cell);
        }

        // display_offset is used to convert terminal coordinates to visual coordinates
        // When scrolled up, cell lines can be negative (history), we add display_offset
        // to convert to visual screen coordinates (0 = top of visible area)
        let display_offset = content.display_offset as i32;

        // Cursor position in terminal coordinates (not visual)
        let cursor_term_line = content.cursor.point.line.0;
        let cursor_col = content.cursor.point.column.0;

        for (term_line, mut cells) in lines {
            // Convert terminal line to visual line (0 = top of screen)
            let visual_line = term_line + display_offset;

            // Sort cells by column
            cells.sort_by_key(|c| c.point.column.0);

            let mut current_run: Option<StyledRun> = None;

            for cell in cells {
                let col = cell.point.column.0;
                let c = cell.cell.c;

                // Skip empty cells (but keep spaces for proper layout)
                if c == '\0' {
                    continue;
                }

                // Skip wide char spacers - they are placeholders for the second column
                // of wide characters (like CJK). The actual character is in the previous cell.
                if cell.cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
                    continue;
                }

                // Calculate character width using Unicode width rules:
                // - ASCII characters: 1 column
                // - CJK characters (Chinese, Japanese, Korean): 2 columns
                // - Emoji: typically 2 columns
                // - Zero-width characters: 0 columns
                // Fallback to Flags::WIDE_CHAR if unicode-width returns None
                let char_col_span = c.width().unwrap_or_else(|| {
                    if cell.cell.flags.contains(Flags::WIDE_CHAR) {
                        2
                    } else {
                        1
                    }
                });

                // Check cursor using terminal coordinates (not visual)
                // Only mark as cursor if cursor is visible (respects DECTCEM mode)
                let is_cursor =
                    cursor_visible && term_line == cursor_term_line && col == cursor_col;
                let is_selected = self
                    .selection
                    .map(|s| s.contains(col, visual_line as usize))
                    .unwrap_or(false);

                let fg_color = if is_cursor {
                    self.theme.background
                } else if is_selected {
                    self.theme.foreground
                } else {
                    self.get_fg_color(&cell.cell)
                };

                let bg_color = if is_cursor {
                    Some(self.theme.cursor_color)
                } else if is_selected {
                    Some(self.theme.selection_background)
                } else {
                    self.get_bg_color(&cell.cell)
                };

                let bold = cell.cell.flags.contains(Flags::BOLD);
                let italic = cell.cell.flags.contains(Flags::ITALIC);
                let underline = cell.cell.flags.contains(Flags::UNDERLINE);

                // Check if we can extend the current run
                // Use cell_count (like zed) to check column position - this naturally breaks
                // batches after wide characters because their column offset is 2 but cell_count is 1
                if let Some(ref mut run) = current_run {
                    let can_extend = run.fg_color == fg_color
                        && run.bg_color == bg_color
                        && run.bold == bold
                        && run.italic == italic
                        && run.underline == underline
                        && (run.start_col + run.cell_count) == col;

                    if can_extend {
                        run.text.push(c);
                        run.cell_count += 1;
                        run.col_span += char_col_span;
                        continue;
                    } else {
                        // Finish current run and start new one
                        runs.push(current_run.take().unwrap());
                    }
                }

                // Start new run with visual line coordinate
                current_run = Some(StyledRun {
                    text: c.to_string(),
                    line: visual_line,
                    start_col: col,
                    cell_count: 1,
                    col_span: char_col_span,
                    fg_color,
                    bg_color,
                    bold,
                    italic,
                    underline,
                });
            }

            // Don't forget the last run
            if let Some(run) = current_run {
                runs.push(run);
            }
        }

        runs
    }
}

impl Element for TerminalElement {
    type RequestLayoutState = ();
    type PrepaintState = LayoutState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 1.0;
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();

        let layout_id = window.request_layout(style, None, _cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // Calculate cell dimensions
        let text_system = cx.text_system();
        let font = Font {
            family: self.theme.font_family.clone().into(),
            ..Default::default()
        };
        let font_size = px(self.theme.font_size);
        let font_id = text_system.resolve_font(&font);

        let cell_width = text_system
            .advance(font_id, font_size, 'm')
            .map(|a| a.width)
            .unwrap_or(px(self.theme.font_size * 0.6));

        let line_height = px(self.theme.font_size * self.theme.line_height);

        // Create terminal bounds and resize terminal
        let terminal_bounds = TerminalBounds::new(line_height, cell_width, bounds);

        // Update terminal size based on actual bounds
        self.terminal.update(cx, |terminal, _cx| {
            terminal.set_bounds(terminal_bounds);
        });

        // Get content after potential resize
        let content = self.terminal.read(cx).content().clone();

        let cursor_line = content.cursor.point.line.0 + content.display_offset as i32;
        let cursor_col = content.cursor.point.column.0;
        let cursor_shape = content.cursor.shape;

        // Check if cursor is visible (not hidden by TUI apps via DECTCEM escape sequence)
        let cursor_visible = !matches!(cursor_shape, CursorShape::Hidden);

        // Build text runs, passing cursor visibility to avoid styling hidden cursor cells
        let text_runs = self.build_text_runs(&content, cursor_visible);

        // Calculate cursor bounds for IME positioning (relative to element origin)
        let cursor_bounds = Some(Bounds::new(
            point(
                cell_width * cursor_col as f32,
                line_height * cursor_line as f32,
            ),
            size(cell_width, line_height),
        ));

        // Create base text style for IME rendering
        let base_text_style = TextStyle {
            font_family: self.theme.font_family.clone().into(),
            font_size: font_size.into(),
            color: self.theme.foreground.into(),
            ..Default::default()
        };

        LayoutState {
            text_runs,
            cursor_line,
            cursor_col,
            cursor_shape,
            cell_width,
            line_height,
            background: self.theme.background,
            cursor_color: self.theme.cursor_color,
            cursor_bounds,
            base_text_style,
        }
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        layout: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Update shared bounds for mouse position calculation
        // This allows TerminalView to convert window coordinates to element-relative coordinates
        if let Some(ref shared_bounds) = self.shared_bounds {
            shared_bounds.bounds.set(Some(bounds));
            shared_bounds.cell_width.set(Some(layout.cell_width));
            shared_bounds.line_height.set(Some(layout.line_height));
        }

        // Paint background
        window.paint_quad(fill(bounds, layout.background));

        let origin = bounds.origin;
        let font = Font {
            family: self.theme.font_family.clone().into(),
            ..Default::default()
        };
        let font_size = px(self.theme.font_size);

        // Get marked text (IME composition) for rendering
        let marked_text: Option<String> = {
            let ime_state = &self.terminal_view.read(cx).ime_state;
            ime_state.as_ref().map(|state| state.marked_text.clone())
        };

        // Register input handler for IME support
        let terminal_input_handler = TerminalInputHandler {
            terminal_view: self.terminal_view.clone(),
            cursor_bounds: layout.cursor_bounds.map(|b| b + origin),
        };
        window.handle_input(&self.focus, terminal_input_handler, cx);

        // Paint text runs
        for run in &layout.text_runs {
            let x = origin.x + layout.cell_width * run.start_col as f32;
            let y = origin.y + layout.line_height * run.line as f32;

            // Paint background if needed
            // Use col_span instead of chars().count() to correctly handle wide characters (CJK)
            if let Some(bg) = run.bg_color {
                let bg_bounds = Bounds::new(
                    point(x, y),
                    size(layout.cell_width * run.col_span as f32, layout.line_height),
                );
                window.paint_quad(fill(bg_bounds, bg));
            }

            // Create text style
            let mut text_font = font.clone();
            if run.bold {
                text_font.weight = FontWeight::BOLD;
            }
            if run.italic {
                text_font.style = FontStyle::Italic;
            }

            // Shape and paint text
            // Pass cell_width to shape_line to force monospace grid layout
            // This ensures all characters align to the terminal grid regardless of actual glyph width
            let text_run = gpui::TextRun {
                len: run.text.len(),
                font: text_font,
                color: run.fg_color.into(),
                background_color: None,
                underline: if run.underline {
                    Some(UnderlineStyle {
                        thickness: px(1.0),
                        color: Some(run.fg_color.into()),
                        wavy: false,
                    })
                } else {
                    None
                },
                strikethrough: None,
            };

            let shaped_line = window.text_system().shape_line(
                run.text.clone().into(),
                font_size,
                &[text_run],
                Some(layout.cell_width), // Force monospace grid layout like Zed does
            );

            let _ = shaped_line.paint(
                point(x, y),
                layout.line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            );
        }

        // Paint IME marked text (composition) with underline
        if let Some(ref text_to_mark) = marked_text {
            if !text_to_mark.is_empty() {
                if let Some(ime_bounds) = layout.cursor_bounds {
                    let ime_position = (ime_bounds + origin).origin;

                    // Create underlined text style for IME
                    let ime_text_run = gpui::TextRun {
                        len: text_to_mark.len(),
                        font: font.clone(),
                        color: layout.base_text_style.color,
                        background_color: None,
                        underline: Some(UnderlineStyle {
                            thickness: px(1.0),
                            color: Some(layout.base_text_style.color),
                            wavy: false,
                        }),
                        strikethrough: None,
                    };

                    let shaped_line = window.text_system().shape_line(
                        text_to_mark.clone().into(),
                        font_size,
                        &[ime_text_run],
                        None,
                    );

                    // Paint background to cover terminal text behind marked text
                    let ime_background_bounds =
                        Bounds::new(ime_position, size(shaped_line.width, layout.line_height));
                    window.paint_quad(fill(ime_background_bounds, layout.background));

                    // Paint the marked text
                    let _ = shaped_line.paint(
                        ime_position,
                        layout.line_height,
                        TextAlign::Left,
                        None,
                        window,
                        cx,
                    );
                }
            }
        }

        // Paint cursor - only when there's no marked text and cursor is not hidden
        // TUI programs (like vim, htop, etc.) send escape sequences to hide cursor
        // We check cursor_shape to respect these requests
        if marked_text.is_none() && !matches!(layout.cursor_shape, CursorShape::Hidden) {
            let cursor_x = origin.x + layout.cell_width * layout.cursor_col as f32;
            let cursor_y = origin.y + layout.line_height * layout.cursor_line as f32;
            let cursor_bounds = Bounds::new(
                point(cursor_x, cursor_y),
                size(layout.cell_width, layout.line_height),
            );
            window.paint_quad(fill(cursor_bounds, layout.cursor_color));
        }
    }
}

impl IntoElement for TerminalElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Input handler for terminal IME support
pub struct TerminalInputHandler {
    terminal_view: Entity<TerminalView>,
    cursor_bounds: Option<Bounds<Pixels>>,
}

impl InputHandler for TerminalInputHandler {
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<UTF16Selection> {
        // Terminal always returns empty selection for IME purposes
        Some(UTF16Selection {
            range: 0..0,
            reversed: false,
        })
    }

    fn marked_text_range(&mut self, _window: &mut Window, cx: &mut App) -> Option<Range<usize>> {
        self.terminal_view.read(cx).marked_text_range()
    }

    fn text_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<String> {
        None
    }

    fn replace_text_in_range(
        &mut self,
        _replacement_range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut App,
    ) {
        self.terminal_view.update(cx, |view, view_cx| {
            view.clear_marked_text(view_cx);
            view.commit_text(text, view_cx);
        });
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut App,
    ) {
        self.terminal_view.update(cx, |view, view_cx| {
            view.set_marked_text(new_text.to_string(), view_cx);
        });
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut App) {
        self.terminal_view.update(cx, |view, view_cx| {
            view.clear_marked_text(view_cx);
        });
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<Bounds<Pixels>> {
        // Return cursor bounds offset by the marked text range
        let mut bounds = self.cursor_bounds?;
        // Offset for the character position in the marked text
        let offset_x = bounds.size.width * range_utf16.start as f32;
        bounds.origin.x += offset_x;
        Some(bounds)
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<usize> {
        None
    }

    fn apple_press_and_hold_enabled(&mut self) -> bool {
        false
    }
}
