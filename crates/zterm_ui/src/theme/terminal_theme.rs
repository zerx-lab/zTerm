//! Terminal theme definitions

use gpui::*;

/// Theme configuration for the terminal
#[derive(Clone)]
pub struct TerminalTheme {
    /// Background color
    pub background: Rgba,

    /// Foreground (text) color
    pub foreground: Rgba,

    /// Cursor color
    pub cursor_color: Rgba,

    /// Selection background color
    pub selection_background: Rgba,

    /// ANSI colors (16 colors: 8 normal + 8 bright)
    pub ansi_colors: [Rgba; 16],

    /// Font family
    pub font_family: SharedString,

    /// Font size in pixels
    pub font_size: f32,

    /// Line height multiplier
    pub line_height: f32,
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl TerminalTheme {
    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            background: rgba(0x1a1a1aff),
            foreground: rgba(0xe0e0e0ff),
            cursor_color: rgba(0x00ff00ff),
            selection_background: rgba(0x3d3d3dff),
            ansi_colors: [
                // Normal colors
                rgba(0x1a1a1aff), // Black
                rgba(0xf44747ff), // Red
                rgba(0x6a9955ff), // Green
                rgba(0xd7ba7dff), // Yellow
                rgba(0x569cd6ff), // Blue
                rgba(0xc586c0ff), // Magenta
                rgba(0x4ec9b0ff), // Cyan
                rgba(0xd4d4d4ff), // White
                // Bright colors
                rgba(0x808080ff), // Bright Black
                rgba(0xf44747ff), // Bright Red
                rgba(0x6a9955ff), // Bright Green
                rgba(0xd7ba7dff), // Bright Yellow
                rgba(0x569cd6ff), // Bright Blue
                rgba(0xc586c0ff), // Bright Magenta
                rgba(0x4ec9b0ff), // Bright Cyan
                rgba(0xffffffff), // Bright White
            ],
            font_family: "JetBrainsMono Nerd Font Mono".into(),
            font_size: 14.0,
            line_height: 1.4,
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            background: rgba(0xffffffff),
            foreground: rgba(0x1a1a1aff),
            cursor_color: rgba(0x000000ff),
            selection_background: rgba(0xadd6ffff),
            ansi_colors: [
                // Normal colors
                rgba(0x000000ff), // Black
                rgba(0xc91b00ff), // Red
                rgba(0x00c200ff), // Green
                rgba(0xc7c400ff), // Yellow
                rgba(0x0225c7ff), // Blue
                rgba(0xc930c7ff), // Magenta
                rgba(0x00c5c7ff), // Cyan
                rgba(0xc7c7c7ff), // White
                // Bright colors
                rgba(0x686868ff), // Bright Black
                rgba(0xff6e67ff), // Bright Red
                rgba(0x5ff967ff), // Bright Green
                rgba(0xfefb67ff), // Bright Yellow
                rgba(0x6871ffff), // Bright Blue
                rgba(0xff76ffff), // Bright Magenta
                rgba(0x5ffdffff), // Bright Cyan
                rgba(0xffffffff), // Bright White
            ],
            font_family: "JetBrainsMono Nerd Font Mono".into(),
            font_size: 14.0,
            line_height: 1.4,
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            background: rgba(0x282a36ff),
            foreground: rgba(0xf8f8f2ff),
            cursor_color: rgba(0xf8f8f2ff),
            selection_background: rgba(0x44475aff),
            ansi_colors: [
                // Normal colors
                rgba(0x21222cff), // Black
                rgba(0xff5555ff), // Red
                rgba(0x50fa7bff), // Green
                rgba(0xf1fa8cff), // Yellow
                rgba(0xbd93f9ff), // Blue
                rgba(0xff79c6ff), // Magenta
                rgba(0x8be9fdff), // Cyan
                rgba(0xf8f8f2ff), // White
                // Bright colors
                rgba(0x6272a4ff), // Bright Black
                rgba(0xff6e6eff), // Bright Red
                rgba(0x69ff94ff), // Bright Green
                rgba(0xffffacff), // Bright Yellow
                rgba(0xd6acffff), // Bright Blue
                rgba(0xff92dfff), // Bright Magenta
                rgba(0xa4ffffff), // Bright Cyan
                rgba(0xffffffff), // Bright White
            ],
            font_family: "JetBrainsMono Nerd Font Mono".into(),
            font_size: 14.0,
            line_height: 1.4,
        }
    }

    /// One Dark theme
    pub fn one_dark() -> Self {
        Self {
            background: rgba(0x282c34ff),
            foreground: rgba(0xabb2bfff),
            cursor_color: rgba(0x528bffff),
            selection_background: rgba(0x3e4451ff),
            ansi_colors: [
                // Normal colors
                rgba(0x1e2127ff), // Black
                rgba(0xe06c75ff), // Red
                rgba(0x98c379ff), // Green
                rgba(0xd19a66ff), // Yellow
                rgba(0x61afefff), // Blue
                rgba(0xc678ddff), // Magenta
                rgba(0x56b6c2ff), // Cyan
                rgba(0xabb2bfff), // White
                // Bright colors
                rgba(0x5c6370ff), // Bright Black
                rgba(0xe06c75ff), // Bright Red
                rgba(0x98c379ff), // Bright Green
                rgba(0xd19a66ff), // Bright Yellow
                rgba(0x61afefff), // Bright Blue
                rgba(0xc678ddff), // Bright Magenta
                rgba(0x56b6c2ff), // Bright Cyan
                rgba(0xffffffff), // Bright White
            ],
            font_family: "JetBrainsMono Nerd Font Mono".into(),
            font_size: 14.0,
            line_height: 1.4,
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            background: rgba(0x2e3440ff),
            foreground: rgba(0xd8dee9ff),
            cursor_color: rgba(0xd8dee9ff),
            selection_background: rgba(0x434c5eff),
            ansi_colors: [
                // Normal colors
                rgba(0x3b4252ff), // Black
                rgba(0xbf616aff), // Red
                rgba(0xa3be8cff), // Green
                rgba(0xebcb8bff), // Yellow
                rgba(0x81a1c1ff), // Blue
                rgba(0xb48eadff), // Magenta
                rgba(0x88c0d0ff), // Cyan
                rgba(0xe5e9f0ff), // White
                // Bright colors
                rgba(0x4c566aff), // Bright Black
                rgba(0xbf616aff), // Bright Red
                rgba(0xa3be8cff), // Bright Green
                rgba(0xebcb8bff), // Bright Yellow
                rgba(0x81a1c1ff), // Bright Blue
                rgba(0xb48eadff), // Bright Magenta
                rgba(0x8fbcbbff), // Bright Cyan
                rgba(0xeceff4ff), // Bright White
            ],
            font_family: "JetBrainsMono Nerd Font Mono".into(),
            font_size: 14.0,
            line_height: 1.4,
        }
    }
}

// Note: GPUI component tests require #[gpui::test] and TestAppContext.
// Basic unit tests are in a separate test file to avoid macro expansion issues.
