//! Line 标志位
//!
//! 参考 WezTerm 的 LineBits 设计

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Line 特殊标记（使用位字段优化）
    ///
    /// 对应 WezTerm 的 LineBits
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct LineBits: u16 {
        /// 包含超链接
        const HAS_HYPERLINK = 1 << 1;

        /// 已扫描隐式超链接
        const SCANNED_IMPLICIT_HYPERLINKS = 1 << 2;

        /// 包含隐式超链接
        const HAS_IMPLICIT_HYPERLINKS = 1 << 3;

        /// 双宽行（VT100 DECSWL）
        const DOUBLE_WIDTH = 1 << 4;

        /// 双高行 - 上半部分（VT100 DECDHL）
        const DOUBLE_HEIGHT_TOP = 1 << 5;

        /// 双高行 - 下半部分（VT100 DECDHL）
        const DOUBLE_HEIGHT_BOTTOM = 1 << 6;

        /// BiDi（双向文本）已启用
        const BIDI_ENABLED = 1 << 0;

        /// 从右到左（RTL）文本方向
        const RTL = 1 << 7;

        /// 自动检测文本方向
        const AUTO_DETECT_DIRECTION = 1 << 8;

        /// 行尾被包装（换行）
        const WRAPPED = 1 << 9;
    }
}

impl Default for LineBits {
    fn default() -> Self {
        Self::empty()
    }
}

impl LineBits {
    /// 创建空标志
    #[inline]
    pub fn new() -> Self {
        Self::empty()
    }

    /// 是否有超链接
    #[inline]
    pub fn has_hyperlink(&self) -> bool {
        self.contains(Self::HAS_HYPERLINK)
    }

    /// 设置超链接标志
    #[inline]
    pub fn set_has_hyperlink(&mut self, value: bool) {
        self.set(Self::HAS_HYPERLINK, value);
    }

    /// 是否已扫描隐式超链接
    #[inline]
    pub fn scanned_implicit_hyperlinks(&self) -> bool {
        self.contains(Self::SCANNED_IMPLICIT_HYPERLINKS)
    }

    /// 设置已扫描隐式超链接标志
    #[inline]
    pub fn set_scanned_implicit_hyperlinks(&mut self, value: bool) {
        self.set(Self::SCANNED_IMPLICIT_HYPERLINKS, value);
    }

    /// 是否有隐式超链接
    #[inline]
    pub fn has_implicit_hyperlinks(&self) -> bool {
        self.contains(Self::HAS_IMPLICIT_HYPERLINKS)
    }

    /// 设置隐式超链接标志
    #[inline]
    pub fn set_has_implicit_hyperlinks(&mut self, value: bool) {
        self.set(Self::HAS_IMPLICIT_HYPERLINKS, value);
    }

    /// 是否是双宽行
    #[inline]
    pub fn is_double_width(&self) -> bool {
        self.contains(Self::DOUBLE_WIDTH)
    }

    /// 设置双宽标志
    #[inline]
    pub fn set_double_width(&mut self, value: bool) {
        self.set(Self::DOUBLE_WIDTH, value);
    }

    /// 是否是双高行（上半部分）
    #[inline]
    pub fn is_double_height_top(&self) -> bool {
        self.contains(Self::DOUBLE_HEIGHT_TOP)
    }

    /// 设置双高上半部分标志
    #[inline]
    pub fn set_double_height_top(&mut self, value: bool) {
        self.set(Self::DOUBLE_HEIGHT_TOP, value);
    }

    /// 是否是双高行（下半部分）
    #[inline]
    pub fn is_double_height_bottom(&self) -> bool {
        self.contains(Self::DOUBLE_HEIGHT_BOTTOM)
    }

    /// 设置双高下半部分标志
    #[inline]
    pub fn set_double_height_bottom(&mut self, value: bool) {
        self.set(Self::DOUBLE_HEIGHT_BOTTOM, value);
    }

    /// BiDi 是否已启用
    #[inline]
    pub fn bidi_enabled(&self) -> bool {
        self.contains(Self::BIDI_ENABLED)
    }

    /// 设置 BiDi 启用标志
    #[inline]
    pub fn set_bidi_enabled(&mut self, value: bool) {
        self.set(Self::BIDI_ENABLED, value);
    }

    /// 是否是从右到左文本
    #[inline]
    pub fn is_rtl(&self) -> bool {
        self.contains(Self::RTL)
    }

    /// 设置 RTL 标志
    #[inline]
    pub fn set_rtl(&mut self, value: bool) {
        self.set(Self::RTL, value);
    }

    /// 是否自动检测方向
    #[inline]
    pub fn auto_detect_direction(&self) -> bool {
        self.contains(Self::AUTO_DETECT_DIRECTION)
    }

    /// 设置自动检测方向标志
    #[inline]
    pub fn set_auto_detect_direction(&mut self, value: bool) {
        self.set(Self::AUTO_DETECT_DIRECTION, value);
    }

    /// 行尾是否被包装
    #[inline]
    pub fn is_wrapped(&self) -> bool {
        self.contains(Self::WRAPPED)
    }

    /// 设置包装标志
    #[inline]
    pub fn set_wrapped(&mut self, value: bool) {
        self.set(Self::WRAPPED, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_linebits() {
        let bits = LineBits::default();
        assert!(bits.is_empty());
        assert!(!bits.has_hyperlink());
        assert!(!bits.is_wrapped());
        assert!(!bits.is_double_width());
    }

    #[test]
    fn test_hyperlink_flags() {
        let mut bits = LineBits::new();
        assert!(!bits.has_hyperlink());

        bits.set_has_hyperlink(true);
        assert!(bits.has_hyperlink());

        bits.set_has_hyperlink(false);
        assert!(!bits.has_hyperlink());
    }

    #[test]
    fn test_implicit_hyperlinks() {
        let mut bits = LineBits::new();

        bits.set_scanned_implicit_hyperlinks(true);
        assert!(bits.scanned_implicit_hyperlinks());

        bits.set_has_implicit_hyperlinks(true);
        assert!(bits.has_implicit_hyperlinks());
        assert!(bits.scanned_implicit_hyperlinks());
    }

    #[test]
    fn test_double_width_height() {
        let mut bits = LineBits::new();

        bits.set_double_width(true);
        assert!(bits.is_double_width());

        bits.set_double_height_top(true);
        assert!(bits.is_double_height_top());
        assert!(!bits.is_double_height_bottom());

        bits.set_double_height_bottom(true);
        assert!(bits.is_double_height_bottom());
    }

    #[test]
    fn test_bidi_flags() {
        let mut bits = LineBits::new();

        bits.set_bidi_enabled(true);
        assert!(bits.bidi_enabled());

        bits.set_rtl(true);
        assert!(bits.is_rtl());

        bits.set_auto_detect_direction(true);
        assert!(bits.auto_detect_direction());
    }

    #[test]
    fn test_wrapped_flag() {
        let mut bits = LineBits::new();
        assert!(!bits.is_wrapped());

        bits.set_wrapped(true);
        assert!(bits.is_wrapped());

        bits.set_wrapped(false);
        assert!(!bits.is_wrapped());
    }

    #[test]
    fn test_multiple_flags() {
        let mut bits = LineBits::new();
        bits.set_has_hyperlink(true);
        bits.set_wrapped(true);
        bits.set_double_width(true);

        assert!(bits.has_hyperlink());
        assert!(bits.is_wrapped());
        assert!(bits.is_double_width());
        assert!(!bits.is_rtl());
    }

    #[test]
    fn test_bitflags_operations() {
        let mut bits = LineBits::HAS_HYPERLINK | LineBits::WRAPPED;
        assert!(bits.contains(LineBits::HAS_HYPERLINK));
        assert!(bits.contains(LineBits::WRAPPED));
        assert!(!bits.contains(LineBits::DOUBLE_WIDTH));

        bits.remove(LineBits::HAS_HYPERLINK);
        assert!(!bits.contains(LineBits::HAS_HYPERLINK));
        assert!(bits.contains(LineBits::WRAPPED));
    }

    #[test]
    fn test_memory_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<LineBits>(), 2); // u16 = 2 bytes
    }
}
