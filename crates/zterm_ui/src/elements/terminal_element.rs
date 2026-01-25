//! Terminal rendering element using low-level Element API
//!
//! This implementation follows Zed's approach for optimal performance:
//! - Uses Element trait with PrepaintState for efficient layout caching
//! - Batches text with same styling into single draw calls
//! - Merges adjacent background regions to minimize paint_quad calls
//! - Implements viewport clipping to only render visible cells
//! - Supports full ANSI attributes (DIM, UNDERCURL, STRIKEOUT)
//! - Multiple cursor shapes (Block, Beam, Underline, Hollow)

use crate::components::{SharedBounds, TerminalView};
use crate::theme::TerminalTheme;
use zterm_terminal::alacritty_terminal::index::Point as AlacPoint;
use zterm_terminal::alacritty_terminal::selection::SelectionRange;
use zterm_terminal::alacritty_terminal::term::cell::Flags;
use zterm_terminal::alacritty_terminal::vte::ansi::{Color as AnsiColor, CursorShape};
use zterm_terminal::{IndexedCell, Terminal, TerminalBounds};
use gpui::*;
use std::ops::Range;

/// A batched text run with same styling (following Zed's approach)
#[derive(Debug, Clone)]
pub struct BatchedTextRun {
    text: String,
    line: i32,
    start_col: usize,
    cell_count: usize,
    style: TextRun,
    font_size: Pixels,
}

impl BatchedTextRun {
    fn new(line: i32, start_col: usize, text: String, style: TextRun, font_size: Pixels) -> Self {
        Self {
            text,
            line,
            start_col,
            cell_count: 1,
            style,
            font_size,
        }
    }

    /// Check if another character can be appended to this run
    fn can_append(&self, other_style: &TextRun, next_col: usize) -> bool {
        self.style.font == other_style.font
            && self.style.color == other_style.color
            && self.style.background_color == other_style.background_color
            && self.style.underline == other_style.underline
            && self.style.strikethrough == other_style.strikethrough
            && (self.start_col + self.cell_count) == next_col
    }

    fn append(&mut self, c: char) {
        self.text.push(c);
        self.cell_count += 1;
    }

    /// Paint this text run
    fn paint(
        &self,
        origin: Point<Pixels>,
        cell_width: Pixels,
        line_height: Pixels,
        window: &mut Window,
        cx: &mut App,
    ) {
        let x = origin.x + cell_width * self.start_col as f32;
        let y = origin.y + line_height * self.line as f32;

        let shaped_line = window.text_system().shape_line(
            self.text.clone().into(),
            self.font_size,
            &[self.style.clone()],
            Some(cell_width), // Force monospace grid layout
        );

        let _ = shaped_line.paint(
            point(x, y),
            line_height,
            TextAlign::Left,
            None,
            window,
            cx,
        );
    }
}

/// Background rectangle for a region of cells
#[derive(Debug, Clone)]
pub struct LayoutRect {
    line: i32,
    start_col: usize,
    num_cells: usize,
    color: Rgba,
}

impl LayoutRect {
    fn new(line: i32, start_col: usize, color: Rgba) -> Self {
        Self {
            line,
            start_col,
            num_cells: 1,
            color,
        }
    }

    /// Check if this rect can be extended with another cell
    fn can_extend(&self, next_col: usize, color: Rgba) -> bool {
        self.color == color && (self.start_col + self.num_cells) == next_col
    }

    fn extend(&mut self) {
        self.num_cells += 1;
    }

    /// Check if this rect can merge with another (vertically adjacent)
    fn can_merge_with(&self, other: &Self) -> bool {
        self.color == other.color
            && self.start_col == other.start_col
            && self.num_cells == other.num_cells
            && (self.line + 1 == other.line || other.line + 1 == self.line)
    }

    /// Paint this background rect
    fn paint(
        &self,
        origin: Point<Pixels>,
        cell_width: Pixels,
        line_height: Pixels,
        window: &mut Window,
    ) {
        let x = origin.x + cell_width * self.start_col as f32;
        let y = origin.y + line_height * self.line as f32;
        let width = cell_width * self.num_cells as f32;

        window.paint_quad(fill(
            Bounds::new(point(x, y), size(width, line_height)),
            self.color,
        ));
    }
}

/// Layout state computed during prepaint
pub struct LayoutState {
    /// Batched text runs for efficient rendering
    text_runs: Vec<BatchedTextRun>,
    /// Background rectangles (merged for efficiency)
    background_rects: Vec<LayoutRect>,
    /// Cursor position and style
    cursor_line: i32,
    cursor_col: usize,
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
    /// Y offset for bottom alignment
    y_offset: Pixels,
    /// Whether cursor should be visible
    cursor_visible: bool,
}

/// Terminal element for rendering terminal content
pub struct TerminalElement {
    terminal: Entity<Terminal>,
    terminal_view: Entity<TerminalView>,
    focus: FocusHandle,
    theme: TerminalTheme,
    selection: Option<SelectionRange>,
    shared_bounds: Option<SharedBounds>,
}

impl TerminalElement {
    pub fn new(
        terminal: Entity<Terminal>,
        terminal_view: Entity<TerminalView>,
        focus: FocusHandle,
        theme: TerminalTheme,
        selection: Option<SelectionRange>,
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

    /// Check if a character is decorative (box drawing, block elements, Powerline)
    /// These characters should not have contrast adjustment applied
    fn is_decorative_character(ch: char) -> bool {
        matches!(ch as u32,
            // Unicode box drawing
            0x2500..=0x257F
            // Block elements
            | 0x2580..=0x259F
            // Geometric shapes
            | 0x25A0..=0x25FF
            // Powerline separators (Private Use Area)
            | 0xE0B0..=0xE0B7
            | 0xE0B8..=0xE0BF
            | 0xE0C0..=0xE0CA
            | 0xE0CC..=0xE0D1
            | 0xE0D2..=0xE0D7
        )
    }

    /// Calculate relative luminance for contrast calculation
    fn relative_luminance(color: Rgba) -> f32 {
        let r = if color.r <= 0.03928 {
            color.r / 12.92
        } else {
            ((color.r + 0.055) / 1.055).powf(2.4)
        };
        let g = if color.g <= 0.03928 {
            color.g / 12.92
        } else {
            ((color.g + 0.055) / 1.055).powf(2.4)
        };
        let b = if color.b <= 0.03928 {
            color.b / 12.92
        } else {
            ((color.b + 0.055) / 1.055).powf(2.4)
        };
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Calculate contrast ratio between two colors
    fn contrast_ratio(fg: Rgba, bg: Rgba) -> f32 {
        let l1 = Self::relative_luminance(fg);
        let l2 = Self::relative_luminance(bg);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Ensure minimum contrast between foreground and background
    fn ensure_minimum_contrast(mut fg: Rgba, bg: Rgba, minimum: f32) -> Rgba {
        let current_contrast = Self::contrast_ratio(fg, bg);
        if current_contrast >= minimum {
            return fg;
        }

        // Adjust lightness to meet minimum contrast
        let bg_luminance = Self::relative_luminance(bg);
        let target_luminance = if bg_luminance > 0.5 {
            // Dark text on light background
            bg_luminance * (1.0 / minimum) - 0.05
        } else {
            // Light text on dark background
            (bg_luminance + 0.05) * minimum - 0.05
        };

        // Simple approach: adjust all RGB channels equally
        let current_luminance = Self::relative_luminance(fg);
        if current_luminance > 0.0 {
            let scale = (target_luminance / current_luminance).min(1.0).max(0.0);
            fg.r = (fg.r * scale).min(1.0).max(0.0);
            fg.g = (fg.g * scale).min(1.0).max(0.0);
            fg.b = (fg.b * scale).min(1.0).max(0.0);
        }

        fg
    }

    /// Convert an alacritty color to GPUI color
    fn color_to_gpui(&self, color: &AnsiColor, is_foreground: bool) -> Rgba {
        match color {
            AnsiColor::Named(named) => {
                use zterm_terminal::alacritty_terminal::vte::ansi::NamedColor;
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
                    // 216-color RGB cube (6x6x6)
                    let idx = *idx - 16;
                    let r = (idx / 36) % 6;
                    let g = (idx / 6) % 6;
                    let b = idx % 6;
                    let r = if r > 0 { r * 40 + 55 } else { 0 };
                    let g = if g > 0 { g * 40 + 55 } else { 0 };
                    let b = if b > 0 { b * 40 + 55 } else { 0 };
                    rgba((r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | 0xff)
                } else {
                    // 24-step grayscale (232-255)
                    let gray = (*idx - 232) * 10 + 8;
                    rgba((gray as u32) << 24 | (gray as u32) << 16 | (gray as u32) << 8 | 0xff)
                }
            }
        }
    }

    /// Create text style for a cell with full ANSI attribute support
    fn cell_style(
        &self,
        cell: &zterm_terminal::alacritty_terminal::term::cell::Cell,
        c: char,
        fg_color: AnsiColor,
        bg_color: AnsiColor,
        font: &Font,
        _font_size: Pixels,
    ) -> TextRun {
        let flags = cell.flags;

        // Convert colors
        let mut fg = self.color_to_gpui(&fg_color, true);
        let bg = self.color_to_gpui(&bg_color, false);

        // Apply contrast adjustment (except for decorative characters)
        if !Self::is_decorative_character(c) {
            fg = Self::ensure_minimum_contrast(fg, bg, 4.5);
        }

        // Apply DIM attribute (reduce opacity to 70%)
        if flags.contains(Flags::DIM) {
            fg.a *= 0.7;
        }

        // Underline styles (including wavy underline for UNDERCURL)
        let underline = if flags.contains(Flags::UNDERLINE)
            || flags.contains(Flags::DOUBLE_UNDERLINE)
            || flags.contains(Flags::UNDERCURL)
            || flags.contains(Flags::DOTTED_UNDERLINE)
            || flags.contains(Flags::DASHED_UNDERLINE)
        {
            Some(UnderlineStyle {
                color: Some(fg.into()),
                thickness: px(1.0),
                wavy: flags.contains(Flags::UNDERCURL),
            })
        } else {
            None
        };

        // Strikethrough
        let strikethrough = if flags.contains(Flags::STRIKEOUT) {
            Some(StrikethroughStyle {
                color: Some(fg.into()),
                thickness: px(1.0),
            })
        } else {
            None
        };

        // Font weight and style
        let weight = if flags.contains(Flags::BOLD) {
            FontWeight::BOLD
        } else {
            font.weight
        };

        let style = if flags.contains(Flags::ITALIC) {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };

        TextRun {
            len: c.len_utf8(),
            font: Font {
                family: font.family.clone(),
                features: font.features.clone(),
                weight,
                style,
                ..font.clone()
            },
            color: fg.into(),
            background_color: None, // Background handled separately
            underline,
            strikethrough,
        }
    }

    /// Layout grid cells into batched text runs and background rects
    /// This follows Zed's approach for optimal performance
    fn layout_grid(
        &self,
        cells: impl Iterator<Item = IndexedCell>,
        display_offset: usize,
        cursor_visible: bool,
        cursor_line: i32,
        cursor_col: usize,
        font: &Font,
        font_size: Pixels,
    ) -> (Vec<LayoutRect>, Vec<BatchedTextRun>) {
        let mut bg_rects: Vec<LayoutRect> = Vec::new();
        let mut text_runs: Vec<BatchedTextRun> = Vec::new();
        let mut current_bg_rect: Option<LayoutRect> = None;

        let mut min_terminal_line = i32::MAX;
        let mut max_terminal_line = i32::MIN;
        let mut cell_count = 0;

        for cell in cells {
            let terminal_line = cell.point.line.0;
            min_terminal_line = min_terminal_line.min(terminal_line);
            max_terminal_line = max_terminal_line.max(terminal_line);
            cell_count += 1;
            let c = cell.cell.c;

            // Skip null characters
            if c == '\0' {
                // Flush current background rect
                if let Some(rect) = current_bg_rect.take() {
                    bg_rects.push(rect);
                }
                continue;
            }

            // Skip wide char spacers (but don't break the background)
            if cell.cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
                continue;
            }

            let col = cell.point.column.0;
            let terminal_line = cell.point.line.0;
            // Convert terminal coordinate to visual coordinate
            // terminal_line can be negative when scrolled (e.g., -10 means 10 lines up in history)
            // display_offset tells us how far we've scrolled up
            let visual_line = terminal_line + display_offset as i32;

            // Check if this is the cursor position (using terminal coordinates)
            let is_cursor = cursor_visible && terminal_line == cursor_line - display_offset as i32 && col == cursor_col;

            // Check if selected (using terminal coordinates, like Zed)
            // terminal_line is in alacritty's coordinate system (can be negative when scrolled)
            let is_selected = self.selection
                .map(|sel| {
                    let point = AlacPoint::new(zterm_terminal::alacritty_terminal::index::Line(terminal_line),
                                              zterm_terminal::alacritty_terminal::index::Column(col));
                    // Check if point is within selection range
                    point >= sel.start && point <= sel.end
                })
                .unwrap_or(false);

            // Determine colors (handle INVERSE, cursor, and selection)
            let (mut fg_color, mut bg_color) = if cell.cell.flags.contains(Flags::INVERSE) {
                (cell.cell.bg, cell.cell.fg)
            } else {
                (cell.cell.fg, cell.cell.bg)
            };

            // Override colors for cursor and selection
            if is_cursor {
                fg_color = AnsiColor::Named(zterm_terminal::alacritty_terminal::vte::ansi::NamedColor::Background);
                bg_color = AnsiColor::Named(zterm_terminal::alacritty_terminal::vte::ansi::NamedColor::Cursor);
            } else if is_selected {
                fg_color = AnsiColor::Named(zterm_terminal::alacritty_terminal::vte::ansi::NamedColor::Foreground);
                bg_color = AnsiColor::Spec(zterm_terminal::alacritty_terminal::vte::ansi::Rgb {
                    r: (self.theme.selection_background.r * 255.0) as u8,
                    g: (self.theme.selection_background.g * 255.0) as u8,
                    b: (self.theme.selection_background.b * 255.0) as u8,
                });
            }

            // Create text style
            let text_style = self.cell_style(&cell.cell, c, fg_color, bg_color, font, font_size);

            // Handle background rectangles (use visual coordinates for rendering)
            let bg_rgba = self.color_to_gpui(&bg_color, false);
            if bg_rgba != self.theme.background {
                if let Some(ref mut rect) = current_bg_rect {
                    if rect.line == visual_line && rect.can_extend(col, bg_rgba) {
                        rect.extend();
                    } else {
                        bg_rects.push(current_bg_rect.take().unwrap());
                        current_bg_rect = Some(LayoutRect::new(visual_line, col, bg_rgba));
                    }
                } else {
                    current_bg_rect = Some(LayoutRect::new(visual_line, col, bg_rgba));
                }
            } else {
                // Background matches default, flush current rect
                if let Some(rect) = current_bg_rect.take() {
                    bg_rects.push(rect);
                }
            }

            // Handle text runs (use visual coordinates for rendering)
            if let Some(last_run) = text_runs.last_mut() {
                if last_run.line == visual_line && last_run.can_append(&text_style, col) {
                    last_run.append(c);
                    continue;
                }
            }

            // Start new text run
            text_runs.push(BatchedTextRun::new(
                visual_line,
                col,
                c.to_string(),
                text_style,
                font_size,
            ));
        }

        // Flush remaining background rect
        if let Some(rect) = current_bg_rect {
            bg_rects.push(rect);
        }

        if cell_count > 0 {
            eprintln!("[layout_grid] Processed {} cells, terminal_line range: {} to {}, display_offset: {}, visual_line range: {} to {}",
                cell_count, min_terminal_line, max_terminal_line, display_offset,
                min_terminal_line + display_offset as i32, max_terminal_line + display_offset as i32);

            // Log text runs per line to debug missing lines
            let mut lines_with_text: std::collections::BTreeSet<i32> = std::collections::BTreeSet::new();
            for run in &text_runs {
                lines_with_text.insert(run.line);
            }
            let line_vec: Vec<i32> = lines_with_text.iter().copied().collect();
            if !line_vec.is_empty() {
                eprintln!("[layout_grid] Created text runs for {} unique lines, first: {}, last: {}",
                    line_vec.len(), line_vec.first().unwrap(), line_vec.last().unwrap());

                // Check for gaps in line numbers
                for i in 1..line_vec.len() {
                    let gap = line_vec[i] - line_vec[i-1];
                    if gap > 1 {
                        eprintln!("[layout_grid] WARNING: Gap detected! Line {} to {} (gap of {})",
                            line_vec[i-1], line_vec[i], gap);
                    }
                }
            }
        }

        // Merge background rectangles for efficiency
        bg_rects = Self::merge_background_regions(bg_rects);

        (bg_rects, text_runs)
    }

    /// Merge adjacent background rectangles to reduce paint calls
    /// This implements Zed's background merging algorithm
    fn merge_background_regions(mut regions: Vec<LayoutRect>) -> Vec<LayoutRect> {
        if regions.is_empty() {
            return regions;
        }

        let mut changed = true;
        while changed {
            changed = false;
            let mut i = 0;

            while i < regions.len() {
                let mut j = i + 1;
                while j < regions.len() {
                    if regions[i].can_merge_with(&regions[j]) {
                        // Merge regions[j] into regions[i]
                        let other = regions.remove(j);
                        // Extend to cover both regions
                        if other.line < regions[i].line {
                            regions[i].line = other.line;
                        }
                        changed = true;
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }

        regions
    }

    /// Paint cursor with support for multiple shapes
    fn paint_cursor(
        cursor_shape: CursorShape,
        cursor_bounds: Bounds<Pixels>,
        cursor_color: Rgba,
        focused: bool,
        window: &mut Window,
    ) {
        match cursor_shape {
            CursorShape::Block => {
                if focused {
                    // Solid block cursor
                    window.paint_quad(fill(cursor_bounds, cursor_color));
                } else {
                    // Hollow block when unfocused
                    Self::paint_hollow_cursor(cursor_bounds, cursor_color, window);
                }
            }
            CursorShape::Beam => {
                // Vertical bar cursor (left edge)
                let beam_bounds = Bounds::new(
                    cursor_bounds.origin,
                    size(px(2.0), cursor_bounds.size.height),
                );
                window.paint_quad(fill(beam_bounds, cursor_color));
            }
            CursorShape::Underline => {
                // Horizontal line cursor (bottom edge)
                let underline_bounds = Bounds::new(
                    point(
                        cursor_bounds.origin.x,
                        cursor_bounds.origin.y + cursor_bounds.size.height - px(2.0),
                    ),
                    size(cursor_bounds.size.width, px(2.0)),
                );
                window.paint_quad(fill(underline_bounds, cursor_color));
            }
            CursorShape::HollowBlock => {
                // Hollow block cursor
                Self::paint_hollow_cursor(cursor_bounds, cursor_color, window);
            }
            CursorShape::Hidden => {
                // Don't paint anything
            }
        }
    }

    /// Paint a hollow (outline) cursor
    fn paint_hollow_cursor(bounds: Bounds<Pixels>, color: Rgba, window: &mut Window) {
        let thickness = px(1.0);

        // Top edge
        window.paint_quad(fill(
            Bounds::new(bounds.origin, size(bounds.size.width, thickness)),
            color,
        ));
        // Bottom edge
        window.paint_quad(fill(
            Bounds::new(
                point(bounds.origin.x, bounds.bottom() - thickness),
                size(bounds.size.width, thickness),
            ),
            color,
        ));
        // Left edge
        window.paint_quad(fill(
            Bounds::new(bounds.origin, size(thickness, bounds.size.height)),
            color,
        ));
        // Right edge
        window.paint_quad(fill(
            Bounds::new(
                point(bounds.right() - thickness, bounds.origin.y),
                size(thickness, bounds.size.height),
            ),
            color,
        ));
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
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 1.0;
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();

        let layout_id = window.request_layout(style, None, cx);
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

        // Get scrollback lines from config
        // Create terminal bounds (scrollback is configured in Terminal, not bounds)
        let terminal_bounds = TerminalBounds::new(line_height, cell_width, bounds);

        // Update terminal size
        self.terminal.update(cx, |terminal, _cx| {
            terminal.set_bounds(terminal_bounds);
        });

        // Get content
        let content = self.terminal.read(cx).content().clone();

        let cursor_line = content.cursor.point.line.0 + content.display_offset as i32;
        let cursor_col = content.cursor.point.column.0;
        let cursor_shape = content.cursor.shape;
        let cursor_visible = !matches!(cursor_shape, CursorShape::Hidden);

        // Layout all cells from alacritty's renderable_content
        // Alacritty already handles display_offset and returns the correct visible cells
        // We pass display_offset to convert terminal coordinates to visual coordinates
        let (background_rects, text_runs) = self.layout_grid(
            content.cells.iter().cloned(),
            content.display_offset,
            cursor_visible,
            cursor_line,
            cursor_col,
            &font,
            font_size,
        );

        // Calculate y_offset for bottom alignment
        // Ensure y_offset is non-negative (top-align when content exceeds bounds)
        let screen_lines = content.screen_lines;
        let content_height = line_height * screen_lines as f32;
        let y_offset = (bounds.size.height - content_height).max(px(0.0));

        // Calculate actual rows that fit in bounds
        let actual_rows = (bounds.size.height / line_height).floor() as usize;
        eprintln!("[prepaint] screen_lines: {}, actual_rows: {}, bounds.height: {}, line_height: {}, content_height: {}, y_offset: {}",
            screen_lines, actual_rows, bounds.size.height, line_height, content_height, y_offset);

        // Cursor bounds for IME
        let cursor_bounds = Some(Bounds::new(
            point(
                cell_width * cursor_col as f32,
                y_offset + line_height * cursor_line as f32,
            ),
            size(cell_width, line_height),
        ));

        // Base text style for IME
        let base_text_style = TextStyle {
            font_family: self.theme.font_family.clone().into(),
            font_size: font_size.into(),
            color: self.theme.foreground.into(),
            ..Default::default()
        };

        LayoutState {
            text_runs,
            background_rects,
            cursor_line,
            cursor_col,
            cursor_shape,
            cell_width,
            line_height,
            background: self.theme.background,
            cursor_color: self.theme.cursor_color,
            cursor_bounds,
            base_text_style,
            y_offset,
            cursor_visible,
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
        if let Some(ref shared_bounds) = self.shared_bounds {
            shared_bounds.bounds.set(Some(bounds));
            shared_bounds.cell_width.set(Some(layout.cell_width));
            shared_bounds.line_height.set(Some(layout.line_height));
            shared_bounds.y_offset.set(Some(layout.y_offset));
        }

        // Use content mask to prevent rendering outside bounds
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            // Paint background
            window.paint_quad(fill(bounds, layout.background));

            let origin = point(bounds.origin.x, bounds.origin.y + layout.y_offset);

            // Get marked text (IME composition)
        let marked_text: Option<String> = {
            let ime_state = &self.terminal_view.read(cx).ime_state;
            ime_state.as_ref().map(|state| state.marked_text.clone())
        };

        // Register input handler for IME
        let terminal_input_handler = TerminalInputHandler {
            terminal_view: self.terminal_view.clone(),
            cursor_bounds: layout.cursor_bounds.map(|b| b + bounds.origin),
        };
        window.handle_input(&self.focus, terminal_input_handler, cx);

        // Paint background rectangles
        for rect in &layout.background_rects {
            rect.paint(origin, layout.cell_width, layout.line_height, window);
        }

        // Paint text runs
        for run in &layout.text_runs {
            run.paint(origin, layout.cell_width, layout.line_height, window, cx);
        }

        // Paint IME marked text
        if let Some(ref text_to_mark) = marked_text {
            if !text_to_mark.is_empty() {
                if let Some(ime_bounds) = layout.cursor_bounds {
                    let ime_position = (ime_bounds + bounds.origin).origin;
                    let font = Font {
                        family: self.theme.font_family.clone().into(),
                        ..Default::default()
                    };
                    let font_size = px(self.theme.font_size);

                    let ime_text_run = gpui::TextRun {
                        len: text_to_mark.len(),
                        font,
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

                    let ime_background_bounds =
                        Bounds::new(ime_position, size(shaped_line.width, layout.line_height));
                    window.paint_quad(fill(ime_background_bounds, layout.background));

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

        // Paint cursor (with multiple shape support)
        if marked_text.is_none() && layout.cursor_visible {
            // Use the same origin calculation as text/background for consistency
            let cursor_x = origin.x + layout.cell_width * layout.cursor_col as f32;
            let cursor_y = origin.y + layout.line_height * layout.cursor_line as f32;
            let cursor_bounds = Bounds::new(
                point(cursor_x, cursor_y),
                size(layout.cell_width, layout.line_height),
            );

            let focused = self.focus.is_focused(window);
            Self::paint_cursor(layout.cursor_shape, cursor_bounds, layout.cursor_color, focused, window);
        }
        }); // End of with_content_mask
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
        let mut bounds = self.cursor_bounds?;
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
