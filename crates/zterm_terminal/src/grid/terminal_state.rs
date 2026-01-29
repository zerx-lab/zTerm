//! TerminalState 结构
//!
//! 完整的终端状态，包含 Screen、Cursor 和渲染属性

use serde::{Deserialize, Serialize};

use super::cursor::{Cursor, CursorShape};
use super::screen::Screen;
use super::{Cell, CellAttributes, PhysRowIndex, VisibleRowIndex};
use std::ops::Range;

/// 滚动区域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrollRegion {
    /// 起始行（包含，0-based）
    pub top: usize,
    /// 结束行（包含，0-based）
    pub bottom: usize,
}

impl ScrollRegion {
    pub fn new(top: usize, bottom: usize) -> Self {
        Self { top, bottom }
    }

    /// 是否包含指定行
    #[inline]
    pub fn contains(&self, row: usize) -> bool {
        row >= self.top && row <= self.bottom
    }

    /// 获取区域高度
    #[inline]
    pub fn height(&self) -> usize {
        self.bottom.saturating_sub(self.top) + 1
    }
}

/// 终端模式标志
bitflags::bitflags! {
    /// 终端模式
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TerminalModes: u32 {
        /// 光标键应用模式（DECCKM）
        const APPLICATION_CURSOR = 1 << 0;
        /// 自动换行模式（DECAWM）
        const AUTO_WRAP = 1 << 1;
        /// 插入模式（IRM）
        const INSERT = 1 << 2;
        /// 换行/新行模式（LNM）
        const LINE_FEED_NEW_LINE = 1 << 3;
        /// 括号粘贴模式
        const BRACKETED_PASTE = 1 << 4;
        /// 焦点报告模式
        const FOCUS_REPORTING = 1 << 5;
        /// 鼠标报告模式
        const MOUSE_REPORTING = 1 << 6;
        /// 替代屏幕缓冲区
        const ALTERNATE_SCREEN = 1 << 7;
    }
}

impl Default for TerminalModes {
    fn default() -> Self {
        Self::AUTO_WRAP
    }
}

/// 终端状态
///
/// 包含完整的终端状态：Screen、Cursor、模式等
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalState {
    /// 主屏幕
    screen: Screen,
    /// 替代屏幕（可选）
    alt_screen: Option<Screen>,
    /// 光标
    cursor: Cursor,
    /// 保存的光标位置（用于 DECSC/DECRC）
    saved_cursor: Option<Cursor>,
    /// 当前属性（用于新输入）
    attrs: CellAttributes,
    /// 滚动区域（上下边界，使用 Range<usize>，end 是不包含的）
    /// 参考 WezTerm: top_and_bottom_margins
    top_and_bottom_margins: Range<usize>,
    /// 左右边界
    /// 参考 WezTerm: left_and_right_margins
    left_and_right_margins: Range<usize>,
    /// 终端模式
    modes: TerminalModes,
    /// 当前标题
    title: String,
    /// 下一个字符是否触发自动换行
    /// 参考 WezTerm 的 wrap_next 机制
    /// 当光标在行尾且 AUTO_WRAP 启用时，下一个打印字符会先换行
    #[serde(skip)]
    wrap_next: bool,
    /// 用于检测 PSReadLine 重绘模式
    /// 记录所有因 scroll_up 而"残留"的物理行索引
    /// 当检测到光标"回退"时，清除这些行
    #[serde(skip)]
    scroll_source_rows: Vec<usize>, // 物理行索引列表
    /// 记录输入开始的视口行（用于检测回退）
    #[serde(skip)]
    input_start_visible_row: Option<usize>,
}

impl TerminalState {
    /// 创建新的终端状态
    pub fn new(rows: usize, cols: usize, max_scrollback: usize) -> Self {
        Self {
            screen: Screen::new(rows, cols, max_scrollback),
            alt_screen: None,
            cursor: Cursor::default(),
            saved_cursor: None,
            attrs: CellAttributes::default(),
            top_and_bottom_margins: 0..rows,
            left_and_right_margins: 0..cols,
            modes: TerminalModes::default(),
            title: String::new(),
            wrap_next: false,
            scroll_source_rows: Vec::new(),
            input_start_visible_row: None,
        }
    }

    /// 获取当前屏幕
    #[inline]
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// 获取可变当前屏幕
    #[inline]
    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    /// 获取光标
    #[inline]
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// 获取可变光标
    #[inline]
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    /// 获取当前属性
    #[inline]
    pub fn attrs(&self) -> &CellAttributes {
        &self.attrs
    }

    /// 获取可变当前属性
    #[inline]
    pub fn attrs_mut(&mut self) -> &mut CellAttributes {
        &mut self.attrs
    }

    /// 设置属性
    #[inline]
    pub fn set_attrs(&mut self, attrs: CellAttributes) {
        self.attrs = attrs.clone();
        self.cursor.attrs = attrs;
    }

    /// 获取视口尺寸
    #[inline]
    pub fn size(&self) -> (usize, usize) {
        (self.screen.cols(), self.screen.rows())
    }

    /// 获取行数
    #[inline]
    pub fn rows(&self) -> usize {
        self.screen.rows()
    }

    /// 获取列数
    #[inline]
    pub fn cols(&self) -> usize {
        self.screen.cols()
    }

    // ========== 光标操作 ==========

    /// 移动光标到指定位置（视口坐标）
    ///
    /// 这个方法还会检测 PSReadLine 风格的重绘模式：
    /// 当光标移动到之前因 scroll_up 而"移位"的行时，
    /// 清除旧位置的重复内容
    pub fn set_cursor_pos(&mut self, x: usize, y: usize) {
        let max_cols = self.cols();
        let max_rows = self.rows();

        let new_x = x.min(max_cols.saturating_sub(1));
        let new_y = y.min(max_rows.saturating_sub(1));

        // 检测重绘模式：如果光标移动到行首，可能是 PSReadLine 重绘
        if x == 0 {
            self.handle_redraw_detection(new_y);
        }

        self.cursor.x = new_x;
        self.cursor.y = new_y;

        // 移动光标时重置 wrap_next 状态
        // 参考 WezTerm: 显式移动光标会取消延迟换行
        self.wrap_next = false;
    }

    /// 处理重绘检测
    ///
    /// 当检测到 PSReadLine 风格的重绘（光标移回之前写入的行）时，
    /// 清除可见区域中所有"残留"的重复内容
    fn handle_redraw_detection(&mut self, target_vis_row: usize) {
        // 检查是否是重绘场景：
        // 1. 有记录的输入开始位置
        // 2. 目标行与输入开始位置相同
        // 3. 有 scroll_up 产生的"残留"行
        if let Some(input_start) = self.input_start_visible_row {
            if target_vis_row == input_start && !self.scroll_source_rows.is_empty() {
                let scrollback = self.screen.scrollback_lines();
                let visible_start = scrollback;
                let visible_end = scrollback + self.rows();

                tracing::debug!(
                    "handle_redraw_detection: detected redraw to vis_row={}, clearing {} source rows",
                    target_vis_row,
                    self.scroll_source_rows.len()
                );

                // 清除所有在可见区域内的"残留"行
                for &source_phys in &self.scroll_source_rows {
                    if source_phys >= visible_start && source_phys < visible_end {
                        tracing::debug!(
                            "  clearing source_phys={} (now at vis_row={})",
                            source_phys,
                            source_phys - visible_start
                        );

                        let phys_idx = PhysRowIndex::new(source_phys);
                        if let Some(line) = self.screen.get_line_mut(phys_idx) {
                            line.clear();
                        }
                    }
                }

                // 清除记录，准备下一轮输入
                self.scroll_source_rows.clear();
                self.input_start_visible_row = None;
            }
        }
    }

    /// 保存光标位置
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor.clone());
    }

    /// 恢复光标位置
    pub fn restore_cursor(&mut self) {
        if let Some(saved) = self.saved_cursor.clone() {
            self.cursor = saved;
        }
    }

    /// 设置光标形状
    #[inline]
    pub fn set_cursor_shape(&mut self, shape: CursorShape) {
        self.cursor.set_shape(shape);
    }

    /// 设置光标可见性
    #[inline]
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor.set_visible(visible);
    }

    // ========== 屏幕操作 ==========

    /// 切换到替代屏幕
    pub fn switch_to_alternate_screen(&mut self) {
        if self.alt_screen.is_none() {
            self.alt_screen = Some(Screen::new(
                self.rows(),
                self.cols(),
                self.screen.max_scrollback(),
            ));
        }

        std::mem::swap(&mut self.screen, self.alt_screen.as_mut().unwrap());
        self.modes.insert(TerminalModes::ALTERNATE_SCREEN);
    }

    /// 切换回主屏幕
    pub fn switch_to_main_screen(&mut self) {
        if let Some(ref mut alt) = self.alt_screen {
            std::mem::swap(&mut self.screen, alt);
            self.modes.remove(TerminalModes::ALTERNATE_SCREEN);
        }
    }

    /// 是否在替代屏幕
    #[inline]
    pub fn is_alternate_screen(&self) -> bool {
        self.modes.contains(TerminalModes::ALTERNATE_SCREEN)
    }

    /// 清空屏幕
    pub fn clear_screen(&mut self) {
        self.screen.clear();
        self.cursor.home();
    }

    /// 调整大小
    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        self.screen.resize(new_rows, new_cols);
        if let Some(ref mut alt) = self.alt_screen {
            alt.resize(new_rows, new_cols);
        }
        self.cursor.clamp(new_cols, new_rows);
        // 重置 margins 为新尺寸
        self.top_and_bottom_margins = 0..new_rows;
        self.left_and_right_margins = 0..new_cols;
        // 重置 wrap_next 状态
        self.wrap_next = false;
    }

    // ========== 滚动区域 ==========

    /// 设置滚动区域
    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        let max_rows = self.rows();
        // 使用 Range，end 是不包含的，所以 bottom + 1
        if top <= bottom && bottom < max_rows {
            self.top_and_bottom_margins = top..(bottom + 1);
        }
    }

    /// 清除滚动区域
    pub fn clear_scroll_region(&mut self) {
        let rows = self.rows();
        self.top_and_bottom_margins = 0..rows;
    }

    /// 获取滚动区域
    #[inline]
    pub fn scroll_region(&self) -> Option<ScrollRegion> {
        // 兼容旧接口：如果是全屏范围则返回 None
        if self.top_and_bottom_margins.start == 0 
            && self.top_and_bottom_margins.end == self.rows() 
        {
            None
        } else {
            Some(ScrollRegion::new(
                self.top_and_bottom_margins.start,
                self.top_and_bottom_margins.end.saturating_sub(1),
            ))
        }
    }

    /// 获取有效滚动区域（如果未设置则返回全屏）
    pub fn effective_scroll_region(&self) -> ScrollRegion {
        ScrollRegion::new(
            self.top_and_bottom_margins.start,
            self.top_and_bottom_margins.end.saturating_sub(1),
        )
    }

    /// 获取上下边界 (Range 形式，end 不包含)
    #[inline]
    pub fn top_and_bottom_margins(&self) -> &Range<usize> {
        &self.top_and_bottom_margins
    }

    /// 获取左右边界 (Range 形式，end 不包含)
    #[inline]
    pub fn left_and_right_margins(&self) -> &Range<usize> {
        &self.left_and_right_margins
    }

    /// 获取 wrap_next 状态
    #[inline]
    pub fn wrap_next(&self) -> bool {
        self.wrap_next
    }

    /// 设置 wrap_next 状态
    #[inline]
    pub fn set_wrap_next(&mut self, value: bool) {
        self.wrap_next = value;
    }

    // ========== 模式管理 ==========

    /// 设置模式
    #[inline]
    pub fn set_mode(&mut self, mode: TerminalModes, enabled: bool) {
        if enabled {
            self.modes.insert(mode);
        } else {
            self.modes.remove(mode);
        }
    }

    /// 检查模式是否启用
    #[inline]
    pub fn is_mode_enabled(&self, mode: TerminalModes) -> bool {
        self.modes.contains(mode)
    }

    /// 获取所有模式
    #[inline]
    pub fn modes(&self) -> TerminalModes {
        self.modes
    }

    // ========== 标题管理 ==========

    /// 设置终端标题
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// 获取终端标题
    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    // ========== 行操作 ==========

    /// 获取光标所在行
    pub fn current_line(&self) -> Option<&super::Line> {
        let vis_idx = VisibleRowIndex::new(self.cursor.y as isize);
        self.screen.get_visible_line(vis_idx)
    }

    /// 获取可变光标所在行
    pub fn current_line_mut(&mut self) -> Option<&mut super::Line> {
        let vis_idx = VisibleRowIndex::new(self.cursor.y as isize);
        self.screen.get_visible_line_mut(vis_idx)
    }

    /// 在光标位置写入单元格
    pub fn write_cell(&mut self, cell: Cell) {
        let cell_width = cell.width();
        let width = self.left_and_right_margins.end;

        // 处理 wrap_next 状态（参考 WezTerm）
        // 如果上一次打印到达了行尾，这次打印前需要先换行
        if self.wrap_next {
            // 标记当前行为 wrapped
            let y = self.cursor.y;
            let vis_idx = VisibleRowIndex::new(y as isize);
            if let Some(phys) = self.screen.visible_to_phys(vis_idx) {
                if let Some(line) = self.screen.get_line_mut(phys) {
                    line.set_wrapped(true);
                }
            }
            // 执行换行
            self.new_line(true);
            self.wrap_next = false;
        }

        let x = self.cursor.x;
        let y = self.cursor.y;
        let vis_idx = VisibleRowIndex::new(y as isize);

        // 记录输入开始位置（用于检测重绘模式）
        if x == 0 && self.input_start_visible_row.is_none() {
            self.input_start_visible_row = Some(y);
        }

        if let Some(phys) = self.screen.visible_to_phys(vis_idx) {
            if let Some(line) = self.screen.get_line_mut(phys) {
                if x < line.len() {
                    line.set_cell(x, cell);
                } else {
                    // 扩展行
                    line.resize(x + 1, Cell::blank());
                    line.set_cell(x, cell);
                }
            }
        }

        // 检查是否到达行尾（wrappable 条件）
        let wrappable = x + cell_width as usize >= width;

        if !wrappable {
            // 未到达行尾，正常移动光标
            self.cursor.x += cell_width as usize;
            self.wrap_next = false;
        } else {
            // 到达行尾，设置 wrap_next 而不是立即换行
            // 这是 VT100 的标准行为
            self.wrap_next = self.modes.contains(TerminalModes::AUTO_WRAP);
        }
    }

    /// 执行换行操作（参考 WezTerm 的 new_line）
    ///
    /// - move_to_first_column: 是否移动到第一列
    pub fn new_line(&mut self, move_to_first_column: bool) {
        let x = if move_to_first_column {
            self.left_and_right_margins.start
        } else {
            self.cursor.x
        };
        let y = self.cursor.y;

        // 检查是否在滚动区域底部
        let new_y = if y == self.top_and_bottom_margins.end.saturating_sub(1) {
            // 在底部，需要滚动
            self.scroll_up(1);
            y // 光标行号不变
        } else {
            y + 1
        };

        self.cursor.x = x;
        self.cursor.y = new_y.min(self.rows().saturating_sub(1));
        self.wrap_next = false;
    }

    /// 在滚动区域内向上滚动（参考 WezTerm 的 scroll_up）
    pub fn scroll_up(&mut self, num_rows: usize) {
        // 记录滚动前当前输入行的物理索引（用于重绘检测）
        // 只在有输入开始位置记录时才记录
        if let Some(input_start) = self.input_start_visible_row {
            let scrollback = self.screen.scrollback_lines();
            // 记录即将被滚动的行（从 input_start 到当前光标行）
            for vis_row in input_start..=self.cursor.y {
                let phys = scrollback + vis_row;
                if !self.scroll_source_rows.contains(&phys) {
                    self.scroll_source_rows.push(phys);
                }
            }
        }

        let top = self.top_and_bottom_margins.start;
        let bottom = self.top_and_bottom_margins.end;
        let left = self.left_and_right_margins.start;
        let right = self.left_and_right_margins.end;

        // 如果是全屏滚动（没有左右边界限制），使用简单的滚动
        if left == 0 && right == self.cols() {
            self.screen.scroll_up_within_margins(top, bottom, num_rows);
        } else {
            // 有左右边界，需要更复杂的处理（暂时简化实现）
            self.screen.scroll_up_within_margins(top, bottom, num_rows);
        }
    }

    /// 在滚动区域内向下滚动
    pub fn scroll_down(&mut self, num_rows: usize) {
        let top = self.top_and_bottom_margins.start;
        let bottom = self.top_and_bottom_margins.end;
        self.screen.scroll_down_within_margins(top, bottom, num_rows);
    }

    /// 滚动（在滚动区域内）
    pub fn scroll_up_region(&mut self, lines: usize) {
        self.scroll_up(lines);
    }

    // ========== 擦除操作 ==========

    /// 擦除显示 (ED - Erase Display)
    ///
    /// - Mode 0: 从光标到屏幕末尾
    /// - Mode 1: 从屏幕开始到光标
    /// - Mode 2: 整个屏幕
    /// - Mode 3: 整个屏幕和 scrollback
    pub fn erase_display(&mut self, mode: u16) {
        let cursor_x = self.cursor.x;
        let cursor_y = self.cursor.y;
        let cols = self.cols();
        let rows = self.rows();

        match mode {
            0 => {
                // 从光标到屏幕末尾
                // 1. 擦除当前行光标位置到行末
                self.erase_line_from(cursor_y, cursor_x, cols);
                // 2. 擦除光标下方的所有行
                for row in (cursor_y + 1)..rows {
                    self.erase_entire_line(row);
                }
            }
            1 => {
                // 从屏幕开始到光标
                // 1. 擦除光标上方的所有行
                for row in 0..cursor_y {
                    self.erase_entire_line(row);
                }
                // 2. 擦除当前行从开始到光标
                self.erase_line_to(cursor_y, cursor_x);
            }
            2 => {
                // 整个屏幕（不影响 scrollback）
                for row in 0..rows {
                    self.erase_entire_line(row);
                }
            }
            3 => {
                // 整个屏幕和 scrollback
                self.screen.clear();
                self.screen.clear_scrollback();
            }
            _ => {}
        }
    }

    /// 擦除行 (EL - Erase Line)
    ///
    /// - Mode 0: 从光标到行末
    /// - Mode 1: 从行首到光标
    /// - Mode 2: 整行
    pub fn erase_line(&mut self, mode: u16) {
        let cursor_x = self.cursor.x;
        let cursor_y = self.cursor.y;
        let cols = self.cols();

        match mode {
            0 => {
                // 从光标到行末
                self.erase_line_from(cursor_y, cursor_x, cols);
            }
            1 => {
                // 从行首到光标（包含光标位置）
                self.erase_line_to(cursor_y, cursor_x);
            }
            2 => {
                // 整行
                self.erase_entire_line(cursor_y);
            }
            _ => {}
        }
    }

    /// 内部方法：擦除行从指定位置到末尾
    fn erase_line_from(&mut self, row: usize, start_col: usize, end_col: usize) {
        let vis_idx = VisibleRowIndex::new(row as isize);
        if let Some(phys) = self.screen.visible_to_phys(vis_idx) {
            if let Some(line) = self.screen.get_line_mut(phys) {
                let blank = Cell::blank();
                for col in start_col..end_col {
                    if col < line.len() {
                        line.set_cell(col, blank.clone());
                    }
                }
            }
        }
    }

    /// 内部方法：擦除行从开始到指定位置（包含）
    fn erase_line_to(&mut self, row: usize, end_col: usize) {
        let vis_idx = VisibleRowIndex::new(row as isize);
        if let Some(phys) = self.screen.visible_to_phys(vis_idx) {
            if let Some(line) = self.screen.get_line_mut(phys) {
                let blank = Cell::blank();
                for col in 0..=end_col {
                    if col < line.len() {
                        line.set_cell(col, blank.clone());
                    }
                }
            }
        }
    }

    /// 内部方法：擦除整行
    fn erase_entire_line(&mut self, row: usize) {
        let cols = self.cols();
        let vis_idx = VisibleRowIndex::new(row as isize);
        if let Some(phys) = self.screen.visible_to_phys(vis_idx) {
            if let Some(line) = self.screen.get_line_mut(phys) {
                line.clear();
                // 重新调整为正确宽度
                line.resize(cols, Cell::blank());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_state_new() {
        let state = TerminalState::new(24, 80, 1000);
        assert_eq!(state.rows(), 24);
        assert_eq!(state.cols(), 80);
        assert_eq!(state.cursor().position(), (0, 0));
    }

    #[test]
    fn test_set_cursor_pos() {
        let mut state = TerminalState::new(24, 80, 1000);
        state.set_cursor_pos(10, 5);
        assert_eq!(state.cursor().position(), (10, 5));

        state.set_cursor_pos(100, 100);
        assert_eq!(state.cursor().position(), (79, 23)); // 限制在范围内
    }

    #[test]
    fn test_save_restore_cursor() {
        let mut state = TerminalState::new(24, 80, 1000);
        state.set_cursor_pos(10, 5);
        state.save_cursor();

        state.set_cursor_pos(20, 10);
        assert_eq!(state.cursor().position(), (20, 10));

        state.restore_cursor();
        assert_eq!(state.cursor().position(), (10, 5));
    }

    #[test]
    fn test_alternate_screen() {
        let mut state = TerminalState::new(24, 80, 1000);
        assert!(!state.is_alternate_screen());

        state.switch_to_alternate_screen();
        assert!(state.is_alternate_screen());

        state.switch_to_main_screen();
        assert!(!state.is_alternate_screen());
    }

    #[test]
    fn test_scroll_region() {
        let mut state = TerminalState::new(24, 80, 1000);
        assert!(state.scroll_region().is_none());

        state.set_scroll_region(5, 20);
        let region = state.scroll_region().unwrap();
        assert_eq!(region.top, 5);
        assert_eq!(region.bottom, 20);
        assert_eq!(region.height(), 16);

        state.clear_scroll_region();
        assert!(state.scroll_region().is_none());
    }

    #[test]
    fn test_modes() {
        let mut state = TerminalState::new(24, 80, 1000);

        assert!(state.is_mode_enabled(TerminalModes::AUTO_WRAP));

        state.set_mode(TerminalModes::INSERT, true);
        assert!(state.is_mode_enabled(TerminalModes::INSERT));

        state.set_mode(TerminalModes::INSERT, false);
        assert!(!state.is_mode_enabled(TerminalModes::INSERT));
    }

    #[test]
    fn test_title() {
        let mut state = TerminalState::new(24, 80, 1000);
        assert_eq!(state.title(), "");

        state.set_title("Test Terminal".to_string());
        assert_eq!(state.title(), "Test Terminal");
    }

    #[test]
    fn test_resize() {
        let mut state = TerminalState::new(24, 80, 1000);

        state.resize(30, 100);
        assert_eq!(state.rows(), 30);
        assert_eq!(state.cols(), 100);
    }

    #[test]
    fn test_clear_screen() {
        let mut state = TerminalState::new(24, 80, 1000);
        state.set_cursor_pos(10, 5);

        state.clear_screen();
        assert_eq!(state.cursor().position(), (0, 0));
    }

    #[test]
    fn test_cursor_shape() {
        let mut state = TerminalState::new(24, 80, 1000);

        state.set_cursor_shape(CursorShape::Bar);
        assert_eq!(state.cursor().shape, CursorShape::Bar);
    }

    #[test]
    fn test_cursor_visibility() {
        let mut state = TerminalState::new(24, 80, 1000);
        assert!(state.cursor().visible);

        state.set_cursor_visible(false);
        assert!(!state.cursor().visible);
    }

    #[test]
    fn test_attrs() {
        let mut state = TerminalState::new(24, 80, 1000);
        let mut attrs = CellAttributes::default();
        attrs.set_bold(true);

        state.set_attrs(attrs.clone());
        assert_eq!(state.attrs(), &attrs);
        assert!(state.attrs().is_bold());
    }

    #[test]
    fn test_write_cell() {
        let mut state = TerminalState::new(24, 80, 1000);
        let cell = Cell::new("a");

        state.write_cell(cell);
        assert_eq!(state.cursor().x, 1);
    }

    #[test]
    fn test_write_cell_wrap() {
        let mut state = TerminalState::new(24, 80, 1000);
        state.cursor.x = 79;

        // 写入第一个字符 - 设置 wrap_next 但不立即换行 (VT100 标准行为)
        let cell = Cell::new("a");
        state.write_cell(cell);

        // 光标应该保持在最后一列，wrap_next 被设置
        assert_eq!(state.cursor().x, 79);
        assert_eq!(state.cursor().y, 0);
        assert!(state.wrap_next, "wrap_next should be set");

        // 写入第二个字符 - 触发换行
        let cell2 = Cell::new("b");
        state.write_cell(cell2);

        // 现在光标应该在第二行第一列之后
        assert_eq!(state.cursor().x, 1);
        assert_eq!(state.cursor().y, 1);
    }

    #[test]
    fn test_scroll_region_contains() {
        let region = ScrollRegion::new(5, 20);

        assert!(!region.contains(4));
        assert!(region.contains(5));
        assert!(region.contains(15));
        assert!(region.contains(20));
        assert!(!region.contains(21));
    }

    #[test]
    fn test_effective_scroll_region() {
        let mut state = TerminalState::new(24, 80, 1000);

        let region = state.effective_scroll_region();
        assert_eq!(region.top, 0);
        assert_eq!(region.bottom, 23);

        state.set_scroll_region(5, 15);
        let region = state.effective_scroll_region();
        assert_eq!(region.top, 5);
        assert_eq!(region.bottom, 15);
    }

    // ========== ED/EL 测试 ==========

    #[test]
    fn test_erase_line_mode_0() {
        // Mode 0: 从光标到行末
        let mut state = TerminalState::new(5, 10, 100);

        // 写入一行 "abcdefghij"
        for c in "abcdefghij".chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }
        state.set_cursor_pos(0, 0); // 回到行首

        // 移动光标到位置 5
        state.set_cursor_pos(5, 0);

        // 擦除从光标到行末
        state.erase_line(0);

        // 获取行内容
        let line = state.current_line().unwrap();
        let text = line.text();

        // 前 5 个字符应该保留 "abcde"
        assert!(text.starts_with("abcde"));
    }

    #[test]
    fn test_erase_line_mode_1() {
        // Mode 1: 从行首到光标（包含）
        let mut state = TerminalState::new(5, 10, 100);

        // 写入一行
        for c in "abcdefghij".chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }
        state.set_cursor_pos(0, 0);

        // 移动光标到位置 5
        state.set_cursor_pos(5, 0);

        // 擦除从行首到光标
        state.erase_line(1);

        // 获取行内容
        let line = state.current_line().unwrap();
        let text = line.text();

        // 前 6 个字符应该被擦除，后面保留 "ghij"
        assert!(text.ends_with("ghij"));
    }

    #[test]
    fn test_erase_line_mode_2() {
        // Mode 2: 整行
        let mut state = TerminalState::new(5, 10, 100);

        // 写入一行
        for c in "abcdefghij".chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }
        state.set_cursor_pos(0, 0);

        // 擦除整行
        state.erase_line(2);

        // 获取行内容
        let line = state.current_line().unwrap();
        let text = line.text();

        // 整行应该是空的
        assert!(text.trim().is_empty());
    }

    #[test]
    fn test_erase_display_mode_0() {
        // Mode 0: 从光标到屏幕末尾
        let mut state = TerminalState::new(3, 10, 100);

        // 写入 3 行
        for i in 0..3 {
            for c in "0123456789".chars() {
                state.write_cell(Cell::new(&c.to_string()));
            }
            if i < 2 {
                // 手动换行（通过移动光标）
                state.set_cursor_pos(0, i + 1);
            }
        }

        // 移动光标到第 1 行，第 5 列
        state.set_cursor_pos(5, 1);

        // 擦除从光标到屏幕末尾
        state.erase_display(0);

        // 第 0 行应该完整保留
        let line0 = state.screen().get_line_text(0).unwrap();
        assert_eq!(line0, "0123456789");

        // 第 1 行前 5 个字符应该保留
        let line1 = state.screen().get_line_text(1).unwrap();
        assert!(line1.starts_with("01234"));

        // 第 2 行应该被清空
        let line2 = state.screen().get_line_text(2).unwrap();
        assert!(line2.trim().is_empty());
    }

    #[test]
    fn test_erase_display_mode_2() {
        // Mode 2: 整个屏幕
        let mut state = TerminalState::new(3, 10, 100);

        // 写入一些内容
        for c in "abcdefghij".chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }

        // 擦除整个屏幕
        state.erase_display(2);

        // 所有行都应该是空的
        for i in 0..3 {
            let line = state.screen().get_line_text(i).unwrap();
            assert!(line.trim().is_empty(), "Line {} should be empty", i);
        }
    }

    #[test]
    fn test_auto_wrap_content_not_duplicated() {
        // 测试自动换行时内容不会被复制
        // 5行10列的终端
        let mut state = TerminalState::new(5, 10, 100);

        // 输入 15 个字符（应该占用 1.5 行）
        let input = "abcdefghij12345";
        for c in input.chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }

        // 检查光标位置
        assert_eq!(state.cursor().x, 5, "cursor.x should be 5");
        assert_eq!(state.cursor().y, 1, "cursor.y should be 1 (second row)");

        // 检查第一行内容
        let line0 = state.screen().get_line_text(0).unwrap();
        assert_eq!(
            line0.trim_end(),
            "abcdefghij",
            "First line should contain first 10 chars"
        );

        // 检查第二行内容
        let line1 = state.screen().get_line_text(1).unwrap();
        assert_eq!(
            line1.trim_end(),
            "12345",
            "Second line should contain remaining 5 chars"
        );

        // 确保第一行没有被复制到第二行
        assert!(
            !line1.contains("abcdefghij"),
            "Second line should NOT contain first line's content"
        );
    }

    #[test]
    fn test_auto_wrap_triggers_scroll_at_bottom() {
        // 测试在屏幕底部时自动换行触发滚动
        // 3行5列的终端
        let mut state = TerminalState::new(3, 5, 100);

        // 填满所有 3 行（每行 5 个字符，共 15 个）
        let input = "aaaaabbbbbccccc";
        for c in input.chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }

        // 由于延迟换行 (wrap_next) 机制：
        // - 每行第 5 个字符写入后，光标保持在 x=4，设置 wrap_next=true
        // - 下一个字符触发换行
        // 所以 15 个字符后，光标在 x=4, y=2，wrap_next=true
        assert_eq!(state.cursor().x, 4, "cursor.x should be at last column");
        assert_eq!(state.cursor().y, 2, "cursor.y should be at last row");
        assert!(state.wrap_next, "wrap_next should be set");

        // 输入一个字符，触发换行和滚动
        state.write_cell(Cell::new("d"));

        // 检查滚动后的行内容
        // scrollback 应该有 1 行
        assert_eq!(state.screen().scrollback_lines(), 1);

        // 第一行现在应该是 "bbbbb"（原来的第二行）
        let line0 = state.screen().get_line_text(0).unwrap();
        assert_eq!(line0.trim_end(), "bbbbb", "First visible line after scroll");

        // 第二行应该是 "ccccc"（原来的第三行）
        let line1 = state.screen().get_line_text(1).unwrap();
        assert_eq!(
            line1.trim_end(),
            "ccccc",
            "Second visible line after scroll"
        );

        // 第三行应该是 "d"
        let line2 = state.screen().get_line_text(2).unwrap();
        assert_eq!(
            line2.trim_end(),
            "d",
            "Third visible line (new) after scroll"
        );
    }

    #[test]
    fn test_get_all_viewport_content() {
        // 测试获取视口内容的正确性
        let mut state = TerminalState::new(3, 10, 100);

        // 写入 3 行不同的内容
        state.write_cell(Cell::new("A"));
        state.set_cursor_pos(0, 1);
        state.write_cell(Cell::new("B"));
        state.set_cursor_pos(0, 2);
        state.write_cell(Cell::new("C"));

        // 获取每行文本
        let line0 = state.screen().get_line_text(0).unwrap();
        let line1 = state.screen().get_line_text(1).unwrap();
        let line2 = state.screen().get_line_text(2).unwrap();

        // 每行应该有不同的内容
        assert!(line0.starts_with("A"), "Line 0 should start with A");
        assert!(line1.starts_with("B"), "Line 1 should start with B");
        assert!(line2.starts_with("C"), "Line 2 should start with C");
    }

    #[test]
    fn test_continuous_input_wrap_no_duplication() {
        // 模拟连续输入超长内容，验证不会出现内容重复
        let mut state = TerminalState::new(5, 10, 100);

        // 输入 25 个字符（应该占用 2.5 行）
        // 第一行: 0123456789
        // 第二行: ABCDEFGHIJ
        // 第三行: KLMNO
        let chars1 = "0123456789";
        let chars2 = "ABCDEFGHIJ";
        let chars3 = "KLMNO";

        for c in chars1.chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }
        for c in chars2.chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }
        for c in chars3.chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }

        // 验证光标位置
        assert_eq!(state.cursor().x, 5, "cursor.x should be 5");
        assert_eq!(state.cursor().y, 2, "cursor.y should be 2");

        // 验证每行内容
        let line0 = state.screen().get_line_text(0).unwrap();
        let line1 = state.screen().get_line_text(1).unwrap();
        let line2 = state.screen().get_line_text(2).unwrap();

        println!("Line 0: '{}'", line0);
        println!("Line 1: '{}'", line1);
        println!("Line 2: '{}'", line2);

        assert_eq!(line0, chars1, "Line 0 should be first 10 chars");
        assert_eq!(line1, chars2, "Line 1 should be second 10 chars");
        assert_eq!(line2, chars3, "Line 2 should be last 5 chars");

        // 确保行内容不重复
        assert_ne!(line0, line1, "Line 0 and Line 1 should be different");
        assert_ne!(line1, line2, "Line 1 and Line 2 should be different");
        assert_ne!(line0, line2, "Line 0 and Line 2 should be different");
    }

    #[test]
    fn test_screen_lines_after_wrap() {
        // 测试换行后物理行的内容是否正确
        let mut state = TerminalState::new(3, 5, 100);

        // 输入 12 个字符，触发多次换行
        for c in "AAAAABBBBBCC".chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }

        // 检查物理行
        let screen = state.screen();
        for i in 0..screen.total_lines() {
            let phys = crate::grid::PhysRowIndex::new(i);
            if let Some(line) = screen.get_line(phys) {
                let text = line.text();
                println!("Phys line {}: '{}' (len={})", i, text, line.len());
            }
        }

        // 验证没有重复
        let line0 = state.screen().get_line_text(0).unwrap();
        let line1 = state.screen().get_line_text(1).unwrap();
        let line2 = state.screen().get_line_text(2).unwrap();

        assert_eq!(line0, "AAAAA", "Line 0 content");
        assert_eq!(line1, "BBBBB", "Line 1 content");
        assert_eq!(line2, "CC", "Line 2 content");
    }

    #[test]
    fn test_long_input_with_scroll() {
        // 模拟用户截图中的场景：输入超长内容导致多次换行和滚动
        // 5 行 10 列的终端，输入超过 50 个字符
        let mut state = TerminalState::new(5, 10, 100);

        // 输入 60 个字符 (6 行，会导致滚动)
        // Line 0: 0123456789
        // Line 1: ABCDEFGHIJ
        // Line 2: abcdefghij
        // Line 3: !@#$%^&*()
        // Line 4: 9876543210
        // Line 5: ZYXWVUTSRQ (这一行会导致 scroll_up)
        let input = "0123456789ABCDEFGHIJabcdefghij!@#$%^&*()9876543210ZYXWVUTSRQ";

        println!("\n=== 输入 {} 个字符 ===", input.len());
        for (i, c) in input.chars().enumerate() {
            state.write_cell(Cell::new(&c.to_string()));
            if (i + 1) % 10 == 0 {
                println!(
                    "After char {}: cursor=({}, {}), total_lines={}, scrollback={}",
                    i + 1,
                    state.cursor().x,
                    state.cursor().y,
                    state.screen().total_lines(),
                    state.screen().scrollback_lines()
                );
            }
        }

        println!("\n=== 检查物理行 ===");
        let screen = state.screen();
        for i in 0..screen.total_lines() {
            let phys = crate::grid::PhysRowIndex::new(i);
            if let Some(line) = screen.get_line(phys) {
                let text = line.text();
                println!("Phys[{}]: '{}' (len={})", i, text, line.len());
            }
        }

        println!("\n=== 检查可见行 ===");
        for i in 0..5 {
            let text = state.screen().get_line_text(i).unwrap();
            println!("Visible[{}]: '{}'", i, text);
        }

        // 验证：scrollback 应该有 1 行
        // 60 字符 / 10 列 = 6 行内容
        // 由于延迟换行 (wrap_next),最后一个字符写入后光标在第 6 行的 x=9
        // viewport 是 5 行,所以 scrollback = 6 - 5 = 1 行
        assert_eq!(
            state.screen().scrollback_lines(),
            1,
            "Should have 1 scrollback line"
        );

        // 验证：每行内容都不同
        let visible_lines: Vec<String> = (0..5)
            .map(|i| state.screen().get_line_text(i).unwrap())
            .collect();

        // 检查没有重复
        for i in 0..visible_lines.len() {
            for j in (i + 1)..visible_lines.len() {
                assert_ne!(
                    visible_lines[i], visible_lines[j],
                    "Line {} and {} should be different: '{}' vs '{}'",
                    i, j, visible_lines[i], visible_lines[j]
                );
            }
        }
    }

    #[test]
    fn test_psreadline_redraw_behavior() {
        // 模拟 PSReadLine 的重绘行为
        // PSReadLine 在用户输入时会：
        // 1. 记住输入行的起始位置
        // 2. 每次按键后，用 CSI H 移回起始位置
        // 3. 重新输出整个输入内容
        //
        // 这个测试模拟：用户在终端最后一行开始输入，输入超过一行宽度
        // 导致 scroll_up，然后 PSReadLine 用 CSI H 移回"原位置"重绘
        let mut state = TerminalState::new(5, 10, 100);

        // 步骤 1: 将光标移到最后一行（模拟 prompt 在底部）
        state.set_cursor_pos(0, 4);

        // 记录输入起始的"视口行"
        let input_start_row = state.cursor().y; // 4 (0-indexed)

        println!("\n=== 初始状态 ===");
        println!(
            "cursor=({}, {}), scrollback={}",
            state.cursor().x,
            state.cursor().y,
            state.screen().scrollback_lines()
        );

        // 步骤 2: 输入第一批字符（不触发换行）
        let input1 = "12345";
        for c in input1.chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }

        println!("\n=== 输入 '{}' 后 ===", input1);
        println!(
            "cursor=({}, {}), scrollback={}",
            state.cursor().x,
            state.cursor().y,
            state.screen().scrollback_lines()
        );

        // 步骤 3: PSReadLine 风格的重绘
        // 移回起始位置并重绘整个输入
        state.set_cursor_pos(0, input_start_row);
        for c in input1.chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }

        // 验证：内容应该正确
        let line = state.screen().get_line_text(4).unwrap();
        assert_eq!(line, input1, "Line should contain input after first redraw");

        // 步骤 4: 继续输入，触发换行
        let input2 = "12345ABCDE"; // 10 个字符触发换行
        state.set_cursor_pos(0, input_start_row);

        // 详细追踪每个字符的写入
        println!("\n=== 开始输入 '{}' ===", input2);
        for (i, c) in input2.chars().enumerate() {
            let before_scrollback = state.screen().scrollback_lines();
            let cursor_before = (state.cursor().x, state.cursor().y);

            state.write_cell(Cell::new(&c.to_string()));

            let after_scrollback = state.screen().scrollback_lines();
            let cursor_after = (state.cursor().x, state.cursor().y);

            if before_scrollback != after_scrollback {
                println!(
                    "  char[{}]='{}': cursor {:?} -> {:?}, scroll_up! scrollback {} -> {}",
                    i, c, cursor_before, cursor_after, before_scrollback, after_scrollback
                );
            }
        }

        println!("\n=== 输入 '{}' 后 (触发换行) ===", input2);
        println!(
            "cursor=({}, {}), scrollback={}",
            state.cursor().x,
            state.cursor().y,
            state.screen().scrollback_lines()
        );

        // 此时 scrollback 应该为 1（因为换行触发了 scroll_up）
        let scrollback_after = state.screen().scrollback_lines();
        println!("scrollback = {}", scrollback_after);

        // 步骤 5: PSReadLine 再次重绘 - 这是问题发生的地方！
        // PSReadLine 会发送 CSI {input_start_row+1};1H 移回"原位置"
        // 但 scroll_up 后，原来的 row 4 现在对应的是不同的物理行
        println!("\n=== PSReadLine 重绘（问题点）===");
        println!(
            "PSReadLine 认为起始行是 row={}, 但 scrollback={}",
            input_start_row, scrollback_after
        );

        // 模拟 PSReadLine 行为：移回 row 4 并重绘
        state.set_cursor_pos(0, input_start_row);

        // 关键问题：此时写入会写到哪里？
        let vis_idx = VisibleRowIndex::new(input_start_row as isize);
        let phys_before = state.screen().visible_to_phys(vis_idx);
        println!(
            "visible_row={} -> phys_row={:?}",
            input_start_row,
            phys_before.map(|p| p.get())
        );

        // PSReadLine 重绘整个输入
        let input3 = "12345ABCDEFGH"; // 增加到 13 个字符
        for c in input3.chars() {
            state.write_cell(Cell::new(&c.to_string()));
        }

        println!("\n=== 重绘后的状态 ===");
        println!(
            "cursor=({}, {}), scrollback={}",
            state.cursor().x,
            state.cursor().y,
            state.screen().scrollback_lines()
        );

        // 打印所有物理行
        println!("\n=== 所有物理行 ===");
        for i in 0..state.screen().total_lines() {
            let phys = crate::grid::PhysRowIndex::new(i);
            if let Some(line) = state.screen().get_line(phys) {
                let text = line.text();
                println!("Phys[{}]: '{}'", i, text);
            }
        }

        // 打印所有可见行
        println!("\n=== 所有可见行 ===");
        for i in 0..5 {
            let text = state.screen().get_line_text(i).unwrap();
            println!("Visible[{}]: '{}'", i, text);
        }

        // 检查是否有内容重复
        let visible_lines: Vec<String> = (0..5)
            .map(|i| state.screen().get_line_text(i).unwrap())
            .collect();

        // 找出非空行
        let non_empty_lines: Vec<&String> = visible_lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .collect();

        println!("\n=== 非空行 ===");
        for (i, line) in non_empty_lines.iter().enumerate() {
            println!("[{}]: '{}'", i, line);
        }

        // 分析这个场景：
        // PSReadLine 在 scroll_up 后重绘，旧内容保留在 scrollback 中
        // 这实际上是正确的行为！
        //
        // 但用户报告的问题是：在 **可见区域** 看到了重复的行
        // 让我们验证可见行中是否有重复

        // 在真实场景中，PSReadLine 重绘应该：
        // 1. 移动到输入开始位置
        // 2. 输出整个输入内容
        // 3. 如果触发换行，旧行进入 scrollback，新行在 visible
        //
        // 所以可见区域不应该有重复！如果有重复，那是渲染 bug

        // 检查**可见区域**是否有重复
        // visible[2] 和 visible[3] 不应该相同
        println!("\n=== 验证可见区域 ===");
        let has_visible_duplicate = visible_lines[2] == visible_lines[3];
        println!("Visible[2] == Visible[3]? {}", has_visible_duplicate);

        // 这个断言应该**失败**来展示问题
        // 在正确的实现中，visible[2] 应该是 scrollback 后的内容
        // visible[3] 和 visible[4] 应该是新输入
        assert!(
            !has_visible_duplicate || visible_lines[2].trim().is_empty(),
            "Visible rows 2 and 3 should not have the same non-empty content. \
             Got: '{}' and '{}'",
            visible_lines[2],
            visible_lines[3]
        );
    }

    #[test]
    fn test_psreadline_multi_line_redraw() {
        // 模拟用户截图中的场景：输入一个非常长的字符串，跨越多行
        // 每次按键后 PSReadLine 都会重绘，导致多次 scroll_up
        // 验证不会出现多行重复
        let mut state = TerminalState::new(10, 20, 100);

        // 将光标移到最后一行（模拟 prompt 在底部）
        state.set_cursor_pos(0, 9);

        let input_start_row = state.cursor().y;

        println!("\n=== 初始状态 ===");
        println!(
            "cursor=({}, {}), scrollback={}, rows={}",
            state.cursor().x,
            state.cursor().y,
            state.screen().scrollback_lines(),
            state.rows()
        );

        // 模拟 PSReadLine 的渐进式重绘
        // 每次添加一个字符，然后"重绘"整个输入
        let full_input = "AAAAAAAAAABBBBBBBBBBCCCCCCCCCCDDDDDDDDDD"; // 40 字符，2 行

        for i in 1..=full_input.len() {
            let current_input = &full_input[..i];

            // PSReadLine 移回输入开始位置
            state.set_cursor_pos(0, input_start_row);

            // 重绘整个输入
            for c in current_input.chars() {
                state.write_cell(Cell::new(&c.to_string()));
            }
        }

        println!("\n=== 输入完成后 ===");
        println!(
            "cursor=({}, {}), scrollback={}",
            state.cursor().x,
            state.cursor().y,
            state.screen().scrollback_lines()
        );

        // 打印所有物理行
        println!("\n=== 所有物理行 ===");
        for i in 0..state.screen().total_lines() {
            let phys = crate::grid::PhysRowIndex::new(i);
            if let Some(line) = state.screen().get_line(phys) {
                let text = line.text();
                if !text.trim().is_empty() {
                    println!("Phys[{}]: '{}'", i, text);
                }
            }
        }

        // 打印所有可见行
        println!("\n=== 所有可见行 ===");
        let mut visible_lines = Vec::new();
        for i in 0..10 {
            let text = state.screen().get_line_text(i).unwrap();
            visible_lines.push(text.clone());
            if !text.trim().is_empty() {
                println!("Visible[{}]: '{}'", i, text);
            }
        }

        // 收集非空行
        let non_empty_lines: Vec<(usize, &String)> = visible_lines
            .iter()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
            .collect();

        println!("\n=== 非空行数量: {} ===", non_empty_lines.len());

        // 验证：非空行不应该有重复
        for i in 0..non_empty_lines.len() {
            for j in (i + 1)..non_empty_lines.len() {
                let (idx_i, line_i) = non_empty_lines[i];
                let (idx_j, line_j) = non_empty_lines[j];
                assert_ne!(
                    line_i, line_j,
                    "Visible rows {} and {} should not have the same content: '{}'",
                    idx_i, idx_j, line_i
                );
            }
        }

        // 验证输入内容完整（两行：20+20 字符）
        // 第一行应该是 "AAAAAAAAAABBBBBBBBBB"
        // 第二行应该是 "CCCCCCCCCCDDDDDDDDDD"
        let expected_line1 = "AAAAAAAAAABBBBBBBBBB";
        let expected_line2 = "CCCCCCCCCCDDDDDDDDDD";

        let found_line1 = non_empty_lines
            .iter()
            .any(|(_, l)| l.as_str() == expected_line1);
        let found_line2 = non_empty_lines
            .iter()
            .any(|(_, l)| l.as_str() == expected_line2);

        assert!(
            found_line1,
            "Should find first line '{}' in visible area",
            expected_line1
        );
        assert!(
            found_line2,
            "Should find second line '{}' in visible area",
            expected_line2
        );

        println!("\n=== 测试通过：没有重复行 ===");
    }

    #[test]
    fn test_scroll_up_does_not_change_visible_content_mapping() {
        // 这个测试验证：scroll_up 后，visible_to_phys 的映射应该正确反映
        // "视口第 N 行" 对应的物理行
        //
        // 关键洞察：scroll_up 后，视口内容应该"向上移动"
        // - 原来的 visible row 0 内容进入 scrollback
        // - 原来的 visible row 1 内容变成新的 visible row 0
        // - 底部添加新的空行
        let mut state = TerminalState::new(3, 10, 100);

        // 在每行写入标记内容
        state.set_cursor_pos(0, 0);
        state.write_cell(Cell::new("A")); // row 0
        state.set_cursor_pos(0, 1);
        state.write_cell(Cell::new("B")); // row 1
        state.set_cursor_pos(0, 2);
        state.write_cell(Cell::new("C")); // row 2

        // 验证初始状态
        assert_eq!(state.screen().scrollback_lines(), 0);
        assert_eq!(
            state.screen().get_line_text(0).unwrap().chars().next(),
            Some('A')
        );
        assert_eq!(
            state.screen().get_line_text(1).unwrap().chars().next(),
            Some('B')
        );
        assert_eq!(
            state.screen().get_line_text(2).unwrap().chars().next(),
            Some('C')
        );

        println!("\n=== 初始状态 ===");
        for i in 0..3 {
            println!(
                "Visible[{}]: '{}'",
                i,
                state.screen().get_line_text(i).unwrap()
            );
        }

        // 执行 scroll_up
        state.screen_mut().scroll_up();

        println!("\n=== scroll_up 后 ===");
        println!("scrollback = {}", state.screen().scrollback_lines());
        for i in 0..state.screen().total_lines() {
            let phys = crate::grid::PhysRowIndex::new(i);
            if let Some(line) = state.screen().get_line(phys) {
                println!("Phys[{}]: '{}'", i, line.text());
            }
        }
        for i in 0..3 {
            println!(
                "Visible[{}]: '{}'",
                i,
                state.screen().get_line_text(i).unwrap()
            );
        }

        // 验证 scroll_up 后的状态
        assert_eq!(state.screen().scrollback_lines(), 1);

        // 关键验证：visible 行应该反映滚动后的内容
        // - Visible[0] 应该是原来的 "B"（原 row 1）
        // - Visible[1] 应该是原来的 "C"（原 row 2）
        // - Visible[2] 应该是空行
        let vis0 = state.screen().get_line_text(0).unwrap();
        let vis1 = state.screen().get_line_text(1).unwrap();
        let vis2 = state.screen().get_line_text(2).unwrap();

        assert!(
            vis0.starts_with("B"),
            "After scroll_up, visible row 0 should be 'B', got '{}'",
            vis0
        );
        assert!(
            vis1.starts_with("C"),
            "After scroll_up, visible row 1 should be 'C', got '{}'",
            vis1
        );
        assert!(
            vis2.trim().is_empty(),
            "After scroll_up, visible row 2 should be empty, got '{}'",
            vis2
        );
    }
}
