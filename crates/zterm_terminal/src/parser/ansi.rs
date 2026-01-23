//! ANSI/VT sequence parser wrapper

use crate::buffer::{CellFlags, Color, Grid};
use vte::{Params, Parser, Perform};

/// ANSI parser state
pub struct AnsiParser {
    parser: Parser,
    handler: AnsiHandler,
}

impl AnsiParser {
    /// Create a new ANSI parser
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            handler: AnsiHandler::new(),
        }
    }

    /// Process input bytes
    pub fn process(&mut self, input: &[u8], grid: &mut Grid) {
        self.handler.grid = Some(grid as *mut Grid);
        self.parser.advance(&mut self.handler, input);
        self.handler.grid = None;
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for VTE parser events
struct AnsiHandler {
    grid: Option<*mut Grid>,
    current_fg: Color,
    current_bg: Color,
    current_flags: CellFlags,
}

impl AnsiHandler {
    fn new() -> Self {
        Self {
            grid: None,
            current_fg: Color::Default,
            current_bg: Color::Default,
            current_flags: CellFlags::default(),
        }
    }

    fn grid_mut(&mut self) -> Option<&mut Grid> {
        self.grid.map(|ptr| unsafe { &mut *ptr })
    }
}

impl Perform for AnsiHandler {
    fn print(&mut self, c: char) {
        // Copy state before borrowing grid
        let fg = self.current_fg;
        let bg = self.current_bg;
        let flags = self.current_flags;

        if let Some(grid) = self.grid_mut() {
            let (col, row) = grid.cursor();
            if let Some(cell) = grid.get_cell_mut(col, row) {
                cell.c = c;
                cell.fg = fg;
                cell.bg = bg;
                cell.flags = flags;
            }
            // Move cursor right
            grid.set_cursor(col + 1, row);
        }
    }

    fn execute(&mut self, byte: u8) {
        if let Some(grid) = self.grid_mut() {
            match byte {
                // Bell
                0x07 => {
                    // TODO: Emit bell event
                }
                // Backspace
                0x08 => {
                    let (col, row) = grid.cursor();
                    if col > 0 {
                        grid.set_cursor(col - 1, row);
                    }
                }
                // Tab
                0x09 => {
                    let (col, row) = grid.cursor();
                    let next_tab = (col / 8 + 1) * 8;
                    grid.set_cursor(next_tab.min(grid.cols() - 1), row);
                }
                // Line feed / Vertical tab / Form feed
                0x0A | 0x0B | 0x0C => {
                    let (col, row) = grid.cursor();
                    if row + 1 >= grid.rows() {
                        grid.scroll_up(1);
                    } else {
                        grid.set_cursor(col, row + 1);
                    }
                }
                // Carriage return
                0x0D => {
                    let (_, row) = grid.cursor();
                    grid.set_cursor(0, row);
                }
                _ => {}
            }
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS sequences - not commonly used
    }

    fn put(&mut self, _byte: u8) {
        // DCS data
    }

    fn unhook(&mut self) {
        // End of DCS sequence
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences - operating system commands
        if params.is_empty() {
            return;
        }

        match params[0] {
            // Set window title
            b"0" | b"2" if params.len() > 1 => {
                let _title = String::from_utf8_lossy(params[1]);
                // TODO: Emit title change event
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let params: Vec<u16> = params.iter().map(|p| p[0]).collect();

        // Handle SGR (Select Graphic Rendition) separately as it doesn't need grid
        if action == 'm' {
            self.process_sgr(&params);
            return;
        }

        if let Some(grid) = self.grid_mut() {
            match action {
                // Cursor Up
                'A' => {
                    let n = params.first().copied().unwrap_or(1).max(1) as usize;
                    let (col, row) = grid.cursor();
                    grid.set_cursor(col, row.saturating_sub(n));
                }
                // Cursor Down
                'B' => {
                    let n = params.first().copied().unwrap_or(1).max(1) as usize;
                    let (col, row) = grid.cursor();
                    grid.set_cursor(col, (row + n).min(grid.rows() - 1));
                }
                // Cursor Forward
                'C' => {
                    let n = params.first().copied().unwrap_or(1).max(1) as usize;
                    let (col, row) = grid.cursor();
                    grid.set_cursor((col + n).min(grid.cols() - 1), row);
                }
                // Cursor Back
                'D' => {
                    let n = params.first().copied().unwrap_or(1).max(1) as usize;
                    let (col, row) = grid.cursor();
                    grid.set_cursor(col.saturating_sub(n), row);
                }
                // Cursor Position
                'H' | 'f' => {
                    let row = params.first().copied().unwrap_or(1).max(1) as usize - 1;
                    let col = params.get(1).copied().unwrap_or(1).max(1) as usize - 1;
                    grid.set_cursor(col, row);
                }
                // Erase in Display
                'J' => {
                    let mode = params.first().copied().unwrap_or(0);
                    let (col, row) = grid.cursor();
                    match mode {
                        0 => {
                            // Clear from cursor to end of screen
                            for r in row..grid.rows() {
                                let start_col = if r == row { col } else { 0 };
                                if let Some(grid_row) = grid.get_row_mut(r) {
                                    for c in start_col..grid_row.len() {
                                        grid_row[c].reset();
                                    }
                                }
                            }
                        }
                        1 => {
                            // Clear from beginning to cursor
                            for r in 0..=row {
                                let end_col = if r == row { col + 1 } else { grid.cols() };
                                if let Some(grid_row) = grid.get_row_mut(r) {
                                    for c in 0..end_col {
                                        grid_row[c].reset();
                                    }
                                }
                            }
                        }
                        2 | 3 => {
                            // Clear entire screen
                            grid.clear();
                        }
                        _ => {}
                    }
                }
                // Erase in Line
                'K' => {
                    let mode = params.first().copied().unwrap_or(0);
                    let (col, row) = grid.cursor();
                    if let Some(grid_row) = grid.get_row_mut(row) {
                        match mode {
                            0 => {
                                // Clear from cursor to end of line
                                for c in col..grid_row.len() {
                                    grid_row[c].reset();
                                }
                            }
                            1 => {
                                // Clear from beginning to cursor
                                for c in 0..=col {
                                    grid_row[c].reset();
                                }
                            }
                            2 => {
                                // Clear entire line
                                grid_row.clear();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // ESC sequences
    }
}

impl AnsiHandler {
    fn process_sgr(&mut self, params: &[u16]) {
        if params.is_empty() {
            // Reset all attributes
            self.current_fg = Color::Default;
            self.current_bg = Color::Default;
            self.current_flags = CellFlags::default();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => {
                    self.current_fg = Color::Default;
                    self.current_bg = Color::Default;
                    self.current_flags = CellFlags::default();
                }
                1 => self.current_flags.bold = true,
                2 => self.current_flags.dim = true,
                3 => self.current_flags.italic = true,
                4 => self.current_flags.underline = true,
                5 | 6 => self.current_flags.blink = true,
                7 => self.current_flags.inverse = true,
                8 => self.current_flags.hidden = true,
                9 => self.current_flags.strikethrough = true,
                22 => {
                    self.current_flags.bold = false;
                    self.current_flags.dim = false;
                }
                23 => self.current_flags.italic = false,
                24 => self.current_flags.underline = false,
                25 => self.current_flags.blink = false,
                27 => self.current_flags.inverse = false,
                28 => self.current_flags.hidden = false,
                29 => self.current_flags.strikethrough = false,
                // Foreground colors
                30..=37 => self.current_fg = Color::Named((params[i] - 30) as u8),
                38 => {
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        // 256 color
                        self.current_fg = Color::Indexed(params[i + 2] as u8);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        // True color
                        self.current_fg = Color::Rgb(
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                39 => self.current_fg = Color::Default,
                // Background colors
                40..=47 => self.current_bg = Color::Named((params[i] - 40) as u8),
                48 => {
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        // 256 color
                        self.current_bg = Color::Indexed(params[i + 2] as u8);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        // True color
                        self.current_bg = Color::Rgb(
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                49 => self.current_bg = Color::Default,
                // Bright foreground colors
                90..=97 => self.current_fg = Color::Named((params[i] - 90 + 8) as u8),
                // Bright background colors
                100..=107 => self.current_bg = Color::Named((params[i] - 100 + 8) as u8),
                _ => {}
            }
            i += 1;
        }
    }
}
