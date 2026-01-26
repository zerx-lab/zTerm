//! Cell 属性完整实现
//!
//! 参考 WezTerm 的 CellAttributes 设计，使用位字段优化存储

use super::cell_attrs::{CellFlags, Intensity, SemanticType, UnderlineStyle};
use super::color::Color;
use serde::{Deserialize, Serialize};

/// Cell 属性
///
/// 参考 WezTerm 的设计：
/// - 使用 u32 位字段存储常见属性（紧凑）
/// - 前景色和背景色独立存储
/// - 扩展属性（超链接、图像等）延迟分配
///
/// 位字段布局（32 位）：
/// - Bit 0-1:   Intensity (2 bits)
/// - Bit 2-4:   UnderlineStyle (3 bits)
/// - Bit 5-6:   Blink (2 bits)
/// - Bit 7:     Italic
/// - Bit 8:     Reverse
/// - Bit 9:     StrikeThrough
/// - Bit 10:    Invisible
/// - Bit 11:    Wrapped
/// - Bit 12:    Overline
/// - Bit 13-14: SemanticType (2 bits)
/// - Bit 15-31: 保留
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellAttributes {
    /// 位字段存储的属性
    attributes: u32,

    /// 前景色
    pub foreground: Color,

    /// 背景色
    pub background: Color,

    /// 下划线颜色（可选）
    pub underline_color: Option<Color>,
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self {
            attributes: 0,
            foreground: Color::DefaultForeground,
            background: Color::DefaultBackground,
            underline_color: None,
        }
    }
}

// 位字段操作宏
macro_rules! bitfield_getter {
    ($name:ident, $type:ty, $mask:expr, $shift:expr) => {
        #[inline]
        pub fn $name(&self) -> $type {
            <$type>::from_u8(((self.attributes >> $shift) & $mask) as u8)
        }
    };
}

macro_rules! bitfield_setter {
    ($name:ident, $type:ty, $mask:expr, $shift:expr) => {
        #[inline]
        pub fn $name(&mut self, value: $type) {
            self.attributes =
                (self.attributes & !($mask << $shift)) | ((value.to_u8() as u32 & $mask) << $shift);
        }
    };
}

macro_rules! flag_getter {
    ($name:ident, $bit:expr) => {
        #[inline]
        pub fn $name(&self) -> bool {
            (self.attributes & (1 << $bit)) != 0
        }
    };
}

macro_rules! flag_setter {
    ($name:ident, $bit:expr) => {
        #[inline]
        pub fn $name(&mut self, value: bool) {
            if value {
                self.attributes |= 1 << $bit;
            } else {
                self.attributes &= !(1 << $bit);
            }
        }
    };
}

impl CellAttributes {
    /// 创建默认属性
    pub fn new() -> Self {
        Self::default()
    }

    // ========== Intensity ==========
    bitfield_getter!(intensity, Intensity, 0b11, 0);
    bitfield_setter!(set_intensity, Intensity, 0b11, 0);

    // ========== Underline Style ==========
    bitfield_getter!(underline, UnderlineStyle, 0b111, 2);
    bitfield_setter!(set_underline, UnderlineStyle, 0b111, 2);

    // ========== SemanticType ==========
    bitfield_getter!(semantic_type, SemanticType, 0b11, 13);
    bitfield_setter!(set_semantic_type, SemanticType, 0b11, 13);

    // ========== Boolean Flags ==========
    flag_getter!(italic, 7);
    flag_setter!(set_italic, 7);

    flag_getter!(reverse, 8);
    flag_setter!(set_reverse, 8);

    flag_getter!(strikethrough, 9);
    flag_setter!(set_strikethrough, 9);

    flag_getter!(invisible, 10);
    flag_setter!(set_invisible, 10);

    flag_getter!(wrapped, 11);
    flag_setter!(set_wrapped, 11);

    flag_getter!(overline, 12);
    flag_setter!(set_overline, 12);

    // ========== 便捷方法 ==========

    /// 是否是粗体
    #[inline]
    pub fn is_bold(&self) -> bool {
        self.intensity() == Intensity::Bold
    }

    /// 设置粗体
    #[inline]
    pub fn set_bold(&mut self, bold: bool) {
        self.set_intensity(if bold {
            Intensity::Bold
        } else {
            Intensity::Normal
        });
    }

    /// 是否是暗淡
    #[inline]
    pub fn is_dim(&self) -> bool {
        self.intensity() == Intensity::Dim
    }

    /// 设置暗淡
    #[inline]
    pub fn set_dim(&mut self, dim: bool) {
        self.set_intensity(if dim { Intensity::Dim } else { Intensity::Normal });
    }

    /// 是否有下划线
    #[inline]
    pub fn has_underline(&self) -> bool {
        self.underline() != UnderlineStyle::None
    }

    /// 设置前景色
    #[inline]
    pub fn set_foreground(&mut self, color: Color) {
        self.foreground = color;
    }

    /// 设置背景色
    #[inline]
    pub fn set_background(&mut self, color: Color) {
        self.background = color;
    }

    /// 设置下划线颜色
    #[inline]
    pub fn set_underline_color(&mut self, color: Option<Color>) {
        self.underline_color = color;
    }

    /// 转换为 CellFlags（用于兼容）
    pub fn to_flags(&self) -> CellFlags {
        let mut flags = CellFlags::empty();

        if self.is_bold() {
            flags.insert(CellFlags::BOLD);
        }
        if self.is_dim() {
            flags.insert(CellFlags::DIM);
        }
        if self.italic() {
            flags.insert(CellFlags::ITALIC);
        }
        if self.has_underline() {
            flags.insert(CellFlags::UNDERLINE);
            if self.underline() == UnderlineStyle::Double {
                flags.insert(CellFlags::DOUBLE_UNDERLINE);
            }
        }
        if self.reverse() {
            flags.insert(CellFlags::REVERSE);
        }
        if self.strikethrough() {
            flags.insert(CellFlags::STRIKETHROUGH);
        }
        if self.invisible() {
            flags.insert(CellFlags::INVISIBLE);
        }
        if self.overline() {
            flags.insert(CellFlags::OVERLINE);
        }
        if self.wrapped() {
            flags.insert(CellFlags::WRAPPED);
        }

        flags
    }

    /// 从 CellFlags 设置属性
    pub fn from_flags(flags: CellFlags) -> Self {
        let mut attrs = Self::default();

        if flags.contains(CellFlags::BOLD) {
            attrs.set_bold(true);
        }
        if flags.contains(CellFlags::DIM) {
            attrs.set_dim(true);
        }
        if flags.contains(CellFlags::ITALIC) {
            attrs.set_italic(true);
        }
        if flags.contains(CellFlags::UNDERLINE) {
            attrs.set_underline(if flags.contains(CellFlags::DOUBLE_UNDERLINE) {
                UnderlineStyle::Double
            } else {
                UnderlineStyle::Single
            });
        }
        if flags.contains(CellFlags::REVERSE) {
            attrs.set_reverse(true);
        }
        if flags.contains(CellFlags::STRIKETHROUGH) {
            attrs.set_strikethrough(true);
        }
        if flags.contains(CellFlags::INVISIBLE) {
            attrs.set_invisible(true);
        }
        if flags.contains(CellFlags::OVERLINE) {
            attrs.set_overline(true);
        }
        if flags.contains(CellFlags::WRAPPED) {
            attrs.set_wrapped(true);
        }

        attrs
    }

    /// 重置为默认属性（保留颜色）
    pub fn reset(&mut self) {
        self.attributes = 0;
        self.underline_color = None;
    }

    /// 重置所有（包括颜色）
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }

    /// 克隆 SGR 属性（仅文本属性，不包括语义类型和包装标志）
    ///
    /// 用于清屏操作时保留背景色
    pub fn clone_sgr_only(&self) -> Self {
        let mut result = Self {
            attributes: self.attributes,
            foreground: self.foreground,
            background: self.background,
            underline_color: self.underline_color,
        };

        // 清除非 SGR 属性
        result.set_semantic_type(SemanticType::default());
        result.set_wrapped(false);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_attributes() {
        let attrs = CellAttributes::default();
        assert_eq!(attrs.intensity(), Intensity::Normal);
        assert_eq!(attrs.underline(), UnderlineStyle::None);
        assert_eq!(attrs.semantic_type(), SemanticType::Output);
        assert!(!attrs.italic());
        assert!(!attrs.reverse());
        assert_eq!(attrs.foreground, Color::DefaultForeground);
        assert_eq!(attrs.background, Color::DefaultBackground);
    }

    #[test]
    fn test_intensity() {
        let mut attrs = CellAttributes::new();
        assert!(!attrs.is_bold());

        attrs.set_bold(true);
        assert!(attrs.is_bold());
        assert_eq!(attrs.intensity(), Intensity::Bold);

        attrs.set_dim(true);
        assert!(attrs.is_dim());
        assert!(!attrs.is_bold());
    }

    #[test]
    fn test_underline() {
        let mut attrs = CellAttributes::new();
        assert!(!attrs.has_underline());

        attrs.set_underline(UnderlineStyle::Single);
        assert!(attrs.has_underline());
        assert_eq!(attrs.underline(), UnderlineStyle::Single);

        attrs.set_underline(UnderlineStyle::Curly);
        assert_eq!(attrs.underline(), UnderlineStyle::Curly);
    }

    #[test]
    fn test_flags() {
        let mut attrs = CellAttributes::new();

        attrs.set_italic(true);
        assert!(attrs.italic());

        attrs.set_reverse(true);
        assert!(attrs.reverse());
        assert!(attrs.italic());

        attrs.set_italic(false);
        assert!(!attrs.italic());
        assert!(attrs.reverse());
    }

    #[test]
    fn test_semantic_type() {
        let mut attrs = CellAttributes::new();
        assert_eq!(attrs.semantic_type(), SemanticType::Output);

        attrs.set_semantic_type(SemanticType::Prompt);
        assert_eq!(attrs.semantic_type(), SemanticType::Prompt);

        attrs.set_semantic_type(SemanticType::Input);
        assert_eq!(attrs.semantic_type(), SemanticType::Input);
    }

    #[test]
    fn test_colors() {
        let mut attrs = CellAttributes::new();
        attrs.foreground = Color::RED;
        attrs.background = Color::BLUE;
        attrs.underline_color = Some(Color::GREEN);

        assert_eq!(attrs.foreground, Color::RED);
        assert_eq!(attrs.background, Color::BLUE);
        assert_eq!(attrs.underline_color, Some(Color::GREEN));
    }

    #[test]
    fn test_to_from_flags() {
        let flags = CellFlags::BOLD | CellFlags::ITALIC | CellFlags::UNDERLINE;
        let attrs = CellAttributes::from_flags(flags);

        assert!(attrs.is_bold());
        assert!(attrs.italic());
        assert!(attrs.has_underline());

        let converted_flags = attrs.to_flags();
        assert!(converted_flags.contains(CellFlags::BOLD));
        assert!(converted_flags.contains(CellFlags::ITALIC));
        assert!(converted_flags.contains(CellFlags::UNDERLINE));
    }

    #[test]
    fn test_reset() {
        let mut attrs = CellAttributes::new();
        attrs.set_bold(true);
        attrs.set_italic(true);
        attrs.foreground = Color::RED;

        attrs.reset();
        assert!(!attrs.is_bold());
        assert!(!attrs.italic());
        assert_eq!(attrs.foreground, Color::RED); // 颜色保留

        attrs.reset_all();
        assert_eq!(attrs.foreground, Color::DefaultForeground); // 颜色也重置
    }

    #[test]
    fn test_clone_sgr_only() {
        let mut attrs = CellAttributes::new();
        attrs.set_bold(true);
        attrs.foreground = Color::RED;
        attrs.background = Color::BLUE;
        attrs.set_semantic_type(SemanticType::Prompt);
        attrs.set_wrapped(true);

        let cloned = attrs.clone_sgr_only();
        assert!(cloned.is_bold());
        assert_eq!(cloned.foreground, Color::RED);
        assert_eq!(cloned.background, Color::BLUE);
        assert_eq!(cloned.semantic_type(), SemanticType::Output); // 语义类型被重置
        assert!(!cloned.wrapped()); // 包装标志被重置
    }

    #[test]
    fn test_bitfield_size() {
        // 验证结构体大小合理
        use std::mem::size_of;
        let size = size_of::<CellAttributes>();
        // u32 (4) + Color (2 * ?) + Option<Color> (? + 1)
        // 应该小于 64 字节
        assert!(size < 64, "CellAttributes size: {} bytes", size);
    }
}
