//! 终端颜色类型
//!
//! 参考 WezTerm 的颜色系统，支持：
//! - ANSI 16 色（0-15）
//! - ANSI 256 色（0-255）
//! - RGB 真彩色（24位）
//! - 默认前景/背景色

use serde::{Deserialize, Serialize};

/// 终端颜色
///
/// 参考 WezTerm 的 ColorAttribute 设计
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    /// 默认前景色（由主题/配置决定）
    DefaultForeground,

    /// 默认背景色（由主题/配置决定）
    DefaultBackground,

    /// ANSI 颜色索引（0-255）
    /// - 0-7: 标准颜色
    /// - 8-15: 高亮颜色
    /// - 16-255: 256 色调色板
    Indexed(u8),

    /// RGB 真彩色
    Rgb { r: u8, g: u8, b: u8 },
}

impl Default for Color {
    fn default() -> Self {
        Self::DefaultForeground
    }
}

impl Color {
    /// 创建 ANSI 标准颜色（0-7）
    pub const fn ansi(index: u8) -> Self {
        debug_assert!(index < 8, "Standard ANSI color must be 0-7");
        Self::Indexed(index)
    }

    /// 创建 ANSI 高亮颜色（8-15）
    pub const fn bright_ansi(index: u8) -> Self {
        debug_assert!(index < 8, "Bright ANSI color must be 0-7");
        Self::Indexed(index + 8)
    }

    /// 创建 RGB 颜色
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb { r, g, b }
    }

    /// 创建灰度颜色（ANSI 256 色调色板的 232-255）
    pub const fn gray(level: u8) -> Self {
        debug_assert!(level < 24, "Gray level must be 0-23");
        Self::Indexed(232 + level)
    }

    /// 是否是默认颜色
    pub const fn is_default(&self) -> bool {
        matches!(
            self,
            Self::DefaultForeground | Self::DefaultBackground
        )
    }
}

// ANSI 标准颜色常量
impl Color {
    pub const BLACK: Self = Self::ansi(0);
    pub const RED: Self = Self::ansi(1);
    pub const GREEN: Self = Self::ansi(2);
    pub const YELLOW: Self = Self::ansi(3);
    pub const BLUE: Self = Self::ansi(4);
    pub const MAGENTA: Self = Self::ansi(5);
    pub const CYAN: Self = Self::ansi(6);
    pub const WHITE: Self = Self::ansi(7);

    pub const BRIGHT_BLACK: Self = Self::bright_ansi(0);
    pub const BRIGHT_RED: Self = Self::bright_ansi(1);
    pub const BRIGHT_GREEN: Self = Self::bright_ansi(2);
    pub const BRIGHT_YELLOW: Self = Self::bright_ansi(3);
    pub const BRIGHT_BLUE: Self = Self::bright_ansi(4);
    pub const BRIGHT_MAGENTA: Self = Self::bright_ansi(5);
    pub const BRIGHT_CYAN: Self = Self::bright_ansi(6);
    pub const BRIGHT_WHITE: Self = Self::bright_ansi(7);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_default() {
        let color = Color::default();
        assert!(color.is_default());
        assert_eq!(color, Color::DefaultForeground);
    }

    #[test]
    fn test_ansi_colors() {
        assert_eq!(Color::BLACK, Color::Indexed(0));
        assert_eq!(Color::RED, Color::Indexed(1));
        assert_eq!(Color::BRIGHT_BLACK, Color::Indexed(8));
        assert_eq!(Color::BRIGHT_WHITE, Color::Indexed(15));
    }

    #[test]
    fn test_rgb_color() {
        let color = Color::rgb(255, 128, 64);
        assert_eq!(color, Color::Rgb { r: 255, g: 128, b: 64 });
        assert!(!color.is_default());
    }

    #[test]
    fn test_gray() {
        let gray = Color::gray(12);
        assert_eq!(gray, Color::Indexed(244)); // 232 + 12
    }
}
