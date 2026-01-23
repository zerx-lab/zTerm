//! Terminal rendering element using low-level Element API
//!
//! This implementation follows Zed's approach for optimal performance:
//! - Uses Element trait instead of RenderOnce
//! - Batches text with same styling into single draw calls
//! - Uses direct paint calls instead of creating div elements
//! - Resizes terminal based on actual bounds during prepaint

use crate::theme::TerminalTheme;
use axon_terminal::alacritty_terminal::term::cell::Flags;
use axon_terminal::alacritty_terminal::vte::ansi::Color as AnsiColor;
use axon_terminal::{IndexedCell, Terminal, TerminalBounds, TerminalContent};
use gpui::*;

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
    /// Cell dimensions
    cell_width: Pixels,
    line_height: Pixels,
    /// Theme colors
    background: Rgba,
    cursor_color: Rgba,
}

/// Terminal element for rendering terminal content
pub struct TerminalElement {
    terminal: Entity<Terminal>,
    theme: TerminalTheme,
    selection: Option<Selection>,
}

impl TerminalElement {
    /// Create a new terminal element
    pub fn new(terminal: Entity<Terminal>, theme: TerminalTheme) -> Self {
        Self {
            terminal,
            theme,
            selection: None,
        }
    }

    /// Create with selection
    pub fn with_selection(
        terminal: Entity<Terminal>,
        theme: TerminalTheme,
        selection: Option<Selection>,
    ) -> Self {
        Self {
            terminal,
            theme,
            selection,
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
    fn get_bg_color(&self, cell: &axon_terminal::alacritty_terminal::term::cell::Cell) -> Option<Rgba> {
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
    fn build_text_runs(&self, content: &TerminalContent) -> Vec<StyledRun> {
        let mut runs = Vec::new();

        // Group cells by line
        let mut lines: std::collections::BTreeMap<i32, Vec<&IndexedCell>> =
            std::collections::BTreeMap::new();

        for cell in &content.cells {
            lines.entry(cell.point.line.0).or_default().push(cell);
        }

        let cursor_line = content.cursor.point.line.0 + content.display_offset as i32;
        let cursor_col = content.cursor.point.column.0;

        for (line_idx, mut cells) in lines {
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

                let is_cursor = line_idx == cursor_line && col == cursor_col;
                let is_selected = self.selection
                    .map(|s| s.contains(col, line_idx as usize))
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
                if let Some(ref mut run) = current_run {
                    let can_extend = run.fg_color == fg_color
                        && run.bg_color == bg_color
                        && run.bold == bold
                        && run.italic == italic
                        && run.underline == underline
                        && (run.start_col + run.text.chars().count()) == col;

                    if can_extend {
                        run.text.push(c);
                        continue;
                    } else {
                        // Finish current run and start new one
                        runs.push(current_run.take().unwrap());
                    }
                }

                // Start new run
                current_run = Some(StyledRun {
                    text: c.to_string(),
                    line: line_idx,
                    start_col: col,
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

        // Build text runs
        let text_runs = self.build_text_runs(&content);

        let cursor_line = content.cursor.point.line.0 + content.display_offset as i32;
        let cursor_col = content.cursor.point.column.0;

        LayoutState {
            text_runs,
            cursor_line,
            cursor_col,
            cell_width,
            line_height,
            background: self.theme.background,
            cursor_color: self.theme.cursor_color,
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
        // Paint background
        window.paint_quad(fill(bounds, layout.background));

        let origin = bounds.origin;
        let font = Font {
            family: self.theme.font_family.clone().into(),
            ..Default::default()
        };
        let font_size = px(self.theme.font_size);

        // Paint text runs
        for run in &layout.text_runs {
            let x = origin.x + layout.cell_width * run.start_col as f32;
            let y = origin.y + layout.line_height * run.line as f32;

            // Paint background if needed
            if let Some(bg) = run.bg_color {
                let bg_bounds = Bounds::new(
                    point(x, y),
                    size(
                        layout.cell_width * run.text.chars().count() as f32,
                        layout.line_height,
                    ),
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

            let shaped_line = window
                .text_system()
                .shape_line(run.text.clone().into(), font_size, &[text_run], None);

            let _ = shaped_line.paint(
                point(x, y),
                layout.line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            );
        }

        // Paint cursor (block cursor)
        let cursor_x = origin.x + layout.cell_width * layout.cursor_col as f32;
        let cursor_y = origin.y + layout.line_height * layout.cursor_line as f32;
        let cursor_bounds = Bounds::new(
            point(cursor_x, cursor_y),
            size(layout.cell_width, layout.line_height),
        );
        window.paint_quad(fill(cursor_bounds, layout.cursor_color));
    }
}

impl IntoElement for TerminalElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
