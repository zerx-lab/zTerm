//! 终端颜色转换
//!
//! 将 zterm_terminal 的 Color 转换为 GPUI 的 Rgba

use axon_ui::ThemeContext;
use gpui::{App, Rgba};
use zterm_terminal::Color;

/// ANSI 16 色调色板（标准终端颜色）
/// 索引 0-7: 标准颜色
/// 索引 8-15: 高亮颜色
const ANSI_16_COLORS: [(u8, u8, u8); 16] = [
    // 标准颜色 (0-7)
    (0, 0, 0),       // 0: Black
    (205, 49, 49),   // 1: Red
    (13, 188, 121),  // 2: Green
    (229, 229, 16),  // 3: Yellow
    (36, 114, 200),  // 4: Blue
    (188, 63, 188),  // 5: Magenta
    (17, 168, 205),  // 6: Cyan
    (229, 229, 229), // 7: White
    // 高亮颜色 (8-15)
    (102, 102, 102), // 8: Bright Black (Gray)
    (241, 76, 76),   // 9: Bright Red
    (35, 209, 139),  // 10: Bright Green
    (245, 245, 67),  // 11: Bright Yellow
    (59, 142, 234),  // 12: Bright Blue
    (214, 112, 214), // 13: Bright Magenta
    (41, 184, 219),  // 14: Bright Cyan
    (255, 255, 255), // 15: Bright White
];

/// 将终端 Color 转换为 GPUI Rgba
pub fn color_to_rgba(color: &Color, is_foreground: bool, cx: &App) -> Rgba {
    let theme = cx.current_theme();
    let colors = &theme.colors;

    match color {
        Color::DefaultForeground => colors.text.to_rgb(),
        Color::DefaultBackground => colors.background.to_rgb(),
        Color::Indexed(idx) => indexed_color_to_rgba(*idx),
        Color::Rgb { r, g, b } => Rgba {
            r: *r as f32 / 255.0,
            g: *g as f32 / 255.0,
            b: *b as f32 / 255.0,
            a: 1.0,
        },
    }
}

/// 将 256 色索引转换为 Rgba
fn indexed_color_to_rgba(idx: u8) -> Rgba {
    if idx < 16 {
        // ANSI 16 色
        let (r, g, b) = ANSI_16_COLORS[idx as usize];
        Rgba {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    } else if idx < 232 {
        // 216 色立方体 (6x6x6)
        // 索引 16-231
        let idx = idx - 16;
        let r = (idx / 36) % 6;
        let g = (idx / 6) % 6;
        let b = idx % 6;

        // 每个分量的值: 0, 95, 135, 175, 215, 255
        let component = |c: u8| -> f32 {
            if c == 0 {
                0.0
            } else {
                (55 + c * 40) as f32 / 255.0
            }
        };

        Rgba {
            r: component(r),
            g: component(g),
            b: component(b),
            a: 1.0,
        }
    } else {
        // 24 级灰度 (232-255)
        let gray = idx - 232;
        let level = (8 + gray * 10) as f32 / 255.0;
        Rgba {
            r: level,
            g: level,
            b: level,
            a: 1.0,
        }
    }
}

/// 获取前景色（考虑反转模式）
pub fn get_foreground(fg: &Color, bg: &Color, reverse: bool, cx: &App) -> Rgba {
    if reverse {
        color_to_rgba(bg, false, cx)
    } else {
        color_to_rgba(fg, true, cx)
    }
}

/// 获取背景色（考虑反转模式）
pub fn get_background(fg: &Color, bg: &Color, reverse: bool, cx: &App) -> Rgba {
    if reverse {
        color_to_rgba(fg, true, cx)
    } else {
        color_to_rgba(bg, false, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_16_colors() {
        // Black
        let black = indexed_color_to_rgba(0);
        assert_eq!(black.r, 0.0);
        assert_eq!(black.g, 0.0);
        assert_eq!(black.b, 0.0);

        // Red
        let red = indexed_color_to_rgba(1);
        assert!(red.r > 0.5);
        assert!(red.g < 0.3);
        assert!(red.b < 0.3);

        // Bright White
        let white = indexed_color_to_rgba(15);
        assert_eq!(white.r, 1.0);
        assert_eq!(white.g, 1.0);
        assert_eq!(white.b, 1.0);
    }

    #[test]
    fn test_256_cube() {
        // 索引 16 应该是黑色 (0,0,0)
        let c16 = indexed_color_to_rgba(16);
        assert_eq!(c16.r, 0.0);
        assert_eq!(c16.g, 0.0);
        assert_eq!(c16.b, 0.0);

        // 索引 231 应该是白色 (5,5,5)
        let c231 = indexed_color_to_rgba(231);
        assert_eq!(c231.r, 1.0);
        assert_eq!(c231.g, 1.0);
        assert_eq!(c231.b, 1.0);
    }

    #[test]
    fn test_grayscale() {
        // 232 是最暗的灰色
        let dark = indexed_color_to_rgba(232);
        assert!(dark.r < 0.1);

        // 255 是最亮的灰色
        let light = indexed_color_to_rgba(255);
        assert!(light.r > 0.9);
    }
}
