//! 终端单元格（Cell）
//!
//! 参考 WezTerm 的 Cell 设计

use super::attributes::CellAttributes;
use super::color::Color;
use serde::{Deserialize, Serialize};

/// 终端单元格
///
/// 包含一个字符（grapheme cluster）及其显示属性
///
/// 注意：WezTerm 使用 TeenyString 优化小字符串存储，
/// 这里简化为使用 String。后续可以优化。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    /// 字符内容（UTF-8 grapheme cluster）
    ///
    /// 可以是：
    /// - 单个 ASCII 字符 (e.g., "a")
    /// - 单个 Unicode 字符 (e.g., "中")
    /// - 组合字符 (e.g., "é" = "e" + combining acute)
    /// - Emoji (e.g., "👍")
    /// - 双宽字符 (e.g., "中" 占 2 个 cell 宽度)
    text: String,

    /// 显示属性
    pub attrs: CellAttributes,

    /// 字符宽度（在终端中占据的 cell 数量）
    ///
    /// - 1: 正常宽度字符
    /// - 2: 双宽字符（如中文、日文、全宽符号）
    /// - 0: 组合字符或零宽字符
    width: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank()
    }
}

impl Cell {
    /// 创建空白单元格
    pub fn blank() -> Self {
        Self {
            text: String::from(" "),
            attrs: CellAttributes::default(),
            width: 1,
        }
    }

    /// 创建空白单元格（带自定义属性）
    pub fn blank_with_attrs(attrs: CellAttributes) -> Self {
        Self {
            text: String::from(" "),
            attrs,
            width: 1,
        }
    }

    /// 创建带文本的单元格（自动计算宽度）
    pub fn new(text: &str) -> Self {
        use unicode_width::UnicodeWidthStr;
        let width = text.width() as u8;
        Self {
            text: text.to_string(),
            attrs: CellAttributes::default(),
            width: if width == 0 { 1 } else { width },
        }
    }

    /// 创建带属性的单元格（自动计算宽度）
    pub fn with_attrs_auto(text: &str, attrs: CellAttributes) -> Self {
        use unicode_width::UnicodeWidthStr;
        let width = text.width() as u8;
        Self {
            text: text.to_string(),
            attrs,
            width: if width == 0 { 1 } else { width },
        }
    }

    /// 创建带属性和指定宽度的单元格
    pub fn new_with_attrs(text: &str, attrs: CellAttributes, width: u8) -> Self {
        Self {
            text: text.to_string(),
            attrs,
            width,
        }
    }

    /// 获取文本内容
    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 获取文本内容（别名，兼容旧代码）
    #[inline]
    pub fn str(&self) -> &str {
        &self.text
    }

    /// 设置文本内容
    #[inline]
    pub fn set_text(&mut self, text: &str) {
        use unicode_width::UnicodeWidthStr;
        self.text = text.to_string();
        let width = text.width() as u8;
        self.width = if width == 0 { 1 } else { width };
    }

    /// 设置文本内容（带指定宽度）
    #[inline]
    pub fn set_str(&mut self, text: String, width: u8) {
        self.text = text;
        self.width = width;
    }

    /// 获取字符宽度
    #[inline]
    pub fn width(&self) -> u8 {
        self.width
    }

    /// 设置字符宽度
    #[inline]
    pub fn set_width(&mut self, width: u8) {
        self.width = width;
    }

    /// 是否是空白单元格
    pub fn is_blank(&self) -> bool {
        self.text == " " && self.attrs == CellAttributes::default()
    }

    /// 是否是双宽字符
    #[inline]
    pub fn is_double_width(&self) -> bool {
        self.width >= 2
    }

    /// 重置为空白单元格
    pub fn reset(&mut self) {
        self.text = String::from(" ");
        self.attrs = CellAttributes::default();
        self.width = 1;
    }

    /// 重置为指定属性的空白单元格
    pub fn reset_with_attrs(&mut self, attrs: CellAttributes) {
        self.text = String::from(" ");
        self.attrs = attrs;
        self.width = 1;
    }

    /// 克隆单元格（深拷贝）
    pub fn clone_cell(&self) -> Self {
        Self {
            text: self.text.clone(),
            attrs: self.attrs.clone(),
            width: self.width,
        }
    }

    /// 创建带特定前景色的单元格
    pub fn with_foreground(text: &str, color: Color) -> Self {
        let mut cell = Self::new(text);
        cell.attrs.foreground = color;
        cell
    }

    /// 创建带特定背景色的单元格
    pub fn with_background(text: &str, color: Color) -> Self {
        let mut cell = Self::new(text);
        cell.attrs.background = color;
        cell
    }

    /// 从字符创建单元格
    pub fn from_char(c: char) -> Self {
        Self::new(&c.to_string())
    }

    /// 从字符和属性创建单元格
    pub fn from_char_with_attrs(c: char, attrs: CellAttributes) -> Self {
        Self::with_attrs_auto(&c.to_string(), attrs)
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
    use crate::grid::cell_attrs::SemanticType;

    #[test]
    fn test_blank_cell() {
        let cell = Cell::blank();
        assert_eq!(cell.str(), " ");
        assert_eq!(cell.width(), 1);
        assert!(cell.is_blank());
        assert!(!cell.is_double_width());
    }

    #[test]
    fn test_new_cell() {
        let cell = Cell::new("a");
        assert_eq!(cell.text(), "a");
        assert_eq!(cell.width(), 1);
        assert!(!cell.is_blank());
    }

    #[test]
    fn test_double_width() {
        let cell = Cell::new("中");
        assert_eq!(cell.text(), "中");
        assert_eq!(cell.width(), 2);
        assert!(cell.is_double_width());
    }

    #[test]
    fn test_with_attrs() {
        let mut attrs = CellAttributes::default();
        attrs.set_bold(true);
        attrs.foreground = Color::Indexed(1); // RED

        let cell = Cell::with_attrs_auto("a", attrs.clone());
        assert_eq!(cell.text(), "a");
        assert!(cell.attrs.is_bold());
        assert_eq!(cell.attrs.foreground, Color::Indexed(1));
    }

    #[test]
    fn test_reset() {
        let mut cell = Cell::new("a");
        cell.attrs.set_bold(true);

        cell.reset();
        assert_eq!(cell.text(), " ");
        assert!(!cell.attrs.is_bold());
        assert!(cell.is_blank());
    }

    #[test]
    fn test_reset_with_attrs() {
        let mut attrs = CellAttributes::default();
        attrs.background = Color::Indexed(4); // BLUE

        let mut cell = Cell::new("a");
        cell.reset_with_attrs(attrs.clone());

        assert_eq!(cell.text(), " ");
        assert_eq!(cell.attrs.background, Color::Indexed(4));
    }

    #[test]
    fn test_from_char() {
        let cell_a = Cell::from_char('a');
        assert_eq!(cell_a.text(), "a");
        assert_eq!(cell_a.width(), 1);

        let cell_chinese = Cell::from_char('中');
        assert_eq!(cell_chinese.text(), "中");
        assert_eq!(cell_chinese.width(), 2);
    }

    #[test]
    fn test_semantic_type() {
        let mut attrs = CellAttributes::default();
        attrs.set_semantic_type(SemanticType::Prompt);

        let cell = Cell::with_attrs_auto("$", attrs);
        assert_eq!(cell.attrs.semantic_type(), SemanticType::Prompt);
    }

    #[test]
    fn test_clone_cell() {
        let cell1 = Cell::from_char('x');
        let cell2 = cell1.clone_cell();

        assert_eq!(cell1.text(), cell2.text());
        assert_eq!(cell1.width(), cell2.width());
    }

    #[test]
    fn test_with_foreground() {
        let cell = Cell::with_foreground("a", Color::Indexed(2)); // GREEN
        assert_eq!(cell.attrs.foreground, Color::Indexed(2));
    }
}
