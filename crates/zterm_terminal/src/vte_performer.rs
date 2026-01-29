//! VTE Performer - 将 VTE 解析事件应用到 TerminalState
//!
//! 性能优化：使用脏标记代替每次操作触发 Wakeup 事件
//! 在一批数据处理完成后统一检查并发送 Wakeup

use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use vte::{Params, Perform};

use crate::event::{TerminalEvent, TerminalEventListener};
use crate::grid::{
    Cell, CellAttributes, Color, CursorShape, Intensity, TerminalState, UnderlineStyle,
};

/// VTE Performer
pub struct VtePerformer {
    /// 终端状态
    state: Arc<Mutex<TerminalState>>,
    /// 事件监听器
    event_listener: Arc<dyn TerminalEventListener>,
    /// 脏标记：是否需要发送 Wakeup 事件
    /// 用于批量处理数据后统一触发一次刷新，而不是每个字符触发一次
    needs_wakeup: Arc<AtomicBool>,
}

impl VtePerformer {
    pub fn new(
        state: Arc<Mutex<TerminalState>>,
        event_listener: Arc<dyn TerminalEventListener>,
    ) -> Self {
        Self {
            state,
            event_listener,
            needs_wakeup: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 标记需要 Wakeup（设置脏标记）
    #[inline]
    fn mark_dirty(&self) {
        self.needs_wakeup.store(true, Ordering::Release);
    }

    /// 检查并清除脏标记，返回是否需要发送 Wakeup
    /// 调用后脏标记会被清除
    pub fn take_needs_wakeup(&self) -> bool {
        self.needs_wakeup.swap(false, Ordering::AcqRel)
    }

    /// 检查脏标记并发送 Wakeup 事件（如果需要）
    /// 这是一个便捷方法，用于批量处理后统一发送 Wakeup
    pub fn flush_wakeup(&self) {
        if self.take_needs_wakeup() {
            self.event_listener.on_event(TerminalEvent::Wakeup);
        }
    }

    /// 发送非 Wakeup 事件（这些事件不适合批处理）
    #[inline]
    fn send_event(&self, event: TerminalEvent) {
        self.event_listener.on_event(event);
    }

    /// 应用 SGR (Select Graphic Rendition) 参数
    fn handle_sgr(&mut self, params: &Params) {
        let mut state = self.state.lock();
        let mut attrs = state.attrs().clone();

        let mut iter = params.iter();
        while let Some(param) = iter.next() {
            let value = param[0];
            match value {
                0 => attrs = CellAttributes::default(),            // Reset
                1 => attrs.set_intensity(Intensity::Bold),         // Bold
                2 => attrs.set_intensity(Intensity::Dim),          // Dim
                3 => attrs.set_italic(true),                       // Italic
                4 => attrs.set_underline(UnderlineStyle::Single),  // Underline
                5 | 6 => attrs.set_italic(true),                   // Blink (用 italic 代替)
                7 => attrs.set_reverse(true),                      // Reverse
                8 => attrs.set_invisible(true),                    // Invisible
                9 => attrs.set_strikethrough(true),                // Strikethrough
                21 => attrs.set_underline(UnderlineStyle::Double), // Double underline
                22 => attrs.set_intensity(Intensity::Normal),      // Normal intensity
                23 => attrs.set_italic(false),                     // Not italic
                24 => attrs.set_underline(UnderlineStyle::None),   // Not underlined
                25 => attrs.set_italic(false),                     // Not blinking
                27 => attrs.set_reverse(false),                    // Not reversed
                28 => attrs.set_invisible(false),                  // Not invisible
                29 => attrs.set_strikethrough(false),              // Not crossed out
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
        // 使用 trace 级别避免性能影响
        tracing::trace!("VtePerformer::print('{}') at cursor", c.escape_debug());

        let mut state = self.state.lock();
        let attrs = state.attrs().clone();
        let cell = Cell::with_attrs_auto(&c.to_string(), attrs);
        state.write_cell(cell);

        // 标记脏而不是立即发送 Wakeup
        // Wakeup 将在 process_pty_data 完成后统一发送
        self.mark_dirty();
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
                // 使用 new_line(false) 正确处理滚动和重绘检测
                self.state.lock().new_line(false);
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
                tracing::debug!("CSI H/f (CUP): move cursor to row={}, col={}", row, col);
                self.state.lock().set_cursor_pos(col, row);
            }
            'J' => {
                // ED (Erase Display)
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                tracing::trace!("ED (Erase Display) mode={}", mode);
                self.state.lock().erase_display(mode);
            }
            'K' => {
                // EL (Erase Line)
                let mode = params.iter().next().map(|p| p[0]).unwrap_or(0);
                tracing::trace!("EL (Erase Line) mode={}", mode);
                self.state.lock().erase_line(mode);
            }
            'L' => {
                // IL (Insert Line)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                tracing::trace!("IL (Insert Line) n={}", n);
                let mut state = self.state.lock();
                for _ in 0..n {
                    let y = state.cursor().y;
                    let vis_idx = super::grid::VisibleRowIndex::new(y as isize);
                    if let Some(phys) = state.screen().visible_to_phys(vis_idx) {
                        state.screen_mut().insert_line(phys);
                    }
                }
            }
            'M' => {
                // DL (Delete Line)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                tracing::trace!("DL (Delete Line) n={}", n);
                let mut state = self.state.lock();
                for _ in 0..n {
                    let y = state.cursor().y;
                    let vis_idx = super::grid::VisibleRowIndex::new(y as isize);
                    if let Some(phys) = state.screen().visible_to_phys(vis_idx) {
                        state.screen_mut().delete_line(phys);
                    }
                }
            }
            '@' => {
                // ICH (Insert Character)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                tracing::trace!("ICH (Insert Character) n={}", n);
                let mut state = self.state.lock();
                let x = state.cursor().x;
                let y = state.cursor().y;
                let cols = state.cols();
                let vis_idx = super::grid::VisibleRowIndex::new(y as isize);
                if let Some(phys) = state.screen().visible_to_phys(vis_idx) {
                    if let Some(line) = state.screen_mut().get_line_mut(phys) {
                        // 在当前位置插入 n 个空白字符
                        for _ in 0..n {
                            if x < cols {
                                line.resize(line.len().max(x + 1), super::grid::Cell::blank());
                                // 简单实现：将后面的字符右移
                                let cells = line.to_vec();
                                let mut new_cells = cells[..x].to_vec();
                                new_cells.push(super::grid::Cell::blank());
                                new_cells.extend_from_slice(&cells[x..]);
                                new_cells.truncate(cols);
                                *line = super::grid::Line::from_cells(new_cells);
                            }
                        }
                    }
                }
            }
            'P' => {
                // DCH (Delete Character)
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                tracing::trace!("DCH (Delete Character) n={}", n);
                let mut state = self.state.lock();
                let x = state.cursor().x;
                let y = state.cursor().y;
                let vis_idx = super::grid::VisibleRowIndex::new(y as isize);
                if let Some(phys) = state.screen().visible_to_phys(vis_idx) {
                    if let Some(line) = state.screen_mut().get_line_mut(phys) {
                        let mut cells = line.to_vec();
                        // 删除从当前位置开始的 n 个字符
                        if x < cells.len() {
                            let end = (x + n).min(cells.len());
                            cells.drain(x..end);
                        }
                        *line = super::grid::Line::from_cells(cells);
                    }
                }
            }
            'S' => {
                // SU (Scroll Up) - 向上滚动 n 行
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                tracing::trace!("SU (Scroll Up) n={}", n);
                // 使用 state.scroll_up 确保重绘检测正常工作
                self.state.lock().scroll_up(n);
            }
            'T' => {
                // SD (Scroll Down) - 向下滚动 n 行
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                tracing::trace!("SD (Scroll Down) n={}", n);
                let mut state = self.state.lock();
                for _ in 0..n {
                    state.screen_mut().scroll_down();
                }
            }
            'X' => {
                // ECH (Erase Character) - 擦除当前位置开始的 n 个字符
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                tracing::trace!("ECH (Erase Character) n={}", n);
                let mut state = self.state.lock();
                let x = state.cursor().x;
                let y = state.cursor().y;
                let vis_idx = super::grid::VisibleRowIndex::new(y as isize);
                if let Some(phys) = state.screen().visible_to_phys(vis_idx) {
                    if let Some(line) = state.screen_mut().get_line_mut(phys) {
                        for i in 0..n {
                            if x + i < line.len() {
                                line.set_cell(x + i, super::grid::Cell::blank());
                            }
                        }
                    }
                }
            }
            'E' => {
                // CNL (Cursor Next Line) - 移动到下 n 行的开头
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                let mut state = self.state.lock();
                let max_rows = state.rows();
                state.cursor_mut().move_down(n, max_rows);
                state.cursor_mut().carriage_return();
            }
            'F' => {
                // CPL (Cursor Previous Line) - 移动到上 n 行的开头
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize;
                let mut state = self.state.lock();
                state.cursor_mut().move_up(n);
                state.cursor_mut().carriage_return();
            }
            'G' => {
                // CHA (Cursor Character Absolute) - 移动到指定列
                let col = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize - 1;
                let mut state = self.state.lock();
                let y = state.cursor().y;
                tracing::debug!(
                    "CSI G (CHA): move cursor to col={} (row stays at {})",
                    col,
                    y
                );
                state.set_cursor_pos(col, y);
            }
            'd' => {
                // VPA (Vertical Position Absolute) - 移动到指定行
                let row = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1) as usize - 1;
                let mut state = self.state.lock();
                let x = state.cursor().x;
                state.set_cursor_pos(x, row);
            }
            'h' => {
                // Set Mode
                if !intermediates.is_empty() && intermediates[0] == b'?' {
                    // DEC Private Mode
                    for param in params.iter() {
                        match param[0] {
                            7 => {
                                // DECAWM - Auto Wrap Mode
                                self.state
                                    .lock()
                                    .set_mode(super::grid::TerminalModes::AUTO_WRAP, true);
                                tracing::debug!("DECAWM: Auto wrap enabled");
                            }
                            25 => self.state.lock().set_cursor_visible(true), // DECTCEM
                            1049 => self.state.lock().switch_to_alternate_screen(),
                            _ => {
                                tracing::trace!("Unhandled DEC Private Mode Set: {}", param[0]);
                            }
                        }
                    }
                }
            }
            'l' => {
                // Reset Mode
                if !intermediates.is_empty() && intermediates[0] == b'?' {
                    for param in params.iter() {
                        match param[0] {
                            7 => {
                                // DECAWM - Auto Wrap Mode
                                self.state
                                    .lock()
                                    .set_mode(super::grid::TerminalModes::AUTO_WRAP, false);
                                tracing::debug!("DECAWM: Auto wrap disabled");
                            }
                            25 => self.state.lock().set_cursor_visible(false), // DECTCEM
                            1049 => self.state.lock().switch_to_main_screen(),
                            _ => {
                                tracing::trace!("Unhandled DEC Private Mode Reset: {}", param[0]);
                            }
                        }
                    }
                }
            }
            'n' => {
                // DSR (Device Status Report)
                let param = params.iter().next().map(|p| p[0]).unwrap_or(0);
                match param {
                    5 => {
                        // Status report - respond with "OK"
                        tracing::debug!("DSR: Status Report requested");
                        self.event_listener
                            .on_event(TerminalEvent::PtyWrite(b"\x1b[0n".to_vec()));
                    }
                    6 => {
                        // Cursor Position Report (CPR)
                        let state = self.state.lock();
                        let cursor = state.cursor();
                        // 终端坐标从1开始,所以+1
                        let response = format!("\x1b[{};{}R", cursor.y + 1, cursor.x + 1);
                        tracing::debug!(
                            "DSR: Cursor position report: row={}, col={}",
                            cursor.y + 1,
                            cursor.x + 1
                        );
                        drop(state);
                        self.event_listener
                            .on_event(TerminalEvent::PtyWrite(response.into_bytes()));
                    }
                    _ => {
                        tracing::debug!("Unhandled DSR param: {}", param);
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
            'r' => {
                // DECSTBM - Set Top and Bottom Margins (Scrolling Region)
                let mut iter = params.iter();
                let top = iter.next().map(|p| p[0]).unwrap_or(1).max(1) as usize - 1;
                let bottom = iter
                    .next()
                    .map(|p| p[0] as usize)
                    .unwrap_or_else(|| self.state.lock().rows())
                    .saturating_sub(1);
                tracing::debug!("DECSTBM: Set scroll region top={}, bottom={}", top, bottom);
                self.state.lock().set_scroll_region(top, bottom);
            }
            's' => {
                // SCP (Save Cursor Position) - 也可能是 DECSC
                tracing::trace!("SCP: Save cursor position");
                self.state.lock().save_cursor();
            }
            'u' => {
                // RCP (Restore Cursor Position) - 也可能是 DECRC
                tracing::trace!("RCP: Restore cursor position");
                self.state.lock().restore_cursor();
            }
            _ => {
                tracing::debug!("Unhandled CSI: {} params={:?}", action, params);
            }
        }

        // 标记脏而不是立即发送 Wakeup
        self.mark_dirty();
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        if ignore {
            return;
        }

        match byte {
            b'D' => {
                // IND (Index) - 向下移动一行（如在底部则滚动）
                // 使用 new_line 确保重绘检测正常工作
                self.state.lock().new_line(false);
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
                tracing::debug!(
                    "Unhandled ESC: 0x{:02x} intermediates={:?}",
                    byte,
                    intermediates
                );
            }
        }

        // 标记脏而不是立即发送 Wakeup
        self.mark_dirty();
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
            #[cfg(feature = "shell-integration")]
            "531" => {
                // zTerm 自定义 JSON 元数据协议
                self.handle_osc_531(params);
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
    /// 解析 OSC 参数中的 key=value 格式
    fn parse_osc_params(params_str: &str) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        for part in params_str.split(';') {
            if let Some((key, value)) = part.split_once('=') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        map
    }

    fn handle_osc_133(&mut self, params: &[&[u8]]) {
        use crate::event::ShellIntegrationEvent;
        use crate::shell_integration::OscSequence;

        if params.len() < 2 {
            return;
        }

        let subcommand = String::from_utf8_lossy(params[1]);
        let cmd_char = match subcommand.chars().next() {
            Some(c) => c,
            None => return,
        };

        // 解析参数（subcommand 可能是 "A" 或 "A;aid=xxx;json=..."）
        let params_str = if subcommand.len() > 1 {
            &subcommand[1..]
        } else {
            ""
        };
        let parsed_params = Self::parse_osc_params(params_str);
        let aid = parsed_params.get("aid").cloned();

        // 构建 OscSequence 并发出 RawOscSequence 事件
        let osc_sequence = match cmd_char {
            'A' => {
                // 提示符开始 - 可能包含 JSON 元数据
                let json = parsed_params.get("json").and_then(|s| {
                    // 反转义并解析 JSON
                    let unescaped = Self::unescape_osc_value(s);
                    serde_json::from_str(&unescaped).ok()
                });
                Some(OscSequence::PromptStart { aid, json })
            }
            'B' => Some(OscSequence::CommandStart { aid }),
            'C' => Some(OscSequence::CommandExecuting { aid }),
            'D' => {
                // 命令结束 - 解析退出码
                let exit_code = if let Some(code_str) = parsed_params.get("exit_code") {
                    code_str.parse::<i32>().ok()
                } else if params_str.starts_with(';') {
                    // 旧格式: ;0 或 ;0;aid=xxx
                    let remaining = &params_str[1..];
                    if let Some((first, _)) = remaining.split_once(';') {
                        first.trim().parse::<i32>().ok()
                    } else {
                        remaining.trim().parse::<i32>().ok()
                    }
                } else {
                    None
                };

                let json = parsed_params.get("json").and_then(|s| {
                    let unescaped = Self::unescape_osc_value(s);
                    serde_json::from_str(&unescaped).ok()
                });

                Some(OscSequence::CommandFinished {
                    exit_code,
                    aid,
                    json,
                })
            }
            _ => None,
        };

        if let Some(seq) = osc_sequence {
            tracing::debug!("OSC 133 parsed: {:?}", seq);
            self.event_listener
                .on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::RawOscSequence(seq),
                ));
        }
    }

    fn handle_osc_633(&mut self, params: &[&[u8]]) {
        use crate::event::ShellIntegrationEvent;
        use crate::shell_integration::OscSequence;

        if params.len() < 2 {
            return;
        }

        let subcommand = String::from_utf8_lossy(params[1]);

        let osc_sequence = if let Some(cwd) = subcommand.strip_prefix("P;Cwd=") {
            Some(OscSequence::WorkingDirectory(cwd.to_string()))
        } else if let Some(cmd_text) = subcommand.strip_prefix("E;") {
            Some(OscSequence::CommandText(cmd_text.to_string()))
        } else {
            None
        };

        if let Some(seq) = osc_sequence {
            tracing::debug!("OSC 633 parsed: {:?}", seq);
            self.event_listener
                .on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::RawOscSequence(seq),
                ));
        }
    }

    /// 处理 OSC 531 - zTerm 自定义 JSON 元数据协议
    fn handle_osc_531(&mut self, params: &[&[u8]]) {
        use crate::event::ShellIntegrationEvent;
        use crate::shell_integration::{JsonDataType, OscSequence};

        if params.len() < 2 {
            return;
        }

        // 获取 JSON 数据（可能被转义）
        let escaped_json = String::from_utf8_lossy(params[1]);
        let json_str = Self::unescape_osc_value(&escaped_json);

        // 解析 JSON
        match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(payload) => {
                // 根据 type 字段确定数据类型
                let data_type = payload
                    .get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| match t {
                        "prompt_start" | "shell_started" | "directory_changed" => {
                            JsonDataType::BlockMeta
                        }
                        "command_start" | "command_end" => JsonDataType::CommandMeta,
                        _ => JsonDataType::Custom,
                    })
                    .unwrap_or(JsonDataType::Custom);

                tracing::info!("OSC 531 parsed: type={:?}, payload={}", data_type, payload);

                let seq = OscSequence::JsonData { data_type, payload };
                self.event_listener
                    .on_event(TerminalEvent::ShellIntegration(
                        ShellIntegrationEvent::RawOscSequence(seq),
                    ));
            }
            Err(e) => {
                tracing::warn!("Failed to parse OSC 531 JSON: {} (data: {})", e, json_str);
            }
        }
    }

    /// 反转义 OSC 值（逆向 __ZTerm-Escape-Value）
    fn unescape_osc_value(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.peek() {
                    Some('x') => {
                        chars.next(); // 消费 'x'
                        let hex: String = chars.by_ref().take(2).collect();
                        if hex.len() == 2 {
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                result.push(byte as char);
                                continue;
                            }
                        }
                        result.push('\\');
                        result.push('x');
                        result.push_str(&hex);
                    }
                    Some('n') => {
                        chars.next();
                        result.push('\n');
                    }
                    Some('\\') => {
                        chars.next();
                        result.push('\\');
                    }
                    _ => result.push(ch),
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}
