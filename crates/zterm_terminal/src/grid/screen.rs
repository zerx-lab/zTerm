//! Screen 结构
//!
//! 终端屏幕管理，包括 scrollback 和视口
//! 对应 WezTerm 的 Screen

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::line::Line;
use super::stable_row_index::{PhysRowIndex, StableRowIndex, VisibleRowIndex};

/// 终端屏幕
///
/// 对应 WezTerm 的 Screen
/// 管理所有行（scrollback + viewport）并提供稳定的索引系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screen {
    /// 所有行（scrollback + viewport）
    lines: VecDeque<Line>,
    /// 视口高度（行数）
    rows: usize,
    /// 视口宽度（列数）
    cols: usize,
    /// Scrollback 缓冲区最大行数
    max_scrollback: usize,
    /// 当前 scrollback 偏移（用于稳定索引）
    scrollback_offset: usize,
}

impl Screen {
    /// 创建新屏幕
    pub fn new(rows: usize, cols: usize, max_scrollback: usize) -> Self {
        let mut lines = VecDeque::with_capacity(rows + max_scrollback);
        for _ in 0..rows {
            lines.push_back(Line::with_capacity(cols));
        }

        Self {
            lines,
            rows,
            cols,
            max_scrollback,
            scrollback_offset: 0,
        }
    }

    /// 获取视口行数
    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// 获取视口列数
    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// 获取总行数（scrollback + viewport）
    #[inline]
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// 获取 scrollback 行数
    #[inline]
    pub fn scrollback_lines(&self) -> usize {
        self.lines.len().saturating_sub(self.rows)
    }

    /// 获取有效的 scrollback 行数（排除顶部空行）
    ///
    /// 这用于限制滚动范围，避免滚动到全是空白的区域
    pub fn effective_scrollback_lines(&self) -> usize {
        // 找到第一个非空行的索引
        let first_non_empty = self
            .lines
            .iter()
            .position(|line| !line.is_empty())
            .unwrap_or(self.lines.len());

        // 有效的滚动范围是从第一个非空行到 viewport 开始的位置
        let scrollback_start = self.scrollback_lines();
        if first_non_empty >= scrollback_start {
            // 第一个非空行在 viewport 中，没有有效的 scrollback
            0
        } else {
            // 有效的 scrollback = scrollback 中非空行的数量
            scrollback_start.saturating_sub(first_non_empty)
        }
    }

    /// 获取最大 scrollback 行数
    #[inline]
    pub fn max_scrollback(&self) -> usize {
        self.max_scrollback
    }

    /// 获取 scrollback 偏移
    #[inline]
    pub fn scrollback_offset(&self) -> usize {
        self.scrollback_offset
    }

    // ========== 索引转换 ==========

    /// StableRowIndex -> PhysRowIndex
    pub fn stable_to_phys(&self, stable: StableRowIndex) -> Option<PhysRowIndex> {
        let stable_idx = stable.get();
        if stable_idx < self.scrollback_offset {
            // 行已被滚出
            None
        } else {
            let phys_idx = stable_idx - self.scrollback_offset;
            if phys_idx < self.lines.len() {
                Some(PhysRowIndex::new(phys_idx))
            } else {
                None
            }
        }
    }

    /// PhysRowIndex -> StableRowIndex
    #[inline]
    pub fn phys_to_stable(&self, phys: PhysRowIndex) -> StableRowIndex {
        StableRowIndex::new(phys.get() + self.scrollback_offset)
    }

    /// VisibleRowIndex -> PhysRowIndex
    pub fn visible_to_phys(&self, visible: VisibleRowIndex) -> Option<PhysRowIndex> {
        let vis_idx = visible.get();
        if vis_idx < 0 {
            // 在 scrollback 中
            let scrollback_lines = self.scrollback_lines();
            let phys_idx = scrollback_lines as isize + vis_idx;
            if phys_idx >= 0 {
                Some(PhysRowIndex::new(phys_idx as usize))
            } else {
                None
            }
        } else if (vis_idx as usize) < self.rows {
            // 在视口中
            let phys_idx = self.scrollback_lines() + vis_idx as usize;
            Some(PhysRowIndex::new(phys_idx))
        } else {
            None
        }
    }

    /// PhysRowIndex -> VisibleRowIndex
    pub fn phys_to_visible(&self, phys: PhysRowIndex) -> VisibleRowIndex {
        let scrollback_lines = self.scrollback_lines();
        let phys_idx = phys.get();

        if phys_idx < scrollback_lines {
            // 在 scrollback 中
            VisibleRowIndex::new(phys_idx as isize - scrollback_lines as isize)
        } else {
            // 在视口中
            VisibleRowIndex::new((phys_idx - scrollback_lines) as isize)
        }
    }

    // ========== 行访问 ==========

    /// 通过物理索引获取行
    pub fn get_line(&self, phys: PhysRowIndex) -> Option<&Line> {
        self.lines.get(phys.get())
    }

    /// 通过物理索引获取可变行
    pub fn get_line_mut(&mut self, phys: PhysRowIndex) -> Option<&mut Line> {
        self.lines.get_mut(phys.get())
    }

    /// 通过稳定索引获取行
    pub fn get_line_stable(&self, stable: StableRowIndex) -> Option<&Line> {
        self.stable_to_phys(stable)
            .and_then(|phys| self.get_line(phys))
    }

    /// 通过稳定索引获取可变行
    pub fn get_line_stable_mut(&mut self, stable: StableRowIndex) -> Option<&mut Line> {
        self.stable_to_phys(stable)
            .and_then(|phys| self.get_line_mut(phys))
    }

    /// 通过可见索引获取行
    pub fn get_visible_line(&self, visible: VisibleRowIndex) -> Option<&Line> {
        self.visible_to_phys(visible)
            .and_then(|phys| self.get_line(phys))
    }

    /// 通过可见索引获取可变行
    pub fn get_visible_line_mut(&mut self, visible: VisibleRowIndex) -> Option<&mut Line> {
        self.visible_to_phys(visible)
            .and_then(|phys| self.get_line_mut(phys))
    }

    // ========== 滚动操作 ==========

    /// 向上滚动一行（新行进入底部）
    pub fn scroll_up(&mut self) {
        // 将第一个视口行移入 scrollback
        // 在底部添加新空行

        // 限制 scrollback 大小
        let total_lines = self.lines.len();
        let scrollback_lines = total_lines.saturating_sub(self.rows);

        if scrollback_lines >= self.max_scrollback {
            // 移除最旧的 scrollback 行
            self.lines.pop_front();
            self.scrollback_offset += 1;
        }

        // 在底部添加新空行
        let new_line = Line::with_capacity(self.cols);
        tracing::debug!(
            "Screen::scroll_up: adding new line (len={}), total {} -> {}",
            new_line.len(),
            self.lines.len(),
            self.lines.len() + 1
        );
        self.lines.push_back(new_line);

        tracing::debug!(
            "Screen::scroll_up: total_lines={}, scrollback={}, rows={}",
            self.lines.len(),
            self.scrollback_lines(),
            self.rows
        );
    }

    /// 向下滚动一行（仅在 scrollback 模式下）
    pub fn scroll_down(&mut self) {
        if self.scrollback_lines() > 0 {
            self.lines.pop_back();
            self.lines.push_front(Line::with_capacity(self.cols));
            if self.scrollback_offset > 0 {
                self.scrollback_offset -= 1;
            }
        }
    }

    /// 在指定的上下边界内向上滚动（参考 WezTerm）
    ///
    /// - top: 滚动区域顶部（包含，0-based）
    /// - bottom: 滚动区域底部（不包含，0-based）
    /// - num_rows: 滚动行数
    pub fn scroll_up_within_margins(&mut self, top: usize, bottom: usize, num_rows: usize) {
        if top >= bottom || num_rows == 0 {
            return;
        }

        let scrollback_start = self.scrollback_lines();
        let region_height = bottom.saturating_sub(top);
        let actual_scroll = num_rows.min(region_height);

        // 如果是全屏滚动（top == 0 && bottom == rows）
        if top == 0 && bottom == self.rows {
            // 使用原有的 scroll_up 逻辑
            for _ in 0..actual_scroll {
                self.scroll_up();
            }
            return;
        }

        // 滚动区域内的滚动（不进入 scrollback）
        // 移动内容：从 top+actual_scroll 开始的行移动到 top
        for i in 0..actual_scroll {
            let src_phys = scrollback_start + top + actual_scroll + i;
            let dst_phys = scrollback_start + top + i;

            if src_phys < scrollback_start + bottom && dst_phys < self.lines.len() {
                // 复制源行内容到目标行
                if src_phys < self.lines.len() {
                    let src_line = self.lines[src_phys].clone();
                    self.lines[dst_phys] = src_line;
                }
            }
        }

        // 在底部填充空白行
        for i in 0..actual_scroll {
            let phys = scrollback_start + bottom - actual_scroll + i;
            if phys < self.lines.len() {
                self.lines[phys] = Line::with_capacity(self.cols);
            }
        }
    }

    /// 在指定的上下边界内向下滚动
    pub fn scroll_down_within_margins(&mut self, top: usize, bottom: usize, num_rows: usize) {
        if top >= bottom || num_rows == 0 {
            return;
        }

        let scrollback_start = self.scrollback_lines();
        let region_height = bottom.saturating_sub(top);
        let actual_scroll = num_rows.min(region_height);

        // 从底部向顶部移动内容
        for i in (0..region_height - actual_scroll).rev() {
            let src_phys = scrollback_start + top + i;
            let dst_phys = scrollback_start + top + i + actual_scroll;

            if dst_phys < scrollback_start + bottom && src_phys < self.lines.len() {
                if dst_phys < self.lines.len() {
                    let src_line = self.lines[src_phys].clone();
                    self.lines[dst_phys] = src_line;
                }
            }
        }

        // 在顶部填充空白行
        for i in 0..actual_scroll {
            let phys = scrollback_start + top + i;
            if phys < self.lines.len() {
                self.lines[phys] = Line::with_capacity(self.cols);
            }
        }
    }

    /// 在指定位置插入空行
    pub fn insert_line(&mut self, phys: PhysRowIndex) {
        if phys.get() < self.lines.len() {
            self.lines
                .insert(phys.get(), Line::with_capacity(self.cols));

            // 限制总行数
            let max_total = self.rows + self.max_scrollback;
            while self.lines.len() > max_total {
                self.lines.pop_back();
            }
        }
    }

    /// 删除指定行
    pub fn delete_line(&mut self, phys: PhysRowIndex) -> Option<Line> {
        if phys.get() < self.lines.len() {
            self.lines.remove(phys.get())
        } else {
            None
        }
    }

    /// 调整屏幕大小
    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        let old_rows = self.rows;
        self.rows = new_rows;
        self.cols = new_cols;

        // 调整行数
        if new_rows > old_rows {
            // 增加行数
            for _ in 0..(new_rows - old_rows) {
                self.lines.push_back(Line::with_capacity(new_cols));
            }
        } else if new_rows < old_rows {
            // 减少行数
            let to_remove = old_rows - new_rows;
            for _ in 0..to_remove {
                if self.lines.len() > new_rows {
                    self.lines.pop_back();
                }
            }
        }

        // 调整每行的宽度
        for line in &mut self.lines {
            if line.len() < new_cols {
                line.resize(new_cols, super::cell::Cell::blank());
            }
        }
    }

    /// 清空屏幕
    pub fn clear(&mut self) {
        self.lines.clear();
        for _ in 0..self.rows {
            self.lines.push_back(Line::with_capacity(self.cols));
        }
        self.scrollback_offset = 0;
    }

    /// 清空 scrollback
    pub fn clear_scrollback(&mut self) {
        let viewport_start = self.scrollback_lines();
        self.lines.drain(0..viewport_start);
        self.scrollback_offset += viewport_start;
    }

    // ========== 文本提取 ==========

    /// 获取指定行的文本内容（基于可见行索引）
    pub fn get_line_text(&self, visible_row: usize) -> Option<String> {
        let scrollback_lines = self.scrollback_lines();
        let phys_idx = scrollback_lines + visible_row;

        self.lines.get(phys_idx).map(|line| {
            let cells = line.to_vec();
            let mut text = String::new();
            for cell in cells {
                text.push_str(cell.text());
            }
            // 去除尾部空格
            text.trim_end().to_string()
        })
    }

    // ========== 迭代器 ==========

    /// 迭代所有物理行
    pub fn iter_phys(&self) -> impl Iterator<Item = (PhysRowIndex, &Line)> {
        self.lines
            .iter()
            .enumerate()
            .map(|(idx, line)| (PhysRowIndex::new(idx), line))
    }

    /// 迭代可见行
    pub fn iter_visible(&self) -> impl Iterator<Item = (VisibleRowIndex, &Line)> {
        let scrollback_lines = self.scrollback_lines();
        self.lines
            .iter()
            .skip(scrollback_lines)
            .take(self.rows)
            .enumerate()
            .map(|(idx, line)| (VisibleRowIndex::new(idx as isize), line))
    }

    /// 获取所有可见行
    pub fn visible_lines(&self) -> &[Line] {
        let start = self.scrollback_lines();
        let end = (start + self.rows).min(self.lines.len());
        // VecDeque 不支持直接切片，需要转换
        // 这里简化处理，实际应该优化
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_new() {
        let screen = Screen::new(24, 80, 1000);
        assert_eq!(screen.rows(), 24);
        assert_eq!(screen.cols(), 80);
        assert_eq!(screen.total_lines(), 24);
        assert_eq!(screen.scrollback_lines(), 0);
    }

    #[test]
    fn test_screen_scroll_up() {
        let mut screen = Screen::new(3, 10, 10);
        assert_eq!(screen.total_lines(), 3);

        screen.scroll_up();
        assert_eq!(screen.total_lines(), 4);
        assert_eq!(screen.scrollback_lines(), 1);

        screen.scroll_up();
        assert_eq!(screen.total_lines(), 5);
        assert_eq!(screen.scrollback_lines(), 2);
    }

    #[test]
    fn test_screen_scroll_up_limit() {
        let mut screen = Screen::new(3, 10, 2);

        // 添加 2 行 scrollback
        screen.scroll_up();
        screen.scroll_up();
        assert_eq!(screen.scrollback_lines(), 2);

        // 第 3 行应该移除最旧的
        screen.scroll_up();
        assert_eq!(screen.scrollback_lines(), 2);
        assert_eq!(screen.scrollback_offset(), 1);
    }

    #[test]
    fn test_stable_to_phys() {
        let mut screen = Screen::new(3, 10, 10);

        let stable0 = StableRowIndex::new(0);
        let phys0 = screen.stable_to_phys(stable0).unwrap();
        assert_eq!(phys0.get(), 0);

        // 滚动后
        screen.scroll_up();
        let stable0_after = StableRowIndex::new(0);
        let phys0_after = screen.stable_to_phys(stable0_after).unwrap();
        assert_eq!(phys0_after.get(), 0); // 现在在 scrollback 中

        let stable1 = StableRowIndex::new(1);
        let phys1 = screen.stable_to_phys(stable1).unwrap();
        assert_eq!(phys1.get(), 1);
    }

    #[test]
    fn test_phys_to_stable() {
        let mut screen = Screen::new(3, 10, 10);

        let phys0 = PhysRowIndex::new(0);
        let stable0 = screen.phys_to_stable(phys0);
        assert_eq!(stable0.get(), 0);

        screen.scroll_up();
        let phys0_after = PhysRowIndex::new(0);
        let stable0_after = screen.phys_to_stable(phys0_after);
        assert_eq!(stable0_after.get(), 0); // scrollback 中的第一行
    }

    #[test]
    fn test_visible_to_phys() {
        let mut screen = Screen::new(3, 10, 10);

        let vis0 = VisibleRowIndex::new(0);
        let phys0 = screen.visible_to_phys(vis0).unwrap();
        assert_eq!(phys0.get(), 0);

        // 添加 scrollback
        screen.scroll_up();
        screen.scroll_up();

        let vis0_after = VisibleRowIndex::new(0);
        let phys0_after = screen.visible_to_phys(vis0_after).unwrap();
        assert_eq!(phys0_after.get(), 2); // 跳过 2 行 scrollback

        // 访问 scrollback
        let vis_neg1 = VisibleRowIndex::new(-1);
        let phys_neg1 = screen.visible_to_phys(vis_neg1).unwrap();
        assert_eq!(phys_neg1.get(), 1); // scrollback 的最后一行
    }

    #[test]
    fn test_phys_to_visible() {
        let mut screen = Screen::new(3, 10, 10);
        screen.scroll_up();
        screen.scroll_up();

        let phys0 = PhysRowIndex::new(0);
        let vis0 = screen.phys_to_visible(phys0);
        assert_eq!(vis0.get(), -2); // scrollback 中

        let phys2 = PhysRowIndex::new(2);
        let vis2 = screen.phys_to_visible(phys2);
        assert_eq!(vis2.get(), 0); // 视口第一行
    }

    #[test]
    fn test_get_line() {
        let screen = Screen::new(3, 10, 10);

        let phys0 = PhysRowIndex::new(0);
        assert!(screen.get_line(phys0).is_some());

        let phys100 = PhysRowIndex::new(100);
        assert!(screen.get_line(phys100).is_none());
    }

    #[test]
    fn test_insert_line() {
        let mut screen = Screen::new(3, 10, 10);
        let initial_count = screen.total_lines();

        screen.insert_line(PhysRowIndex::new(1));
        assert_eq!(screen.total_lines(), initial_count + 1);
    }

    #[test]
    fn test_delete_line() {
        let mut screen = Screen::new(3, 10, 10);

        let deleted = screen.delete_line(PhysRowIndex::new(1));
        assert!(deleted.is_some());
        assert_eq!(screen.total_lines(), 2);
    }

    #[test]
    fn test_resize() {
        let mut screen = Screen::new(24, 80, 1000);

        screen.resize(30, 100);
        assert_eq!(screen.rows(), 30);
        assert_eq!(screen.cols(), 100);
        assert_eq!(screen.total_lines(), 30);
    }

    #[test]
    fn test_clear() {
        let mut screen = Screen::new(3, 10, 10);
        screen.scroll_up();
        screen.scroll_up();

        screen.clear();
        assert_eq!(screen.total_lines(), 3);
        assert_eq!(screen.scrollback_lines(), 0);
        assert_eq!(screen.scrollback_offset(), 0);
    }

    #[test]
    fn test_clear_scrollback() {
        let mut screen = Screen::new(3, 10, 10);
        screen.scroll_up();
        screen.scroll_up();

        let scrollback = screen.scrollback_lines();
        screen.clear_scrollback();
        assert_eq!(screen.scrollback_lines(), 0);
        assert_eq!(screen.scrollback_offset(), scrollback);
    }

    #[test]
    fn test_iter_visible() {
        let screen = Screen::new(3, 10, 10);
        let mut count = 0;

        for (vis_idx, _line) in screen.iter_visible() {
            assert!(vis_idx.is_visible(screen.rows()));
            count += 1;
        }

        assert_eq!(count, 3);
    }

    #[test]
    fn test_effective_scrollback_lines_all_empty() {
        // 所有行都是空的，有效 scrollback 应该是 0
        let mut screen = Screen::new(3, 10, 10);
        screen.scroll_up();
        screen.scroll_up();

        assert_eq!(screen.scrollback_lines(), 2);
        assert_eq!(screen.effective_scrollback_lines(), 0);
    }

    #[test]
    fn test_effective_scrollback_lines_with_content() {
        use crate::grid::Cell;

        let mut screen = Screen::new(3, 10, 10);

        // 在第一行写入内容（使用 push 因为 set_cell 需要先 resize）
        if let Some(line) = screen.get_line_mut(PhysRowIndex::new(0)) {
            line.push(Cell::new("H"));
        }

        // 验证第一行不为空
        assert!(!screen.get_line(PhysRowIndex::new(0)).unwrap().is_empty());

        // 滚动，使第一行进入 scrollback
        screen.scroll_up();
        screen.scroll_up();

        assert_eq!(screen.scrollback_lines(), 2);
        // 第一行有内容，所以有效 scrollback 是 2
        assert_eq!(screen.effective_scrollback_lines(), 2);
    }

    #[test]
    fn test_effective_scrollback_lines_partial_content() {
        use crate::grid::Cell;

        let mut screen = Screen::new(3, 10, 10);

        // 滚动两次，创建 2 行空的 scrollback（原来的第 0、1 行变成 scrollback）
        screen.scroll_up();
        screen.scroll_up();

        // 在物理索引 1 写入内容（这是原来的第 1 行，现在在 scrollback 中）
        if let Some(line) = screen.get_line_mut(PhysRowIndex::new(1)) {
            line.push(Cell::new("X"));
        }

        assert_eq!(screen.scrollback_lines(), 2);
        // 第 0 行是空的，第 1 行有内容
        // 有效 scrollback = scrollback_lines - first_non_empty = 2 - 1 = 1
        assert_eq!(screen.effective_scrollback_lines(), 1);
    }
}
