//! 终端状态管理

use crate::{
    event::{ChannelEventListener, TerminalEvent},
    grid::{CursorShape, TerminalState},
    pty::{PtyEventLoop, PtyMessage},
    vte_performer::VtePerformer,
    PtyConfig, TerminalConfig, TerminalSize,
};

/// 光标信息（用于渲染）
#[derive(Debug, Clone, Copy)]
pub struct CursorInfo {
    /// X 坐标（列）
    pub x: usize,
    /// Y 坐标（行）
    pub y: usize,
    /// 是否可见
    pub visible: bool,
    /// 光标形状
    pub shape: CursorShape,
}
use anyhow::{Context, Result};
use flume::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;
use vte::Parser;

/// 终端实例
pub struct Terminal {
    /// PTY 事件循环消息发送器
    pty_tx: Sender<PtyMessage>,

    /// 终端事件接收器
    event_rx: Receiver<TerminalEvent>,

    /// VTE 解析器
    parser: Arc<Mutex<Parser>>,

    /// VTE Performer
    performer: Arc<Mutex<VtePerformer>>,

    /// 终端状态（Grid、Cursor 等）
    state: Arc<Mutex<TerminalState>>,

    /// 配置
    #[allow(dead_code)]
    config: TerminalConfig,

    /// 当前尺寸
    size: Arc<Mutex<TerminalSize>>,

    /// 显示偏移量（用于滚动，0 表示显示最新内容）
    /// display_offset 表示从底部向上偏移的行数
    display_offset: Arc<Mutex<usize>>,

    /// PTY 事件循环 (保持生命周期,防止线程退出)
    #[allow(dead_code)]
    pty_loop: PtyEventLoop,
}

impl Terminal {
    /// 创建新的终端实例
    pub fn new(pty_config: PtyConfig, term_config: TerminalConfig) -> Result<Self> {
        tracing::info!(
            "Creating Terminal with size {}x{}",
            pty_config.initial_size.cols,
            pty_config.initial_size.rows
        );

        // 创建事件通道
        let (event_tx, event_rx) = flume::unbounded();
        let event_listener = Arc::new(ChannelEventListener::new(event_tx));

        // 创建 PTY 事件循环
        let pty_loop =
            PtyEventLoop::new(&pty_config, event_listener.clone()).context("创建 PTY 失败")?;

        let pty_tx = pty_loop.sender();

        // 启动 PTY 事件循环 (spawn 返回 self 以保持生命周期)
        let pty_loop = pty_loop.spawn().context("启动 PTY 事件循环失败")?;
        tracing::debug!("PTY event loop spawned");

        // 创建终端状态（Grid）
        let rows = pty_config.initial_size.rows as usize;
        let cols = pty_config.initial_size.cols as usize;
        let max_scrollback = term_config.scrollback_lines;
        let state = Arc::new(Mutex::new(TerminalState::new(rows, cols, max_scrollback)));
        tracing::debug!(
            "TerminalState created: {}x{}, scrollback={}",
            rows,
            cols,
            max_scrollback
        );

        // 创建 VTE 解析器和 Performer
        let parser = Arc::new(Mutex::new(Parser::new()));
        let performer = Arc::new(Mutex::new(VtePerformer::new(state.clone(), event_listener)));

        tracing::info!("Terminal created successfully");

        Ok(Self {
            pty_tx,
            event_rx,
            parser,
            performer,
            state,
            config: term_config,
            size: Arc::new(Mutex::new(pty_config.initial_size)),
            display_offset: Arc::new(Mutex::new(0)),
            pty_loop,
        })
    }

    /// 写入数据到 PTY
    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.pty_tx
            .send(PtyMessage::Input(data.to_vec()))
            .context("发送输入失败")
    }

    /// 调整终端尺寸
    pub fn resize(&self, size: TerminalSize) -> Result<()> {
        // Update internal size
        *self.size.lock() = size;

        // Resize the terminal state (screen buffer)
        {
            let mut state = self.state.lock();
            state.resize(size.rows as usize, size.cols as usize);
        }

        // Send resize to PTY
        self.pty_tx
            .send(PtyMessage::Resize(size))
            .context("发送 resize 消息失败")
    }

    /// 获取当前尺寸
    pub fn size(&self) -> TerminalSize {
        *self.size.lock()
    }

    /// 获取事件接收器
    pub fn event_receiver(&self) -> Receiver<TerminalEvent> {
        self.event_rx.clone()
    }

    /// 处理从 PTY 读取的数据
    ///
    /// 性能优化：VTE Performer 使用脏标记而不是每个字符触发 Wakeup，
    /// 在整批数据处理完成后统一发送一次 Wakeup 事件
    pub fn process_pty_data(&self, data: &[u8]) {
        tracing::trace!("Terminal::process_pty_data: {} bytes", data.len());

        let mut parser = self.parser.lock();
        let mut performer = self.performer.lock();

        // 解析所有数据
        parser.advance(&mut *performer, data);

        // 统一发送一次 Wakeup（如果有任何内容变化）
        // flush_wakeup() 会检查脏标记并在需要时发送 Wakeup
        performer.flush_wakeup();
    }

    /// 获取终端状态（用于渲染）
    pub fn state(&self) -> Arc<Mutex<TerminalState>> {
        self.state.clone()
    }

    /// 获取终端内容为文本（用于调试或简单渲染）
    pub fn get_visible_lines(&self) -> Vec<String> {
        let state = self.state.lock();
        let screen = state.screen();
        let rows = state.rows();

        let lines: Vec<String> = (0..rows)
            .map(|row_idx| {
                screen
                    .get_line_text(row_idx)
                    .unwrap_or_else(|| String::new())
            })
            .collect();

        tracing::trace!(
            "get_visible_lines: {} rows, total_lines={}",
            rows,
            screen.total_lines()
        );
        lines
    }

    /// 获取可见区域的 Cell 数据（用于带样式渲染）
    ///
    /// 返回 Vec<Vec<Cell>>，每个内部 Vec 是一行的所有 Cell
    /// 注意：这只返回视口内的行，不包括 scrollback
    pub fn get_visible_cells(&self) -> Vec<Vec<crate::grid::Cell>> {
        let state = self.state.lock();
        let screen = state.screen();
        let rows = state.rows();
        let cols = state.cols();

        (0..rows)
            .map(|row_idx| {
                let vis_idx = crate::grid::VisibleRowIndex::new(row_idx as isize);
                if let Some(phys) = screen.visible_to_phys(vis_idx) {
                    if let Some(line) = screen.get_line(phys) {
                        let mut cells = line.to_vec();
                        // 确保行有足够的列数
                        if cells.len() < cols {
                            cells.resize(cols, crate::grid::Cell::blank());
                        }
                        return cells;
                    }
                }
                // 返回空白行
                vec![crate::grid::Cell::blank(); cols]
            })
            .collect()
    }

    /// 获取所有行的 Cell 数据（scrollback + viewport）
    ///
    /// 用于完整渲染终端内容，支持滚动查看历史
    /// 返回 (cells, scrollback_lines, viewport_rows)
    pub fn get_all_cells(&self) -> (Vec<Vec<crate::grid::Cell>>, usize, usize) {
        let state = self.state.lock();
        let screen = state.screen();
        let cols = state.cols();
        let viewport_rows = state.rows();
        let total_lines = screen.total_lines();
        let scrollback_lines = screen.scrollback_lines();

        tracing::debug!(
            "get_all_cells: total_lines={}, scrollback={}, viewport={}",
            total_lines,
            scrollback_lines,
            viewport_rows
        );

        let cells: Vec<Vec<crate::grid::Cell>> = (0..total_lines)
            .map(|phys_idx| {
                let phys = crate::grid::PhysRowIndex::new(phys_idx);
                if let Some(line) = screen.get_line(phys) {
                    let mut cells = line.to_vec();
                    if cells.len() < cols {
                        cells.resize(cols, crate::grid::Cell::blank());
                    }
                    cells
                } else {
                    vec![crate::grid::Cell::blank(); cols]
                }
            })
            .collect();

        tracing::debug!("get_all_cells: returning {} rows", cells.len());

        (cells, scrollback_lines, viewport_rows)
    }

    /// 获取 scrollback 信息
    pub fn scrollback_info(&self) -> (usize, usize) {
        let state = self.state.lock();
        let screen = state.screen();
        (screen.scrollback_lines(), screen.total_lines())
    }

    /// 获取光标信息（用于渲染）
    pub fn cursor_info(&self) -> CursorInfo {
        let state = self.state.lock();
        let cursor = state.cursor();
        CursorInfo {
            x: cursor.x,
            y: cursor.y,
            visible: cursor.visible,
            shape: cursor.shape,
        }
    }

    /// 获取光标位置
    pub fn cursor_position(&self) -> (usize, usize) {
        let state = self.state.lock();
        let cursor = state.cursor();
        (cursor.x, cursor.y)
    }

    // ========== 滚动功能 ==========

    /// 获取当前显示偏移量
    /// 返回从底部向上偏移的行数，0 表示显示最新内容
    pub fn display_offset(&self) -> usize {
        *self.display_offset.lock()
    }

    /// 获取最大显示偏移量（排除顶部空行的有效 scrollback 行数）
    pub fn max_display_offset(&self) -> usize {
        let state = self.state.lock();
        state.screen().effective_scrollback_lines()
    }

    /// 向上滚动指定行数（查看历史）
    pub fn scroll_up_by(&self, lines: usize) {
        let max_offset = self.max_display_offset();
        let mut offset = self.display_offset.lock();
        *offset = (*offset + lines).min(max_offset);
        tracing::debug!("scroll_up_by {}: new offset = {}", lines, *offset);
    }

    /// 向下滚动指定行数（返回最新）
    pub fn scroll_down_by(&self, lines: usize) {
        let mut offset = self.display_offset.lock();
        *offset = offset.saturating_sub(lines);
        tracing::debug!("scroll_down_by {}: new offset = {}", lines, *offset);
    }

    /// 滚动到顶部（最早的历史）
    pub fn scroll_to_top(&self) {
        let max_offset = self.max_display_offset();
        *self.display_offset.lock() = max_offset;
        tracing::debug!("scroll_to_top: offset = {}", max_offset);
    }

    /// 滚动到底部（最新内容）
    pub fn scroll_to_bottom(&self) {
        *self.display_offset.lock() = 0;
        tracing::debug!("scroll_to_bottom: offset = 0");
    }

    /// 获取视口内的 Cell 数据（考虑 display_offset）
    ///
    /// 返回 (cells, total_lines, display_offset)
    /// cells: 视口内的行，从上到下
    pub fn get_viewport_cells(&self) -> (Vec<Vec<crate::grid::Cell>>, usize, usize) {
        let state = self.state.lock();
        let screen = state.screen();
        let cols = state.cols();
        let viewport_rows = state.rows();
        let total_lines = screen.total_lines();
        let display_offset = *self.display_offset.lock();

        // 计算视口的起始和结束位置
        // display_offset=0 时，显示最后 viewport_rows 行
        // display_offset=N 时，显示从底部向上偏移 N 行的内容
        let end_line = total_lines.saturating_sub(display_offset);
        let start_line = end_line.saturating_sub(viewport_rows);

        let scrollback = screen.scrollback_lines();
        tracing::debug!(
            "get_viewport_cells: total={}, viewport={}, scrollback={}, offset={}, range={}..{}",
            total_lines,
            viewport_rows,
            scrollback,
            display_offset,
            start_line,
            end_line
        );

        let cells: Vec<Vec<crate::grid::Cell>> = (start_line..end_line)
            .enumerate()
            .map(|(view_idx, phys_idx)| {
                let phys = crate::grid::PhysRowIndex::new(phys_idx);
                if let Some(line) = screen.get_line(phys) {
                    let mut cells = line.to_vec();
                    // DEBUG: 打印有内容行的前几个字符
                    let preview: String = cells.iter().take(20).map(|c| c.text()).collect();
                    let non_empty = cells.iter().filter(|c| !c.text().trim().is_empty()).count();
                    if non_empty > 0 && scrollback > 0 {
                        tracing::debug!(
                            "  READ view[{}] = phys[{}]: non_empty={}, preview='{}'",
                            view_idx,
                            phys_idx,
                            non_empty,
                            preview
                        );
                    }
                    if cells.len() < cols {
                        cells.resize(cols, crate::grid::Cell::blank());
                    }
                    cells
                } else {
                    tracing::warn!("  view[{}] = phys[{}]: LINE NOT FOUND!", view_idx, phys_idx);
                    vec![crate::grid::Cell::blank(); cols]
                }
            })
            .collect();

        (cells, total_lines, display_offset)
    }

    /// 关闭终端
    pub fn shutdown(&self) -> Result<()> {
        self.pty_tx
            .send(PtyMessage::Shutdown)
            .context("发送关闭消息失败")
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let pty_config = PtyConfig::default();
        let term_config = TerminalConfig::default();

        let terminal = Terminal::new(pty_config, term_config);
        assert!(terminal.is_ok());
    }

    #[test]
    fn test_terminal_resize() {
        let pty_config = PtyConfig::default();
        let term_config = TerminalConfig::default();

        let terminal = Terminal::new(pty_config, term_config).unwrap();

        let new_size = TerminalSize::new(40, 120);
        assert!(terminal.resize(new_size).is_ok());
        assert_eq!(terminal.size(), new_size);
    }
}
