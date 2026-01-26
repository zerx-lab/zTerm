//! VTE Performer - 将 VTE 解析事件应用到 TerminalState

use parking_lot::Mutex;
use std::sync::Arc;
use vte::{Params, Perform};

use crate::event::{TerminalEvent, TerminalEventListener};
use crate::grid::{Cell, CellAttributes, Color, CursorShape, Intensity, TerminalState, UnderlineStyle};

/// VTE Performer
pub struct VtePerformer {
    /// 终端状态
    state: Arc<Mutex<TerminalState>>,
    /// 事件监听器
    event_listener: Arc<dyn TerminalEventListener>,
}

impl VtePerformer {
    pub fn new(
        state: Arc<Mutex<TerminalState>>,
        event_listener: Arc<dyn TerminalEventListener>,
    ) -> Self {
        Self {
            state,
            event_listener,
        }
    }

    /// 应用 SGR (Select Graphic Rendition) 参数
    fn handle_sgr(&mut self, params: &Params) {
        let mut state = self.state.lock();
        let mut attrs = state.attrs().clone();

        let mut iter = params.iter();
        while let Some(param) = iter.next() {
            let value = param[0];
            match value {
                0 => attrs = CellAttributes::default(), // Reset
                1 => attrs.set_intensity(Intensity::Bold), // Bold
                2 => attrs.set_intensity(Intensity::Dim), // Dim
                3 => attrs.set_italic(true), // Italic
                4 => attrs.set_underline(UnderlineStyle::Single), // Underline
                5 | 6 => attrs.set_italic(true), // Blink (用 italic 代替)
                7 => attrs.set_reverse(true), // Reverse
                8 => attrs.set_invisible(true), // Invisible
                9 => attrs.set_strikethrough(true), // Strikethrough
                21 => attrs.set_underline(UnderlineStyle::Double), // Double underline
                22 => attrs.set_intensity(Intensity::Normal), // Normal intensity
                23 => attrs.set_italic(false), // Not italic
                24 => attrs.set_underline(UnderlineStyle::None), // Not underlined
                25 => attrs.set_italic(false), // Not blinking
                27 => attrs.set_reverse(false), // Not reversed
                28 => attrs.set_invisible(false), // Not invisible
                29 => attrs.set_strikethrough(false), // Not crossed out
                // 前景色 (30-37, 90-97)
                30..=37 => attrs.set_foreground(Color::Indexed((value - 30) as u8)),
                38 => {
                    // 256 color or RGB
                    if let Some(color) = Self::parse_color_sequence(&mut iter) {
                        attrs.set_foreground(color);
                    }
                }
                39 => attrs.set_foreground(Color::DefaultForeground), // Default foreground
                // 背景色 (40-47, 100-107)
                40..=47 => attrs.set_background(Color::Indexed((value - 40) as u8)),
                48 => {
                    // 256 color or RGB
                    if let Some(color) = Self::parse_color_sequence(&mut iter) {
                        attrs.set_background(color);
                    }
                }
                49 => attrs.set_background(Color::DefaultBackground), // Default background
                // 高亮前景色 (90-97)
                90..=97 => attrs.set_foreground(Color::Indexed((value - 90 + 8) as u8)),
                // 高亮背景色 (100-107)
                100..=107 => attrs.set_background(Color::Indexed((value - 100 + 8) as u8)),
                _ => {}
            }
        }

        state.set_attrs(attrs);
    }

    fn parse_color_sequence<'a>(iter: &mut impl Iterator<Item = &'a [u16]>) -> Option<Color> {
        let mode = iter.next()?[0];
        match mode {
            2 => {
                // RGB
                let r = iter.next()?[0] as u8;
                let g = iter.next()?[0] as u8;
                let b = iter.next()?[0] as u8;
                Some(Color::Rgb { r, g, b })
            }
            5 => {
                // 256 color
                let idx = iter.next()?[0] as u8;
                Some(Color::Indexed(idx))
            }
            _ => None,
        }
    }
}

impl Perform for VtePerformer {
    fn print(&mut self, c: char) {
        let mut state = self.state.lock();
        let attrs = state.attrs().clone();
        let cell = Cell::with_attrs_auto(&c.to_string(), attrs);
        state.write_cell(cell);
        self.event_listener.on_event(TerminalEvent::Wakeup);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x07 => {
                // BEL (Bell)
                self.event_listener.on_event(TerminalEvent::BellRing);
            }
            0x08 => {
                // BS (Backspace)
                let mut state = self.state.lock();
                state.cursor_mut().move_left(1);
            }
            0x09 => {
                // HT (Tab) - 移动到下一个 8 的倍数列
                let mut state = self.state.lock();
                let max_cols = state.cols();
                let cursor = state.cursor_mut();
                let next_tab = ((cursor.x / 8) + 1) * 8;
                cursor.x = next_tab.min(max_cols.saturating_sub(1));
            }
            0x0A | 0x0B | 0x0C => {
                // LF, VT, FF (Line feed)
                let mut state = self.state.lock();
                let max_rows = state.rows();
                state.cursor_mut().line_feed(max_rows);
            }
            0x0D => {
                // CR (Carriage return)
                let mut state = self.state.lock();
                state.cursor_mut().carriage_return();
            }
            _ => {
                tracing::trace!("Execute control: 0x{:02x}", byte);
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        if ignore {
            return;
        }

        match action {
            'm' => self.handle_sgr(params), // SGR
            'A' => {
                // CUU (Cursor Up)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                self.state.lock().cursor_mut().move_up(n);
            }
            'B' => {
                // CUD (Cursor Down)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                let max_rows = self.state.lock().rows();
                self.state.lock().cursor_mut().move_down(n, max_rows);
            }
            'C' => {
                // CUF (Cursor Forward)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                let max_cols = self.state.lock().cols();
                self.state.lock().cursor_mut().move_right(n, max_cols);
            }
            'D' => {
                // CUB (Cursor Back)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                self.state.lock().cursor_mut().move_left(n);
            }
            'H' | 'f' => {
                // CUP (Cursor Position)
                let mut iter = params.iter();
                let row = iter.next().map(|p| p[0]).unwrap_or(1).max(1) as usize - 1;
                let col = iter.next().map(|p| p[0]).unwrap_or(1).max(1) as usize - 1;
                self.state.lock().set_cursor_pos(col, row);
            }
            'J' => {
                // ED (Erase Display)
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                match mode {
                    0 | 1 | 2 => {
                        // TODO: 实现擦除显示
                        tracing::debug!("ED mode {}", mode);
                    }
                    _ => {}
                }
            }
            'K' => {
                // EL (Erase Line)
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                match mode {
                    0 | 1 | 2 => {
                        // TODO: 实现擦除行
                        tracing::debug!("EL mode {}", mode);
                    }
                    _ => {}
                }
            }
            'h' => {
                // Set Mode
                if !intermediates.is_empty() && intermediates[0] == b'?' {
                    // DEC Private Mode
                    for param in params.iter() {
                        match param[0] {
                            25 => self.state.lock().set_cursor_visible(true), // DECTCEM
                            1049 => self.state.lock().switch_to_alternate_screen(),
                            _ => {}
                        }
                    }
                }
            }
            'l' => {
                // Reset Mode
                if !intermediates.is_empty() && intermediates[0] == b'?' {
                    for param in params.iter() {
                        match param[0] {
                            25 => self.state.lock().set_cursor_visible(false), // DECTCEM
                            1049 => self.state.lock().switch_to_main_screen(),
                            _ => {}
                        }
                    }
                }
            }
            'q' => {
                // DECSCUSR - Set Cursor Style
                if intermediates.is_empty() {
                    let style = params.iter().next().map(|p| p[0]).unwrap_or(0);
                    let shape = match style {
                        0 | 1 => CursorShape::Block,
                        2 => CursorShape::Block,
                        3 | 4 => CursorShape::Underline,
                        5 | 6 => CursorShape::Bar,
                        _ => CursorShape::Block,
                    };
                    self.state.lock().set_cursor_shape(shape);
                }
            }
            _ => {
                tracing::debug!("Unhandled CSI: {} params={:?}", action, params);
            }
        }

        self.event_listener.on_event(TerminalEvent::Wakeup);
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        if ignore {
            return;
        }

        match byte {
            b'D' => {
                // IND (Index) - 向下滚动
                self.state.lock().screen_mut().scroll_up();
            }
            b'M' => {
                // RI (Reverse Index) - 向上滚动
                self.state.lock().screen_mut().scroll_down();
            }
            b'c' => {
                // RIS (Reset to Initial State)
                self.state.lock().clear_screen();
            }
            _ => {
                tracing::debug!("Unhandled ESC: 0x{:02x} intermediates={:?}", byte, intermediates);
            }
        }

        self.event_listener.on_event(TerminalEvent::Wakeup);
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.is_empty() {
            return;
        }

        let cmd = String::from_utf8_lossy(params[0]);

        match cmd.as_ref() {
            "0" | "2" => {
                // Set window title
                if params.len() > 1 {
                    let title = String::from_utf8_lossy(params[1]).to_string();
                    self.state.lock().set_title(title.clone());
                    self.event_listener
                        .on_event(TerminalEvent::TitleChanged(title));
                }
            }
            #[cfg(feature = "shell-integration")]
            "133" => {
                self.handle_osc_133(params);
            }
            #[cfg(feature = "shell-integration")]
            "633" => {
                self.handle_osc_633(params);
            }
            _ => {
                tracing::debug!("Unhandled OSC: {}", cmd);
            }
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}
}

#[cfg(feature = "shell-integration")]
impl VtePerformer {
    fn handle_osc_133(&mut self, params: &[&[u8]]) {
        use crate::event::ShellIntegrationEvent;

        if params.len() < 2 {
            return;
        }

        let subcommand = String::from_utf8_lossy(params[1]);
        let state = self.state.lock();
        let cursor_y = state.cursor().y;

        match subcommand.as_ref() {
            "A" => {
                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::PromptStart { line: cursor_y },
                ));
            }
            "B" => {
                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::CommandStart { line: cursor_y },
                ));
            }
            "C" => {
                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::CommandExecuting { line: cursor_y },
                ));
            }
            "D" => {
                let exit_code = if params.len() > 2 {
                    String::from_utf8_lossy(params[2])
                        .split(';')
                        .next()
                        .and_then(|s| s.parse::<i32>().ok())
                } else {
                    None
                };

                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::CommandFinished {
                        line: cursor_y,
                        exit_code,
                    },
                ));
            }
            _ => {}
        }
    }

    fn handle_osc_633(&mut self, params: &[&[u8]]) {
        use crate::event::ShellIntegrationEvent;

        if params.len() < 2 {
            return;
        }

        let subcommand = String::from_utf8_lossy(params[1]);

        if let Some(cwd) = subcommand.strip_prefix("P;Cwd=") {
            self.event_listener.on_event(TerminalEvent::ShellIntegration(
                ShellIntegrationEvent::WorkingDirectoryChanged(cwd.to_string()),
            ));
        }
    }
}
