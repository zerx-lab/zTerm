//! TerminalState 结构
//!
//! 完整的终端状态，包含 Screen、Cursor 和渲染属性

use serde::{Deserialize, Serialize};

use super::cursor::{Cursor, CursorShape};
use super::screen::Screen;
use super::{Cell, CellAttributes, PhysRowIndex, VisibleRowIndex};

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
    /// 滚动区域
    scroll_region: Option<ScrollRegion>,
    /// 终端模式
    modes: TerminalModes,
    /// 当前标题
    title: String,
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
            scroll_region: None,
            modes: TerminalModes::default(),
            title: String::new(),
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
    pub fn set_cursor_pos(&mut self, x: usize, y: usize) {
        let max_cols = self.cols();
        let max_rows = self.rows();

        self.cursor.x = x.min(max_cols.saturating_sub(1));
        self.cursor.y = y.min(max_rows.saturating_sub(1));
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
    }

    // ========== 滚动区域 ==========

    /// 设置滚动区域
    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        let max_rows = self.rows();
        if top < bottom && bottom < max_rows {
            self.scroll_region = Some(ScrollRegion::new(top, bottom));
        }
    }

    /// 清除滚动区域
    pub fn clear_scroll_region(&mut self) {
        self.scroll_region = None;
    }

    /// 获取滚动区域
    #[inline]
    pub fn scroll_region(&self) -> Option<ScrollRegion> {
        self.scroll_region
    }

    /// 获取有效滚动区域（如果未设置则返回全屏）
    pub fn effective_scroll_region(&self) -> ScrollRegion {
        self.scroll_region
            .unwrap_or_else(|| ScrollRegion::new(0, self.rows().saturating_sub(1)))
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
        let x = self.cursor.x;
        let y = self.cursor.y;
        let cell_width = cell.width();
        let vis_idx = VisibleRowIndex::new(y as isize);

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

        // 移动光标
        self.cursor.x += cell_width as usize;
        if self.cursor.x >= self.cols() {
            if self.modes.contains(TerminalModes::AUTO_WRAP) {
                self.cursor.x = 0;
                if self.cursor.y + 1 < self.rows() {
                    self.cursor.y += 1;
                } else {
                    self.screen.scroll_up();
                }
            } else {
                self.cursor.x = self.cols() - 1;
            }
        }
    }

    /// 滚动（在滚动区域内）
    pub fn scroll_up_region(&mut self, lines: usize) {
        for _ in 0..lines {
            self.screen.scroll_up();
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

        let cell = Cell::new("a");
        state.write_cell(cell);

        assert_eq!(state.cursor().x, 0);
        assert_eq!(state.cursor().y, 1); // 换行
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
}
