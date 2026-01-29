//! Terminal Scrollbar
//!
//! 终端滚动条实现，参考 Zed 的设计模式。
//! 滚动条直接在 TerminalElement 中绘制，不作为独立组件。

use gpui::{px, Bounds, Hsla, Pixels, Point};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use zterm_terminal::Terminal;

/// 滚动条宽度
pub const SCROLLBAR_WIDTH: Pixels = px(8.0);
/// 滚动条内边距
pub const SCROLLBAR_PADDING: Pixels = px(4.0);
/// 滑块最小高度
pub const THUMB_MIN_HEIGHT: Pixels = px(30.0);

/// 滚动条滑块状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThumbState {
    /// 未激活
    Inactive,
    /// 鼠标悬停
    Hovered,
    /// 正在拖拽，存储点击时鼠标在滑块内的偏移
    Dragging { offset_in_thumb: Pixels },
}

impl Default for ThumbState {
    fn default() -> Self {
        Self::Inactive
    }
}

/// 滚动状态内部数据
#[derive(Debug, Clone, Copy)]
struct ScrollHandleState {
    /// 行高(像素)
    line_height: Pixels,
    /// 总行数(包括滚动历史)
    total_lines: usize,
    /// 视口可见行数
    viewport_lines: usize,
    /// 当前滚动偏移(从底部算起的行数，0=最底部)
    display_offset: usize,
}

impl Default for ScrollHandleState {
    fn default() -> Self {
        Self {
            line_height: px(16.0),
            total_lines: 0,
            viewport_lines: 0,
            display_offset: 0,
        }
    }
}

/// 终端滚动句柄
///
/// 用于在终端和滚动条之间同步滚动状态。
/// 使用 Rc<RefCell<>> 在非 Entity 上下文中共享状态。
#[derive(Clone)]
pub struct TerminalScrollHandle {
    terminal: Arc<Terminal>,
    state: Rc<RefCell<ScrollHandleState>>,
    /// 滑块状态
    pub thumb_state: Rc<Cell<ThumbState>>,
    /// 由滚动条拖拽设置的新偏移，在下一帧渲染时应用
    pub future_display_offset: Rc<Cell<Option<usize>>>,
}

impl TerminalScrollHandle {
    /// 创建新的滚动句柄
    pub fn new(terminal: Arc<Terminal>) -> Self {
        Self {
            terminal,
            state: Rc::new(RefCell::new(ScrollHandleState::default())),
            thumb_state: Rc::new(Cell::new(ThumbState::Inactive)),
            future_display_offset: Rc::new(Cell::new(None)),
        }
    }

    /// 从终端更新滚动状态
    pub fn update(&self, line_height: Pixels) {
        let size = self.terminal.size();
        let (_, total_lines, display_offset) = self.terminal.get_viewport_cells();

        let mut state = self.state.borrow_mut();
        state.line_height = line_height;
        state.total_lines = total_lines;
        state.viewport_lines = size.rows as usize;
        state.display_offset = display_offset;
    }

    /// 应用待处理的滚动偏移
    pub fn apply_pending_scroll(&self) {
        if let Some(new_offset) = self.future_display_offset.take() {
            let current_offset = self.terminal.display_offset();
            if new_offset != current_offset {
                if new_offset > current_offset {
                    self.terminal
                        .scroll_up_by(new_offset.saturating_sub(current_offset));
                } else {
                    self.terminal
                        .scroll_down_by(current_offset.saturating_sub(new_offset));
                }
            }
        }
    }

    /// 是否可滚动
    pub fn is_scrollable(&self) -> bool {
        let state = self.state.borrow();
        state.total_lines > state.viewport_lines
    }

    /// 获取最大滚动偏移（行数）
    pub fn max_offset(&self) -> usize {
        let state = self.state.borrow();
        state.total_lines.saturating_sub(state.viewport_lines)
    }

    /// 获取当前显示偏移
    pub fn display_offset(&self) -> usize {
        self.state.borrow().display_offset
    }

    /// 获取视口行数
    pub fn viewport_lines(&self) -> usize {
        self.state.borrow().viewport_lines
    }

    /// 获取总行数
    pub fn total_lines(&self) -> usize {
        self.state.borrow().total_lines
    }

    /// 获取行高
    pub fn line_height(&self) -> Pixels {
        self.state.borrow().line_height
    }

    /// 计算滑块在轨道中的位置比例 (0.0 = 顶部, 1.0 = 底部)
    pub fn thumb_position_ratio(&self) -> f32 {
        let max = self.max_offset();
        if max == 0 {
            return 0.0;
        }
        // display_offset=0 表示在最底部，display_offset=max 表示在最顶部
        // 位置比例 = 1.0 - (display_offset / max)
        1.0 - (self.display_offset() as f32 / max as f32)
    }

    /// 计算滑块高度比例 (viewport / total)
    pub fn thumb_height_ratio(&self) -> f32 {
        let state = self.state.borrow();
        if state.total_lines == 0 {
            return 1.0;
        }
        (state.viewport_lines as f32 / state.total_lines as f32).clamp(0.05, 1.0)
    }

    /// 计算滑块边界
    pub fn thumb_bounds(&self, track_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        let thumb_height_ratio = self.thumb_height_ratio();
        let thumb_height = (track_bounds.size.height * thumb_height_ratio).max(THUMB_MIN_HEIGHT);

        let scrollable_height = track_bounds.size.height - thumb_height;
        let thumb_position_ratio = self.thumb_position_ratio();
        let thumb_top = track_bounds.origin.y + scrollable_height * thumb_position_ratio;

        Bounds {
            origin: Point::new(track_bounds.origin.x + SCROLLBAR_PADDING, thumb_top),
            size: gpui::Size {
                width: track_bounds.size.width - SCROLLBAR_PADDING * 2.0,
                height: thumb_height,
            },
        }
    }

    /// 根据鼠标Y位置计算新的 display_offset
    pub fn offset_from_mouse_position(
        &self,
        mouse_y: Pixels,
        track_bounds: Bounds<Pixels>,
        drag_offset: Option<Pixels>,
    ) -> usize {
        let thumb_height_ratio = self.thumb_height_ratio();
        let thumb_height = (track_bounds.size.height * thumb_height_ratio).max(THUMB_MIN_HEIGHT);
        let scrollable_height = track_bounds.size.height - thumb_height;

        if scrollable_height <= px(0.0) {
            return 0;
        }

        // 计算滑块顶部应该在的位置
        let thumb_top = match drag_offset {
            Some(offset) => mouse_y - offset,
            None => mouse_y - thumb_height / 2.0, // 点击轨道时，让滑块中心对准点击位置
        };

        // 计算位置比例
        let relative_top = thumb_top - track_bounds.origin.y;
        let position_ratio =
            (f32::from(relative_top) / f32::from(scrollable_height)).clamp(0.0, 1.0);

        // 转换为 display_offset
        // position_ratio=0 表示在顶部 -> display_offset=max
        // position_ratio=1 表示在底部 -> display_offset=0
        let max_offset = self.max_offset();
        ((1.0 - position_ratio) * max_offset as f32).round() as usize
    }

    /// 开始拖拽
    pub fn start_drag(&self, mouse_y: Pixels, track_bounds: Bounds<Pixels>) {
        let thumb_bounds = self.thumb_bounds(track_bounds);
        let offset_in_thumb = mouse_y - thumb_bounds.origin.y;
        self.thumb_state
            .set(ThumbState::Dragging { offset_in_thumb });
    }

    /// 更新拖拽
    pub fn update_drag(&self, mouse_y: Pixels, track_bounds: Bounds<Pixels>) {
        if let ThumbState::Dragging { offset_in_thumb } = self.thumb_state.get() {
            let new_offset =
                self.offset_from_mouse_position(mouse_y, track_bounds, Some(offset_in_thumb));
            self.future_display_offset.set(Some(new_offset));
        }
    }

    /// 结束拖拽
    pub fn end_drag(&self) {
        self.thumb_state.set(ThumbState::Inactive);
    }

    /// 点击轨道跳转
    pub fn jump_to_position(&self, mouse_y: Pixels, track_bounds: Bounds<Pixels>) {
        let new_offset = self.offset_from_mouse_position(mouse_y, track_bounds, None);
        self.future_display_offset.set(Some(new_offset));
    }

    /// 设置悬停状态
    pub fn set_hovered(&self, hovered: bool) {
        let current = self.thumb_state.get();
        match current {
            ThumbState::Dragging { .. } => {} // 拖拽中不改变状态
            _ => {
                self.thumb_state.set(if hovered {
                    ThumbState::Hovered
                } else {
                    ThumbState::Inactive
                });
            }
        }
    }

    /// 检查点是否在滑块内
    pub fn is_point_in_thumb(&self, point: Point<Pixels>, track_bounds: Bounds<Pixels>) -> bool {
        let thumb_bounds = self.thumb_bounds(track_bounds);
        thumb_bounds.contains(&point)
    }

    /// 检查点是否在轨道内
    pub fn is_point_in_track(&self, point: Point<Pixels>, track_bounds: Bounds<Pixels>) -> bool {
        track_bounds.contains(&point)
    }

    /// 获取滑块颜色
    pub fn thumb_color(&self, base_color: Hsla, hover_color: Hsla, active_color: Hsla) -> Hsla {
        match self.thumb_state.get() {
            ThumbState::Inactive => base_color,
            ThumbState::Hovered => hover_color,
            ThumbState::Dragging { .. } => active_color,
        }
    }
}
