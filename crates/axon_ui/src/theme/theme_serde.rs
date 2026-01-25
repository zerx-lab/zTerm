//! 主题序列化和反序列化
//!
//! 支持从 JSON 文件加载主题配置

use super::{Appearance, TerminalAnsiColors, TerminalColors, Theme, ThemeColors};
use gpui::Hsla;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 可序列化的颜色值
///
/// 支持三种格式:
/// - HEX 字符串: "#RRGGBB" 或 "#RRGGBBAA"
/// - RGBA 数组: [r, g, b, a] (r,g,b 为 0-255, a 为 0-1)
/// - HSLA 对象: {"h": 0-360, "s": 0-1, "l": 0-1, "a": 0-1}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SerializableColor {
    /// HEX 格式: "#RRGGBB" 或 "#RRGGBBAA"
    Hex(String),
    /// RGBA 数组: [r, g, b, a]
    Rgba([f32; 4]),
    /// HSLA 对象
    Hsla {
        h: f32,
        s: f32,
        l: f32,
        a: f32,
    },
}

/// 颜色解析错误
#[derive(Debug)]
pub enum ColorParseError {
    InvalidHex(String),
    InvalidRgba(String),
    InvalidHsla(String),
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHex(msg) => write!(f, "Invalid HEX color: {}", msg),
            Self::InvalidRgba(msg) => write!(f, "Invalid RGBA color: {}", msg),
            Self::InvalidHsla(msg) => write!(f, "Invalid HSLA color: {}", msg),
        }
    }
}

impl std::error::Error for ColorParseError {}

impl SerializableColor {
    /// 转换为 GPUI Hsla 类型
    pub fn to_hsla(&self) -> Result<Hsla, ColorParseError> {
        match self {
            Self::Hex(hex) => Self::parse_hex(hex),
            Self::Rgba(rgba) => Self::parse_rgba(rgba),
            Self::Hsla { h, s, l, a } => Ok(gpui::hsla(*h / 360.0, *s, *l, *a)),
        }
    }

    /// 从 GPUI Hsla 创建
    pub fn from_hsla(hsla: Hsla) -> Self {
        Self::Hsla {
            h: hsla.h * 360.0,
            s: hsla.s,
            l: hsla.l,
            a: hsla.a,
        }
    }

    /// 解析 HEX 颜色
    fn parse_hex(hex: &str) -> Result<Hsla, ColorParseError> {
        let hex = hex.trim_start_matches('#');

        let (r, g, b, a) = match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| ColorParseError::InvalidHex("Invalid red component".into()))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| ColorParseError::InvalidHex("Invalid green component".into()))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| ColorParseError::InvalidHex("Invalid blue component".into()))?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| ColorParseError::InvalidHex("Invalid red component".into()))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| ColorParseError::InvalidHex("Invalid green component".into()))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| ColorParseError::InvalidHex("Invalid blue component".into()))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| ColorParseError::InvalidHex("Invalid alpha component".into()))?;
                (r, g, b, a)
            }
            _ => {
                return Err(ColorParseError::InvalidHex(
                    "HEX color must be 6 or 8 characters".into(),
                ))
            }
        };

        // 转换为 HSLA
        let rgba = gpui::Rgba {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        };
        Ok(rgba.into())
    }

    /// 解析 RGBA 数组
    fn parse_rgba(rgba: &[f32; 4]) -> Result<Hsla, ColorParseError> {
        // 验证 RGB 值在 0-255 范围内
        if rgba[0] < 0.0 || rgba[0] > 255.0
            || rgba[1] < 0.0 || rgba[1] > 255.0
            || rgba[2] < 0.0 || rgba[2] > 255.0
        {
            return Err(ColorParseError::InvalidRgba(
                "RGB values must be in range 0-255".into(),
            ));
        }

        // 验证 Alpha 值在 0-1 范围内
        if rgba[3] < 0.0 || rgba[3] > 1.0 {
            return Err(ColorParseError::InvalidRgba(
                "Alpha value must be in range 0-1".into(),
            ));
        }

        let color = gpui::Rgba {
            r: rgba[0] / 255.0,
            g: rgba[1] / 255.0,
            b: rgba[2] / 255.0,
            a: rgba[3],
        };
        Ok(color.into())
    }
}

/// 可序列化的终端 ANSI 颜色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTerminalAnsiColors {
    pub black: SerializableColor,
    pub red: SerializableColor,
    pub green: SerializableColor,
    pub yellow: SerializableColor,
    pub blue: SerializableColor,
    pub magenta: SerializableColor,
    pub cyan: SerializableColor,
    pub white: SerializableColor,
    pub bright_black: SerializableColor,
    pub bright_red: SerializableColor,
    pub bright_green: SerializableColor,
    pub bright_yellow: SerializableColor,
    pub bright_blue: SerializableColor,
    pub bright_magenta: SerializableColor,
    pub bright_cyan: SerializableColor,
    pub bright_white: SerializableColor,
}

impl SerializableTerminalAnsiColors {
    /// 转换为 GPUI TerminalAnsiColors
    pub fn to_terminal_ansi_colors(&self) -> Result<TerminalAnsiColors, ColorParseError> {
        Ok(TerminalAnsiColors {
            black: self.black.to_hsla()?,
            red: self.red.to_hsla()?,
            green: self.green.to_hsla()?,
            yellow: self.yellow.to_hsla()?,
            blue: self.blue.to_hsla()?,
            magenta: self.magenta.to_hsla()?,
            cyan: self.cyan.to_hsla()?,
            white: self.white.to_hsla()?,
            bright_black: self.bright_black.to_hsla()?,
            bright_red: self.bright_red.to_hsla()?,
            bright_green: self.bright_green.to_hsla()?,
            bright_yellow: self.bright_yellow.to_hsla()?,
            bright_blue: self.bright_blue.to_hsla()?,
            bright_magenta: self.bright_magenta.to_hsla()?,
            bright_cyan: self.bright_cyan.to_hsla()?,
            bright_white: self.bright_white.to_hsla()?,
        })
    }

    /// 从 TerminalAnsiColors 创建
    pub fn from_terminal_ansi_colors(ansi: &TerminalAnsiColors) -> Self {
        Self {
            black: SerializableColor::from_hsla(ansi.black),
            red: SerializableColor::from_hsla(ansi.red),
            green: SerializableColor::from_hsla(ansi.green),
            yellow: SerializableColor::from_hsla(ansi.yellow),
            blue: SerializableColor::from_hsla(ansi.blue),
            magenta: SerializableColor::from_hsla(ansi.magenta),
            cyan: SerializableColor::from_hsla(ansi.cyan),
            white: SerializableColor::from_hsla(ansi.white),
            bright_black: SerializableColor::from_hsla(ansi.bright_black),
            bright_red: SerializableColor::from_hsla(ansi.bright_red),
            bright_green: SerializableColor::from_hsla(ansi.bright_green),
            bright_yellow: SerializableColor::from_hsla(ansi.bright_yellow),
            bright_blue: SerializableColor::from_hsla(ansi.bright_blue),
            bright_magenta: SerializableColor::from_hsla(ansi.bright_magenta),
            bright_cyan: SerializableColor::from_hsla(ansi.bright_cyan),
            bright_white: SerializableColor::from_hsla(ansi.bright_white),
        }
    }
}

/// 可序列化的终端颜色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTerminalColors {
    pub background: SerializableColor,
    pub foreground: SerializableColor,
    pub cursor: SerializableColor,
    pub selection_background: SerializableColor,
    pub ansi: SerializableTerminalAnsiColors,
}

impl SerializableTerminalColors {
    /// 转换为 GPUI TerminalColors
    pub fn to_terminal_colors(&self) -> Result<TerminalColors, ColorParseError> {
        Ok(TerminalColors {
            background: self.background.to_hsla()?,
            foreground: self.foreground.to_hsla()?,
            cursor: self.cursor.to_hsla()?,
            selection_background: self.selection_background.to_hsla()?,
            ansi: self.ansi.to_terminal_ansi_colors()?,
        })
    }

    /// 从 TerminalColors 创建
    pub fn from_terminal_colors(terminal: &TerminalColors) -> Self {
        Self {
            background: SerializableColor::from_hsla(terminal.background),
            foreground: SerializableColor::from_hsla(terminal.foreground),
            cursor: SerializableColor::from_hsla(terminal.cursor),
            selection_background: SerializableColor::from_hsla(terminal.selection_background),
            ansi: SerializableTerminalAnsiColors::from_terminal_ansi_colors(&terminal.ansi),
        }
    }
}

/// 可序列化的主题颜色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableThemeColors {
    pub background: SerializableColor,
    pub surface_background: SerializableColor,
    pub border: SerializableColor,
    pub border_variant: SerializableColor,
    pub text: SerializableColor,
    pub text_muted: SerializableColor,
    pub text_placeholder: SerializableColor,
    pub terminal: SerializableTerminalColors,
    pub icon: SerializableColor,
    pub icon_muted: SerializableColor,
    pub danger: SerializableColor,
    pub danger_foreground: SerializableColor,
    pub titlebar_background: SerializableColor,
    pub tab_bar_background: SerializableColor,
    pub tab_active_background: SerializableColor,
    pub tab_inactive_background: SerializableColor,
    pub tab_hover_background: SerializableColor,
    pub tab_active_indicator: SerializableColor,
    pub button_hover_background: SerializableColor,
    pub button_active_background: SerializableColor,
    pub statusbar_background: SerializableColor,
    pub menu_background: SerializableColor,
    pub menu_border: SerializableColor,
    pub menu_item_hover_background: SerializableColor,
    pub menu_item_hover_text: SerializableColor,
    pub menu_item_disabled_text: SerializableColor,
}

impl SerializableThemeColors {
    /// 转换为 GPUI ThemeColors
    pub fn to_theme_colors(&self) -> Result<ThemeColors, ColorParseError> {
        Ok(ThemeColors {
            background: self.background.to_hsla()?,
            surface_background: self.surface_background.to_hsla()?,
            border: self.border.to_hsla()?,
            border_variant: self.border_variant.to_hsla()?,
            text: self.text.to_hsla()?,
            text_muted: self.text_muted.to_hsla()?,
            text_placeholder: self.text_placeholder.to_hsla()?,
            terminal: self.terminal.to_terminal_colors()?,
            icon: self.icon.to_hsla()?,
            icon_muted: self.icon_muted.to_hsla()?,
            danger: self.danger.to_hsla()?,
            danger_foreground: self.danger_foreground.to_hsla()?,
            titlebar_background: self.titlebar_background.to_hsla()?,
            tab_bar_background: self.tab_bar_background.to_hsla()?,
            tab_active_background: self.tab_active_background.to_hsla()?,
            tab_inactive_background: self.tab_inactive_background.to_hsla()?,
            tab_hover_background: self.tab_hover_background.to_hsla()?,
            tab_active_indicator: self.tab_active_indicator.to_hsla()?,
            button_hover_background: self.button_hover_background.to_hsla()?,
            button_active_background: self.button_active_background.to_hsla()?,
            statusbar_background: self.statusbar_background.to_hsla()?,
            menu_background: self.menu_background.to_hsla()?,
            menu_border: self.menu_border.to_hsla()?,
            menu_item_hover_background: self.menu_item_hover_background.to_hsla()?,
            menu_item_hover_text: self.menu_item_hover_text.to_hsla()?,
            menu_item_disabled_text: self.menu_item_disabled_text.to_hsla()?,
        })
    }

    /// 从 ThemeColors 创建
    pub fn from_theme_colors(colors: &ThemeColors) -> Self {
        Self {
            background: SerializableColor::from_hsla(colors.background),
            surface_background: SerializableColor::from_hsla(colors.surface_background),
            border: SerializableColor::from_hsla(colors.border),
            border_variant: SerializableColor::from_hsla(colors.border_variant),
            text: SerializableColor::from_hsla(colors.text),
            text_muted: SerializableColor::from_hsla(colors.text_muted),
            text_placeholder: SerializableColor::from_hsla(colors.text_placeholder),
            terminal: SerializableTerminalColors::from_terminal_colors(&colors.terminal),
            icon: SerializableColor::from_hsla(colors.icon),
            icon_muted: SerializableColor::from_hsla(colors.icon_muted),
            danger: SerializableColor::from_hsla(colors.danger),
            danger_foreground: SerializableColor::from_hsla(colors.danger_foreground),
            titlebar_background: SerializableColor::from_hsla(colors.titlebar_background),
            tab_bar_background: SerializableColor::from_hsla(colors.tab_bar_background),
            tab_active_background: SerializableColor::from_hsla(colors.tab_active_background),
            tab_inactive_background: SerializableColor::from_hsla(colors.tab_inactive_background),
            tab_hover_background: SerializableColor::from_hsla(colors.tab_hover_background),
            tab_active_indicator: SerializableColor::from_hsla(colors.tab_active_indicator),
            button_hover_background: SerializableColor::from_hsla(colors.button_hover_background),
            button_active_background: SerializableColor::from_hsla(colors.button_active_background),
            statusbar_background: SerializableColor::from_hsla(colors.statusbar_background),
            menu_background: SerializableColor::from_hsla(colors.menu_background),
            menu_border: SerializableColor::from_hsla(colors.menu_border),
            menu_item_hover_background: SerializableColor::from_hsla(colors.menu_item_hover_background),
            menu_item_hover_text: SerializableColor::from_hsla(colors.menu_item_hover_text),
            menu_item_disabled_text: SerializableColor::from_hsla(colors.menu_item_disabled_text),
        }
    }
}

/// 可序列化的主题定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTheme {
    pub name: String,
    pub appearance: Appearance,
    pub colors: SerializableThemeColors,
}

impl SerializableTheme {
    /// 转换为 GPUI Theme
    pub fn to_theme(&self) -> Result<Theme, ColorParseError> {
        Ok(Theme::new(
            self.name.clone(),
            self.appearance,
            self.colors.to_theme_colors()?,
        ))
    }

    /// 从 Theme 创建
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            name: theme.name().to_string(),
            appearance: theme.appearance(),
            colors: SerializableThemeColors::from_theme_colors(theme.colors()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_6_digits() {
        let color = SerializableColor::Hex("#1e1e1e".to_string());
        let hsla = color.to_hsla().unwrap();

        // 转换为 RGBA 验证
        let rgba: gpui::Rgba = hsla.into();
        assert!((rgba.r - 30.0 / 255.0).abs() < 0.01);
        assert!((rgba.g - 30.0 / 255.0).abs() < 0.01);
        assert!((rgba.b - 30.0 / 255.0).abs() < 0.01);
        assert_eq!(rgba.a, 1.0);
    }

    #[test]
    fn test_parse_hex_8_digits() {
        let color = SerializableColor::Hex("#1e1e1e80".to_string());
        let hsla = color.to_hsla().unwrap();

        let rgba: gpui::Rgba = hsla.into();
        assert!((rgba.a - 128.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_rgba_array() {
        let color = SerializableColor::Rgba([30.0, 30.0, 30.0, 1.0]);
        let hsla = color.to_hsla().unwrap();

        let rgba: gpui::Rgba = hsla.into();
        assert!((rgba.r - 30.0 / 255.0).abs() < 0.01);
        assert_eq!(rgba.a, 1.0);
    }

    #[test]
    fn test_parse_hsla_object() {
        let color = SerializableColor::Hsla {
            h: 220.0,
            s: 0.13,
            l: 0.15,
            a: 1.0,
        };
        let hsla = color.to_hsla().unwrap();

        assert!((hsla.h - 220.0 / 360.0).abs() < 0.01);
        assert!((hsla.s - 0.13).abs() < 0.01);
        assert!((hsla.l - 0.15).abs() < 0.01);
        assert_eq!(hsla.a, 1.0);
    }

    #[test]
    fn test_invalid_rgba_range() {
        let color = SerializableColor::Rgba([256.0, 0.0, 0.0, 1.0]);
        assert!(color.to_hsla().is_err());
    }

    #[test]
    fn test_invalid_hex_length() {
        let color = SerializableColor::Hex("#1e1".to_string());
        assert!(color.to_hsla().is_err());
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original = gpui::hsla(220.0 / 360.0, 0.13, 0.15, 1.0);
        let serializable = SerializableColor::from_hsla(original);
        let converted = serializable.to_hsla().unwrap();

        assert!((original.h - converted.h).abs() < 0.01);
        assert!((original.s - converted.s).abs() < 0.01);
        assert!((original.l - converted.l).abs() < 0.01);
        assert_eq!(original.a, converted.a);
    }
}
