//! Terminal rendering element

use crate::theme::TerminalTheme;
use axon_terminal::buffer::{Color, Grid};
use gpui::*;

/// Terminal element for rendering terminal content
pub struct TerminalElement {
    /// Lines to render
    lines: Vec<RenderedLine>,

    /// Cursor position
    cursor: (usize, usize),

    /// Theme
    theme: TerminalTheme,
}

/// A pre-processed line for rendering
struct RenderedLine {
    text: String,
}

impl TerminalElement {
    /// Create a new terminal element
    pub fn new(grid: &Grid, theme: TerminalTheme) -> Self {
        let cursor = grid.cursor();

        // Pre-process lines for rendering
        let lines: Vec<RenderedLine> = (0..grid.rows())
            .filter_map(|row_idx| {
                grid.get_row(row_idx).map(|row| {
                    let text: String = row.iter().map(|cell| cell.c).collect();
                    RenderedLine { text }
                })
            })
            .collect();

        Self {
            lines,
            cursor,
            theme,
        }
    }

    /// Convert a terminal color to GPUI color
    #[allow(dead_code)]
    fn color_to_gpui(&self, color: &Color, is_foreground: bool) -> Rgba {
        match color {
            Color::Default => {
                if is_foreground {
                    self.theme.foreground
                } else {
                    self.theme.background
                }
            }
            Color::Named(idx) => self.theme.ansi_colors[*idx as usize % 16],
            Color::Indexed(idx) => {
                // 256 color palette
                if *idx < 16 {
                    self.theme.ansi_colors[*idx as usize]
                } else if *idx < 232 {
                    // 216 color cube (6x6x6)
                    let idx = *idx - 16;
                    let r = (idx / 36) % 6;
                    let g = (idx / 6) % 6;
                    let b = idx % 6;
                    let r = if r > 0 { r * 40 + 55 } else { 0 };
                    let g = if g > 0 { g * 40 + 55 } else { 0 };
                    let b = if b > 0 { b * 40 + 55 } else { 0 };
                    rgba(r as u32 * 0x10000 + g as u32 * 0x100 + b as u32 + 0xff000000)
                } else {
                    // 24 grayscale
                    let gray = (*idx - 232) * 10 + 8;
                    rgba(gray as u32 * 0x10101 + 0xff000000)
                }
            }
            Color::Rgb(r, g, b) => {
                rgba((*r as u32) << 24 | (*g as u32) << 16 | (*b as u32) << 8 | 0xff)
            }
        }
    }
}

impl RenderOnce for TerminalElement {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = self.theme.clone();
        let cursor_col = self.cursor.0;
        let cursor_row = self.cursor.1;

        div()
            .flex()
            .flex_col()
            .font_family(theme.font_family.clone())
            .text_size(px(theme.font_size))
            .line_height(px(theme.font_size * theme.line_height))
            .children(self.lines.into_iter().enumerate().map(|(row_idx, line)| {
                let is_cursor_row = row_idx == cursor_row;

                div()
                    .flex()
                    .flex_row()
                    .whitespace_nowrap()
                    .child(if is_cursor_row {
                        // Render line with cursor
                        let before_cursor: String = line.text.chars().take(cursor_col).collect();
                        let cursor_char: String = line.text.chars().nth(cursor_col).map(|c| c.to_string()).unwrap_or(" ".to_string());
                        let after_cursor: String = line.text.chars().skip(cursor_col + 1).collect();

                        div()
                            .flex()
                            .flex_row()
                            .child(div().child(before_cursor))
                            .child(
                                div()
                                    .bg(theme.cursor_color)
                                    .text_color(theme.background)
                                    .child(cursor_char)
                            )
                            .child(div().child(after_cursor))
                    } else {
                        div().child(line.text)
                    })
            }))
    }
}
