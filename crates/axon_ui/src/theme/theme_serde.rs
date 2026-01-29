//! 主题序列化和反序列化
//!
//! 支持从 JSON 文件加载主题配置

use super::{Appearance, Theme, ThemeColors};
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
    Hsla { h: f32, s: f32, l: f32, a: f32 },
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
                ));
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
        if rgba[0] < 0.0
            || rgba[0] > 255.0
            || rgba[1] < 0.0
            || rgba[1] > 255.0
            || rgba[2] < 0.0
            || rgba[2] > 255.0
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

    // 滚动条颜色
    #[serde(default = "default_scrollbar_thumb_background")]
    pub scrollbar_thumb_background: SerializableColor,
    #[serde(default = "default_scrollbar_thumb_hover_background")]
    pub scrollbar_thumb_hover_background: SerializableColor,
    #[serde(default = "default_scrollbar_thumb_active_background")]
    pub scrollbar_thumb_active_background: SerializableColor,
    #[serde(default = "default_scrollbar_thumb_border")]
    pub scrollbar_thumb_border: SerializableColor,
    #[serde(default = "default_scrollbar_track_background")]
    pub scrollbar_track_background: SerializableColor,
    #[serde(default = "default_scrollbar_track_border")]
    pub scrollbar_track_border: SerializableColor,
}

// Default functions for scrollbar colors
fn default_scrollbar_thumb_background() -> SerializableColor {
    SerializableColor::Hsla { h: 0.0, s: 0.0, l: 0.45, a: 0.3 }
}
fn default_scrollbar_thumb_hover_background() -> SerializableColor {
    SerializableColor::Hsla { h: 0.0, s: 0.0, l: 0.50, a: 0.5 }
}
fn default_scrollbar_thumb_active_background() -> SerializableColor {
    SerializableColor::Hsla { h: 0.0, s: 0.0, l: 0.55, a: 0.7 }
}
fn default_scrollbar_thumb_border() -> SerializableColor {
    SerializableColor::Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 }
}
fn default_scrollbar_track_background() -> SerializableColor {
    SerializableColor::Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 }
}
fn default_scrollbar_track_border() -> SerializableColor {
    SerializableColor::Hsla { h: 220.0, s: 0.13, l: 0.30, a: 1.0 }
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
            scrollbar_thumb_background: self.scrollbar_thumb_background.to_hsla()?,
            scrollbar_thumb_hover_background: self.scrollbar_thumb_hover_background.to_hsla()?,
            scrollbar_thumb_active_background: self.scrollbar_thumb_active_background.to_hsla()?,
            scrollbar_thumb_border: self.scrollbar_thumb_border.to_hsla()?,
            scrollbar_track_background: self.scrollbar_track_background.to_hsla()?,
            scrollbar_track_border: self.scrollbar_track_border.to_hsla()?,
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
            menu_item_hover_background: SerializableColor::from_hsla(
                colors.menu_item_hover_background,
            ),
            menu_item_hover_text: SerializableColor::from_hsla(colors.menu_item_hover_text),
            menu_item_disabled_text: SerializableColor::from_hsla(colors.menu_item_disabled_text),
            scrollbar_thumb_background: SerializableColor::from_hsla(colors.scrollbar_thumb_background),
            scrollbar_thumb_hover_background: SerializableColor::from_hsla(colors.scrollbar_thumb_hover_background),
            scrollbar_thumb_active_background: SerializableColor::from_hsla(colors.scrollbar_thumb_active_background),
            scrollbar_thumb_border: SerializableColor::from_hsla(colors.scrollbar_thumb_border),
            scrollbar_track_background: SerializableColor::from_hsla(colors.scrollbar_track_background),
            scrollbar_track_border: SerializableColor::from_hsla(colors.scrollbar_track_border),
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
