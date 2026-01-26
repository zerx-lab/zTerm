//! 光标（Cursor）结构
//!
//! 对应 WezTerm 的 Cursor

use serde::{Deserialize, Serialize};

use super::CellAttributes;

/// 光标形状
///
/// 对应 VT 序列定义的光标形状
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorShape {
    /// 块状光标（默认）
    Block,
    /// 下划线光标
    Underline,
    /// 竖线光标（I-beam）
    Bar,
}

impl Default for CursorShape {
    fn default() -> Self {
        Self::Block
    }
}

/// 光标
///
/// 对应 WezTerm 的 Cursor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    /// 光标 X 坐标（列）
    pub x: usize,
    /// 光标 Y 坐标（行，视口相对）
    pub y: usize,
    /// 光标形状
    pub shape: CursorShape,
    /// 光标是否可见
    pub visible: bool,
    /// 光标是否闪烁
    pub blink: bool,
    /// 当前属性（用于新输入的字符）
    pub attrs: CellAttributes,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            shape: CursorShape::default(),
            visible: true,
            blink: true,
            attrs: CellAttributes::default(),
        }
    }
}

impl Cursor {
    /// 创建新光标
    pub fn new() -> Self {
        Self::default()
    }

    /// 移动光标到指定位置
    #[inline]
    pub fn set_position(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    /// 获取光标位置
    #[inline]
    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    /// 设置光标形状
    #[inline]
    pub fn set_shape(&mut self, shape: CursorShape) {
        self.shape = shape;
    }

    /// 设置光标可见性
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 设置光标闪烁
    #[inline]
    pub fn set_blink(&mut self, blink: bool) {
        self.blink = blink;
    }

    /// 向右移动 n 列
    pub fn move_right(&mut self, cols: usize, max_cols: usize) {
        self.x = (self.x + cols).min(max_cols.saturating_sub(1));
    }

    /// 向左移动 n 列
    pub fn move_left(&mut self, cols: usize) {
        self.x = self.x.saturating_sub(cols);
    }

    /// 向下移动 n 行
    pub fn move_down(&mut self, rows: usize, max_rows: usize) {
        self.y = (self.y + rows).min(max_rows.saturating_sub(1));
    }

    /// 向上移动 n 行
    pub fn move_up(&mut self, rows: usize) {
        self.y = self.y.saturating_sub(rows);
    }

    /// 移动到行首
    #[inline]
    pub fn carriage_return(&mut self) {
        self.x = 0;
    }

    /// 换行（移动到下一行）
    pub fn line_feed(&mut self, max_rows: usize) {
        if self.y + 1 < max_rows {
            self.y += 1;
        }
    }

    /// 回到主屏幕位置 (0, 0)
    #[inline]
    pub fn home(&mut self) {
        self.x = 0;
        self.y = 0;
    }

    /// 限制光标在视口内
    pub fn clamp(&mut self, max_cols: usize, max_rows: usize) {
        self.x = self.x.min(max_cols.saturating_sub(1));
        self.y = self.y.min(max_rows.saturating_sub(1));
    }

    /// 重置光标到默认状态
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 获取属性引用
    #[inline]
    pub fn attrs(&self) -> &CellAttributes {
        &self.attrs
    }

    /// 获取可变属性引用
    #[inline]
    pub fn attrs_mut(&mut self) -> &mut CellAttributes {
        &mut self.attrs
    }

    /// 设置属性
    #[inline]
    pub fn set_attrs(&mut self, attrs: CellAttributes) {
        self.attrs = attrs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_default() {
        let cursor = Cursor::default();
        assert_eq!(cursor.x, 0);
        assert_eq!(cursor.y, 0);
        assert_eq!(cursor.shape, CursorShape::Block);
        assert!(cursor.visible);
        assert!(cursor.blink);
    }

    #[test]
    fn test_cursor_set_position() {
        let mut cursor = Cursor::new();
        cursor.set_position(10, 5);
        assert_eq!(cursor.position(), (10, 5));
    }

    #[test]
    fn test_cursor_move_right() {
        let mut cursor = Cursor::new();
        cursor.move_right(5, 80);
        assert_eq!(cursor.x, 5);

        cursor.move_right(100, 80);
        assert_eq!(cursor.x, 79); // 限制在 max_cols - 1
    }

    #[test]
    fn test_cursor_move_left() {
        let mut cursor = Cursor::new();
        cursor.x = 10;

        cursor.move_left(3);
        assert_eq!(cursor.x, 7);

        cursor.move_left(100);
        assert_eq!(cursor.x, 0); // 不会变成负数
    }

    #[test]
    fn test_cursor_move_down() {
        let mut cursor = Cursor::new();
        cursor.move_down(5, 24);
        assert_eq!(cursor.y, 5);

        cursor.move_down(100, 24);
        assert_eq!(cursor.y, 23); // 限制在 max_rows - 1
    }

    #[test]
    fn test_cursor_move_up() {
        let mut cursor = Cursor::new();
        cursor.y = 10;

        cursor.move_up(3);
        assert_eq!(cursor.y, 7);

        cursor.move_up(100);
        assert_eq!(cursor.y, 0);
    }

    #[test]
    fn test_cursor_carriage_return() {
        let mut cursor = Cursor::new();
        cursor.x = 50;

        cursor.carriage_return();
        assert_eq!(cursor.x, 0);
    }

    #[test]
    fn test_cursor_line_feed() {
        let mut cursor = Cursor::new();
        cursor.line_feed(24);
        assert_eq!(cursor.y, 1);

        cursor.y = 23;
        cursor.line_feed(24);
        assert_eq!(cursor.y, 23); // 不超过底部
    }

    #[test]
    fn test_cursor_home() {
        let mut cursor = Cursor::new();
        cursor.x = 50;
        cursor.y = 10;

        cursor.home();
        assert_eq!(cursor.position(), (0, 0));
    }

    #[test]
    fn test_cursor_clamp() {
        let mut cursor = Cursor::new();
        cursor.x = 100;
        cursor.y = 100;

        cursor.clamp(80, 24);
        assert_eq!(cursor.x, 79);
        assert_eq!(cursor.y, 23);
    }

    #[test]
    fn test_cursor_reset() {
        let mut cursor = Cursor::new();
        cursor.x = 50;
        cursor.y = 10;
        cursor.visible = false;

        cursor.reset();
        assert_eq!(cursor.x, 0);
        assert_eq!(cursor.y, 0);
        assert!(cursor.visible);
    }

    #[test]
    fn test_cursor_shape() {
        let mut cursor = Cursor::new();
        assert_eq!(cursor.shape, CursorShape::Block);

        cursor.set_shape(CursorShape::Bar);
        assert_eq!(cursor.shape, CursorShape::Bar);

        cursor.set_shape(CursorShape::Underline);
        assert_eq!(cursor.shape, CursorShape::Underline);
    }

    #[test]
    fn test_cursor_visibility() {
        let mut cursor = Cursor::new();
        assert!(cursor.visible);

        cursor.set_visible(false);
        assert!(!cursor.visible);
    }

    #[test]
    fn test_cursor_attrs() {
        let mut cursor = Cursor::new();
        let mut attrs = CellAttributes::default();
        attrs.set_bold(true);

        cursor.set_attrs(attrs.clone());
        assert_eq!(cursor.attrs(), &attrs);
        assert!(cursor.attrs().is_bold());
    }
}
