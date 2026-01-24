//! 内置主题

use super::{Appearance, TerminalAnsiColors, TerminalColors, Theme, ThemeColors};
use gpui::hsla;

/// 创建默认深色主题
pub fn default_dark() -> Theme {
    let colors = ThemeColors {
        // 基础颜色 - 深色背景，浅色文本
        background: hsla(220.0 / 360.0, 0.13, 0.15, 1.0),
        surface_background: hsla(220.0 / 360.0, 0.13, 0.18, 1.0),
        border: hsla(220.0 / 360.0, 0.13, 0.30, 1.0),
        border_variant: hsla(220.0 / 360.0, 0.13, 0.22, 1.0),
        text: hsla(0.0, 0.0, 0.92, 1.0),
        text_muted: hsla(0.0, 0.0, 0.60, 1.0),
        text_placeholder: hsla(0.0, 0.0, 0.45, 1.0),

        // 图标颜色
        icon: hsla(0.0, 0.0, 0.80, 1.0),          // 默认图标颜色
        icon_muted: hsla(0.0, 0.0, 0.50, 1.0),    // 次要图标颜色

        // 语义化颜色
        danger: hsla(355.0 / 360.0, 0.75, 0.65, 1.0),     // 柔和红色
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),      // 白色

        // UI 组件颜色
        titlebar_background: hsla(220.0 / 360.0, 0.13, 0.12, 1.0), // #1e1e1e
        tab_bar_background: hsla(220.0 / 360.0, 0.13, 0.12, 1.0),
        tab_active_background: hsla(220.0 / 360.0, 0.13, 0.18, 1.0),
        tab_inactive_background: hsla(220.0 / 360.0, 0.13, 0.15, 1.0),
        tab_hover_background: hsla(220.0 / 360.0, 0.13, 0.24, 1.0),
        tab_active_indicator: hsla(207.0 / 360.0, 0.82, 0.66, 1.0), // 蓝色指示器
        button_hover_background: hsla(220.0 / 360.0, 0.13, 0.24, 1.0),
        button_active_background: hsla(220.0 / 360.0, 0.13, 0.30, 1.0),
        statusbar_background: hsla(220.0 / 360.0, 0.13, 0.14, 1.0), // 状态栏背景

        // 菜单颜色
        menu_background: hsla(220.0 / 360.0, 0.13, 0.18, 1.0), // 与 surface_background 一致
        menu_border: hsla(220.0 / 360.0, 0.13, 0.30, 1.0), // 与 border 一致
        menu_item_hover_background: hsla(207.0 / 360.0, 0.82, 0.66, 1.0), // 蓝色高亮
        menu_item_hover_text: hsla(0.0, 0.0, 1.0, 1.0), // 白色文本
        menu_item_disabled_text: hsla(0.0, 0.0, 0.45, 1.0), // 灰色禁用文本

        // 终端颜色
        terminal: TerminalColors {
            background: hsla(220.0 / 360.0, 0.13, 0.15, 1.0),
            foreground: hsla(0.0, 0.0, 0.92, 1.0),
            cursor: hsla(207.0 / 360.0, 0.82, 0.66, 1.0),
            selection_background: hsla(207.0 / 360.0, 0.82, 0.66, 0.30),

            ansi: TerminalAnsiColors {
                // 标准 ANSI 颜色
                black: hsla(0.0, 0.0, 0.0, 1.0),
                red: hsla(355.0 / 360.0, 0.65, 0.65, 1.0),
                green: hsla(95.0 / 360.0, 0.38, 0.62, 1.0),
                yellow: hsla(39.0 / 360.0, 0.67, 0.69, 1.0),
                blue: hsla(207.0 / 360.0, 0.82, 0.73, 1.0),  // 提高亮度：0.66 -> 0.73
                magenta: hsla(286.0 / 360.0, 0.51, 0.64, 1.0),
                cyan: hsla(187.0 / 360.0, 0.47, 0.55, 1.0),
                white: hsla(0.0, 0.0, 0.75, 1.0),

                // 亮色 ANSI 颜色
                bright_black: hsla(0.0, 0.0, 0.50, 1.0),
                bright_red: hsla(355.0 / 360.0, 0.75, 0.75, 1.0),
                bright_green: hsla(95.0 / 360.0, 0.48, 0.72, 1.0),
                bright_yellow: hsla(39.0 / 360.0, 0.77, 0.79, 1.0),
                bright_blue: hsla(207.0 / 360.0, 0.92, 0.82, 1.0),  // 提高亮度：0.76 -> 0.82
                bright_magenta: hsla(286.0 / 360.0, 0.61, 0.74, 1.0),
                bright_cyan: hsla(187.0 / 360.0, 0.57, 0.65, 1.0),
                bright_white: hsla(0.0, 0.0, 1.0, 1.0),
            },
        },
    };

    Theme::new("Default Dark", Appearance::Dark, colors)
}

/// 创建 GitHub Dark 主题
pub fn github_dark() -> Theme {
    let colors = ThemeColors {
        // GitHub Dark 基础颜色
        background: hsla(220.0 / 360.0, 0.13, 0.09, 1.0), // #0d1117
        surface_background: hsla(220.0 / 360.0, 0.13, 0.13, 1.0), // #161b22
        border: hsla(215.0 / 360.0, 0.12, 0.22, 1.0), // #30363d
        border_variant: hsla(215.0 / 360.0, 0.10, 0.18, 1.0),
        text: hsla(210.0 / 360.0, 0.24, 0.88, 1.0), // #c9d1d9
        text_muted: hsla(217.0 / 360.0, 0.10, 0.60, 1.0), // #8b949e
        text_placeholder: hsla(215.0 / 360.0, 0.08, 0.47, 1.0),

        // 图标颜色
        icon: hsla(210.0 / 360.0, 0.24, 0.88, 1.0),       // 与文本颜色一致
        icon_muted: hsla(217.0 / 360.0, 0.10, 0.60, 1.0), // 与 text_muted 一致

        // 语义化颜色
        danger: hsla(0.0, 0.73, 0.62, 1.0),               // #ff7b72 GitHub 红
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),      // 白色

        // UI 组件颜色
        titlebar_background: hsla(220.0 / 360.0, 0.13, 0.11, 1.0), // #161b22 稍暗
        tab_bar_background: hsla(220.0 / 360.0, 0.13, 0.11, 1.0),
        tab_active_background: hsla(220.0 / 360.0, 0.13, 0.15, 1.0),
        tab_inactive_background: hsla(220.0 / 360.0, 0.13, 0.11, 1.0),
        tab_hover_background: hsla(220.0 / 360.0, 0.13, 0.18, 1.0),
        tab_active_indicator: hsla(212.0 / 360.0, 0.92, 0.62, 1.0), // #58a6ff GitHub 蓝
        button_hover_background: hsla(220.0 / 360.0, 0.13, 0.18, 1.0),
        button_active_background: hsla(220.0 / 360.0, 0.13, 0.22, 1.0),
        statusbar_background: hsla(220.0 / 360.0, 0.13, 0.13, 1.0), // 状态栏背景

        // 菜单颜色
        menu_background: hsla(220.0 / 360.0, 0.13, 0.13, 1.0), // 与 surface_background 一致
        menu_border: hsla(215.0 / 360.0, 0.12, 0.22, 1.0), // 与 border 一致
        menu_item_hover_background: hsla(212.0 / 360.0, 0.92, 0.62, 1.0), // GitHub 蓝色高亮
        menu_item_hover_text: hsla(0.0, 0.0, 1.0, 1.0), // 白色文本
        menu_item_disabled_text: hsla(217.0 / 360.0, 0.10, 0.50, 1.0), // 灰色禁用文本

        terminal: TerminalColors {
            background: hsla(220.0 / 360.0, 0.13, 0.09, 1.0),
            foreground: hsla(210.0 / 360.0, 0.24, 0.88, 1.0),
            cursor: hsla(212.0 / 360.0, 0.92, 0.62, 1.0), // #58a6ff
            selection_background: hsla(212.0 / 360.0, 0.92, 0.62, 0.25),

            ansi: TerminalAnsiColors {
                black: hsla(0.0, 0.0, 0.13, 1.0),
                red: hsla(0.0, 0.73, 0.62, 1.0), // #ff7b72
                green: hsla(130.0 / 360.0, 0.47, 0.64, 1.0), // #7ee787
                yellow: hsla(39.0 / 360.0, 0.99, 0.68, 1.0), // #f0883e
                blue: hsla(212.0 / 360.0, 0.92, 0.70, 1.0), // #58a6ff - 提高亮度：0.62 -> 0.70
                magenta: hsla(290.0 / 360.0, 0.66, 0.73, 1.0), // #d2a8ff
                cyan: hsla(187.0 / 360.0, 0.73, 0.68, 1.0), // #79c0ff
                white: hsla(210.0 / 360.0, 0.24, 0.88, 1.0),

                bright_black: hsla(215.0 / 360.0, 0.10, 0.45, 1.0),
                bright_red: hsla(0.0, 0.79, 0.70, 1.0),
                bright_green: hsla(130.0 / 360.0, 0.57, 0.74, 1.0),
                bright_yellow: hsla(39.0 / 360.0, 0.99, 0.78, 1.0),
                bright_blue: hsla(212.0 / 360.0, 0.97, 0.78, 1.0),  // 提高亮度：0.72 -> 0.78
                bright_magenta: hsla(290.0 / 360.0, 0.76, 0.83, 1.0),
                bright_cyan: hsla(187.0 / 360.0, 0.83, 0.78, 1.0),
                bright_white: hsla(210.0 / 360.0, 0.30, 0.96, 1.0),
            },
        },
    };

    Theme::new("GitHub Dark", Appearance::Dark, colors)
}

/// 创建 GitHub Light 主题
pub fn github_light() -> Theme {
    let colors = ThemeColors {
        // GitHub Light 基础颜色
        background: hsla(0.0, 0.0, 1.0, 1.0), // #ffffff
        surface_background: hsla(210.0 / 360.0, 0.18, 0.98, 1.0), // #f6f8fa
        border: hsla(210.0 / 360.0, 0.18, 0.85, 1.0), // #d0d7de
        border_variant: hsla(210.0 / 360.0, 0.16, 0.91, 1.0),
        text: hsla(213.0 / 360.0, 0.18, 0.20, 1.0), // #24292f
        text_muted: hsla(213.0 / 360.0, 0.12, 0.45, 1.0), // #57606a
        text_placeholder: hsla(213.0 / 360.0, 0.10, 0.60, 1.0),

        // 图标颜色
        icon: hsla(213.0 / 360.0, 0.18, 0.20, 1.0),       // 与文本颜色一致
        icon_muted: hsla(213.0 / 360.0, 0.12, 0.45, 1.0), // 与 text_muted 一致

        // 语义化颜色
        danger: hsla(0.0, 0.67, 0.42, 1.0),               // #cf222e GitHub 红
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),      // 白色

        // UI 组件颜色
        titlebar_background: hsla(210.0 / 360.0, 0.18, 0.97, 1.0), // #f6f8fa
        tab_bar_background: hsla(210.0 / 360.0, 0.18, 0.97, 1.0),
        tab_active_background: hsla(0.0, 0.0, 1.0, 1.0), // 白色
        tab_inactive_background: hsla(210.0 / 360.0, 0.18, 0.95, 1.0),
        tab_hover_background: hsla(210.0 / 360.0, 0.18, 0.93, 1.0),
        tab_active_indicator: hsla(212.0 / 360.0, 1.0, 0.40, 1.0), // #0969da GitHub 蓝
        button_hover_background: hsla(210.0 / 360.0, 0.18, 0.90, 1.0),
        button_active_background: hsla(210.0 / 360.0, 0.18, 0.85, 1.0),
        statusbar_background: hsla(210.0 / 360.0, 0.18, 0.95, 1.0), // 状态栏背景

        // 菜单颜色
        menu_background: hsla(0.0, 0.0, 1.0, 1.0), // 纯白背景
        menu_border: hsla(210.0 / 360.0, 0.18, 0.85, 1.0), // 与 border 一致
        menu_item_hover_background: hsla(212.0 / 360.0, 1.0, 0.40, 1.0), // GitHub 蓝色高亮
        menu_item_hover_text: hsla(0.0, 0.0, 1.0, 1.0), // 白色文本
        menu_item_disabled_text: hsla(213.0 / 360.0, 0.10, 0.60, 1.0), // 浅灰色禁用文本

        terminal: TerminalColors {
            background: hsla(0.0, 0.0, 1.0, 1.0),
            foreground: hsla(213.0 / 360.0, 0.18, 0.20, 1.0),
            cursor: hsla(212.0 / 360.0, 1.0, 0.40, 1.0), // #0969da
            selection_background: hsla(212.0 / 360.0, 0.89, 0.65, 0.25),

            ansi: TerminalAnsiColors {
                black: hsla(213.0 / 360.0, 0.18, 0.20, 1.0),
                red: hsla(0.0, 0.67, 0.42, 1.0), // #cf222e
                green: hsla(137.0 / 360.0, 0.55, 0.35, 1.0), // #1a7f37
                yellow: hsla(29.0 / 360.0, 0.84, 0.41, 1.0), // #bf8700
                blue: hsla(212.0 / 360.0, 1.0, 0.33, 1.0), // #0969da - 降低亮度：0.40 -> 0.33
                magenta: hsla(278.0 / 360.0, 0.62, 0.46, 1.0), // #8250df
                cyan: hsla(191.0 / 360.0, 0.99, 0.34, 1.0), // #1b7c83
                white: hsla(0.0, 0.0, 0.85, 1.0), // 提高亮度用于反显：0.45 -> 0.85

                bright_black: hsla(213.0 / 360.0, 0.14, 0.35, 1.0),
                bright_red: hsla(0.0, 0.77, 0.52, 1.0),
                bright_green: hsla(137.0 / 360.0, 0.65, 0.45, 1.0),
                bright_yellow: hsla(29.0 / 360.0, 0.94, 0.51, 1.0),
                bright_blue: hsla(212.0 / 360.0, 1.0, 0.42, 1.0),  // 降低亮度：0.50 -> 0.42
                bright_magenta: hsla(278.0 / 360.0, 0.72, 0.56, 1.0),
                bright_cyan: hsla(191.0 / 360.0, 1.0, 0.44, 1.0),
                bright_white: hsla(0.0, 0.0, 0.95, 1.0),
            },
        },
    };

    Theme::new("GitHub Light", Appearance::Light, colors)
}

/// 创建 Tokyo Night Dark 主题
pub fn tokyo_night_dark() -> Theme {
    let colors = ThemeColors {
        // Tokyo Night Dark 基础颜色
        background: hsla(234.0 / 360.0, 0.17, 0.13, 1.0), // #1a1b26
        surface_background: hsla(234.0 / 360.0, 0.20, 0.16, 1.0), // #16161e
        border: hsla(234.0 / 360.0, 0.15, 0.25, 1.0), // #363b54
        border_variant: hsla(234.0 / 360.0, 0.13, 0.20, 1.0),
        text: hsla(232.0 / 360.0, 0.74, 0.85, 1.0), // #c0caf5
        text_muted: hsla(225.0 / 360.0, 0.27, 0.64, 1.0), // #787c99
        text_placeholder: hsla(225.0 / 360.0, 0.20, 0.50, 1.0),

        // 图标颜色
        icon: hsla(232.0 / 360.0, 0.74, 0.85, 1.0),       // 与文本颜色一致
        icon_muted: hsla(225.0 / 360.0, 0.27, 0.64, 1.0), // 与 text_muted 一致

        // 语义化颜色
        danger: hsla(343.0 / 360.0, 0.88, 0.75, 1.0),     // #f7768e Tokyo Night 红
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),      // 白色

        // UI 组件颜色
        titlebar_background: hsla(234.0 / 360.0, 0.17, 0.12, 1.0), // #1a1b26 稍暗
        tab_bar_background: hsla(234.0 / 360.0, 0.17, 0.12, 1.0),
        tab_active_background: hsla(234.0 / 360.0, 0.20, 0.18, 1.0),
        tab_inactive_background: hsla(234.0 / 360.0, 0.17, 0.13, 1.0),
        tab_hover_background: hsla(234.0 / 360.0, 0.23, 0.22, 1.0),
        tab_active_indicator: hsla(217.0 / 360.0, 0.92, 0.78, 1.0), // #7aa2f7 Tokyo Night 蓝
        button_hover_background: hsla(234.0 / 360.0, 0.23, 0.22, 1.0),
        button_active_background: hsla(234.0 / 360.0, 0.26, 0.28, 1.0),
        statusbar_background: hsla(234.0 / 360.0, 0.20, 0.15, 1.0), // 状态栏背景

        // 菜单颜色
        menu_background: hsla(234.0 / 360.0, 0.20, 0.16, 1.0), // 与 surface_background 一致
        menu_border: hsla(234.0 / 360.0, 0.15, 0.25, 1.0), // 与 border 一致
        menu_item_hover_background: hsla(217.0 / 360.0, 0.92, 0.78, 1.0), // Tokyo Night 蓝色
        menu_item_hover_text: hsla(234.0 / 360.0, 0.17, 0.13, 1.0), // 深色文本以保持对比
        menu_item_disabled_text: hsla(225.0 / 360.0, 0.27, 0.64, 1.0), // text_muted 颜色

        terminal: TerminalColors {
            background: hsla(234.0 / 360.0, 0.17, 0.13, 1.0), // #1a1b26
            foreground: hsla(232.0 / 360.0, 0.74, 0.85, 1.0), // #c0caf5
            cursor: hsla(232.0 / 360.0, 0.74, 0.85, 1.0), // #c0caf5
            selection_background: hsla(222.0 / 360.0, 0.40, 0.36, 0.40), // #364A82 with alpha

            ansi: TerminalAnsiColors {
                black: hsla(234.0 / 360.0, 0.22, 0.10, 1.0), // #15161e
                red: hsla(343.0 / 360.0, 0.88, 0.75, 1.0), // #f7768e
                green: hsla(83.0 / 360.0, 0.54, 0.66, 1.0), // #9ece6a
                yellow: hsla(34.0 / 360.0, 0.70, 0.65, 1.0), // #e0af68
                blue: hsla(217.0 / 360.0, 0.92, 0.78, 1.0), // #7aa2f7
                magenta: hsla(265.0 / 360.0, 0.85, 0.79, 1.0), // #bb9af7
                cyan: hsla(199.0 / 360.0, 1.0, 0.74, 1.0), // #7dcfff
                white: hsla(225.0 / 360.0, 0.27, 0.73, 1.0), // #a9b1d6

                bright_black: hsla(234.0 / 360.0, 0.20, 0.30, 1.0), // #414868
                bright_red: hsla(343.0 / 360.0, 0.93, 0.80, 1.0), // #ff7a93
                bright_green: hsla(83.0 / 360.0, 0.64, 0.72, 1.0), // #b9f27c
                bright_yellow: hsla(34.0 / 360.0, 0.80, 0.71, 1.0), // #ff9e64
                bright_blue: hsla(217.0 / 360.0, 0.97, 0.83, 1.0), // #7da6ff
                bright_magenta: hsla(265.0 / 360.0, 0.90, 0.84, 1.0), // #c0b2ff
                bright_cyan: hsla(199.0 / 360.0, 1.0, 0.80, 1.0), // #b4f9f8
                bright_white: hsla(232.0 / 360.0, 0.74, 0.90, 1.0), // #c0caf5
            },
        },
    };

    Theme::new("Tokyo Night", Appearance::Dark, colors)
}

/// 创建 Tokyo Night Light 主题
pub fn tokyo_night_light() -> Theme {
    let colors = ThemeColors {
        // Tokyo Night Light 基础颜色
        background: hsla(240.0 / 360.0, 0.09, 0.85, 1.0), // #d5d6db
        surface_background: hsla(240.0 / 360.0, 0.09, 0.89, 1.0), // #e1e2e7
        border: hsla(240.0 / 360.0, 0.08, 0.73, 1.0), // #b4b5b9
        border_variant: hsla(240.0 / 360.0, 0.08, 0.80, 1.0),
        text: hsla(228.0 / 360.0, 0.14, 0.39, 1.0), // #565a6e
        text_muted: hsla(228.0 / 360.0, 0.10, 0.56, 1.0), // #8990b3
        text_placeholder: hsla(228.0 / 360.0, 0.08, 0.65, 1.0),

        // 图标颜色
        icon: hsla(228.0 / 360.0, 0.14, 0.39, 1.0),       // 与文本颜色一致
        icon_muted: hsla(228.0 / 360.0, 0.10, 0.56, 1.0), // 与 text_muted 一致

        // 语义化颜色
        danger: hsla(349.0 / 360.0, 0.35, 0.41, 1.0),     // #8c4351 Tokyo Night Light 红
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),      // 白色

        // UI 组件颜色
        titlebar_background: hsla(240.0 / 360.0, 0.09, 0.87, 1.0), // #d9dae0
        tab_bar_background: hsla(240.0 / 360.0, 0.09, 0.87, 1.0),
        tab_active_background: hsla(240.0 / 360.0, 0.09, 0.92, 1.0), // 稍亮
        tab_inactive_background: hsla(240.0 / 360.0, 0.09, 0.85, 1.0),
        tab_hover_background: hsla(240.0 / 360.0, 0.09, 0.80, 1.0),
        tab_active_indicator: hsla(221.0 / 360.0, 0.44, 0.37, 1.0), // #34548a Tokyo Night Light 蓝
        button_hover_background: hsla(240.0 / 360.0, 0.09, 0.78, 1.0),
        button_active_background: hsla(240.0 / 360.0, 0.09, 0.73, 1.0),
        statusbar_background: hsla(240.0 / 360.0, 0.09, 0.83, 1.0), // 状态栏背景

        // 菜单颜色
        menu_background: hsla(240.0 / 360.0, 0.09, 0.89, 1.0), // 与 surface_background 一致
        menu_border: hsla(240.0 / 360.0, 0.08, 0.73, 1.0), // 与 border 一致
        menu_item_hover_background: hsla(221.0 / 360.0, 0.44, 0.37, 1.0), // Tokyo Night Light 蓝色
        menu_item_hover_text: hsla(0.0, 0.0, 1.0, 1.0), // 白色文本
        menu_item_disabled_text: hsla(228.0 / 360.0, 0.10, 0.56, 1.0), // text_muted 颜色

        terminal: TerminalColors {
            background: hsla(240.0 / 360.0, 0.09, 0.85, 1.0), // #d5d6db
            foreground: hsla(228.0 / 360.0, 0.14, 0.39, 1.0), // #565a6e
            cursor: hsla(228.0 / 360.0, 0.14, 0.39, 1.0), // #565a6e
            selection_background: hsla(228.0 / 360.0, 0.50, 0.74, 0.30), // #99a7df with alpha

            ansi: TerminalAnsiColors {
                black: hsla(228.0 / 360.0, 0.14, 0.39, 1.0), // #565a6e
                red: hsla(349.0 / 360.0, 0.35, 0.41, 1.0), // #8c4351
                green: hsla(170.0 / 360.0, 0.30, 0.30, 1.0), // #33635c
                yellow: hsla(34.0 / 360.0, 0.75, 0.32, 1.0), // #8f5e15
                blue: hsla(221.0 / 360.0, 0.65, 0.42, 1.0), // 提高亮度使文字更清晰：0.26 -> 0.42，保持高饱和度：0.65
                magenta: hsla(256.0 / 360.0, 0.23, 0.37, 1.0), // #5a4a78
                cyan: hsla(203.0 / 360.0, 0.75, 0.24, 1.0), // #0f4b6e
                white: hsla(0.0, 0.0, 0.85, 1.0), // 提高亮度用于反显：0.60 -> 0.85

                bright_black: hsla(228.0 / 360.0, 0.12, 0.48, 1.0), // #6f7390
                bright_red: hsla(349.0 / 360.0, 0.45, 0.51, 1.0), // #c64a5a
                bright_green: hsla(170.0 / 360.0, 0.40, 0.40, 1.0), // #4d7c70
                bright_yellow: hsla(34.0 / 360.0, 0.85, 0.42, 1.0), // #b67d1e
                bright_blue: hsla(221.0 / 360.0, 0.70, 0.52, 1.0), // 提高亮度使文字更明亮：0.34 -> 0.52，保持高饱和度：0.70
                bright_magenta: hsla(256.0 / 360.0, 0.33, 0.47, 1.0), // #6f5c94
                bright_cyan: hsla(203.0 / 360.0, 0.85, 0.34, 1.0), // #166585
                bright_white: hsla(0.0, 0.0, 0.98, 1.0), // 提高亮度用于反显：0.30 -> 0.98（接近纯白）
            },
        },
    };

    Theme::new("Tokyo Night Light", Appearance::Light, colors)
}

/// 创建包含所有内置主题的注册表
pub fn create_builtin_registry() -> super::ThemeRegistry {
    let mut registry = super::ThemeRegistry::new();

    registry.register(default_dark());
    registry.register(github_dark());
    registry.register(github_light());
    registry.register(tokyo_night_dark());
    registry.register(tokyo_night_light());

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dark_theme() {
        let theme = default_dark();
        assert_eq!(theme.name(), "Default Dark");
        assert_eq!(theme.appearance(), Appearance::Dark);
        assert!(theme.appearance().is_dark());
    }

    #[test]
    fn test_github_dark_theme() {
        let theme = github_dark();
        assert_eq!(theme.name(), "GitHub Dark");
        assert_eq!(theme.appearance(), Appearance::Dark);
    }

    #[test]
    fn test_github_light_theme() {
        let theme = github_light();
        assert_eq!(theme.name(), "GitHub Light");
        assert_eq!(theme.appearance(), Appearance::Light);
        assert!(theme.appearance().is_light());
    }

    #[test]
    fn test_tokyo_night_dark_theme() {
        let theme = tokyo_night_dark();
        assert_eq!(theme.name(), "Tokyo Night");
        assert_eq!(theme.appearance(), Appearance::Dark);
        assert!(theme.appearance().is_dark());
    }

    #[test]
    fn test_tokyo_night_light_theme() {
        let theme = tokyo_night_light();
        assert_eq!(theme.name(), "Tokyo Night Light");
        assert_eq!(theme.appearance(), Appearance::Light);
        assert!(theme.appearance().is_light());
    }

    #[test]
    fn test_builtin_registry_has_five_themes() {
        let registry = create_builtin_registry();
        assert_eq!(registry.all().len(), 5);
    }

    #[test]
    fn test_builtin_registry_contains_all_themes() {
        let registry = create_builtin_registry();

        assert!(registry.get("Default Dark").is_some());
        assert!(registry.get("GitHub Dark").is_some());
        assert!(registry.get("GitHub Light").is_some());
        assert!(registry.get("Tokyo Night").is_some());
        assert!(registry.get("Tokyo Night Light").is_some());
    }

    #[test]
    fn test_builtin_registry_appearance_filter() {
        let registry = create_builtin_registry();

        let dark_themes = registry.by_appearance(Appearance::Dark);
        assert_eq!(dark_themes.len(), 3); // Default Dark + GitHub Dark + Tokyo Night

        let light_themes = registry.by_appearance(Appearance::Light);
        assert_eq!(light_themes.len(), 2); // GitHub Light + Tokyo Night Light
    }

    #[test]
    fn test_theme_colors_not_transparent() {
        let theme = default_dark();
        let colors = theme.colors();

        // 验证主要颜色不透明
        assert_eq!(colors.background.a, 1.0);
        assert_eq!(colors.text.a, 1.0);
        assert_eq!(colors.terminal.background.a, 1.0);
        assert_eq!(colors.terminal.foreground.a, 1.0);
    }

    #[test]
    fn test_ansi_colors_all_defined() {
        let theme = github_dark();
        let ansi = &theme.colors().terminal.ansi;

        // 验证所有 ANSI 颜色的 alpha 值都是 1.0 (不透明)
        assert_eq!(ansi.black.a, 1.0);
        assert_eq!(ansi.red.a, 1.0);
        assert_eq!(ansi.green.a, 1.0);
        assert_eq!(ansi.yellow.a, 1.0);
        assert_eq!(ansi.blue.a, 1.0);
        assert_eq!(ansi.magenta.a, 1.0);
        assert_eq!(ansi.cyan.a, 1.0);
        assert_eq!(ansi.white.a, 1.0);

        assert_eq!(ansi.bright_black.a, 1.0);
        assert_eq!(ansi.bright_red.a, 1.0);
        assert_eq!(ansi.bright_green.a, 1.0);
        assert_eq!(ansi.bright_yellow.a, 1.0);
        assert_eq!(ansi.bright_blue.a, 1.0);
        assert_eq!(ansi.bright_magenta.a, 1.0);
        assert_eq!(ansi.bright_cyan.a, 1.0);
        assert_eq!(ansi.bright_white.a, 1.0);
    }

    #[test]
    fn test_github_themes_color_contrast() {
        let dark = github_dark();
        let light = github_light();

        // 验证深色主题的背景比浅色主题暗
        assert!(dark.colors().background.l < light.colors().background.l);

        // 验证深色主题的文本比浅色主题亮
        assert!(dark.colors().text.l > light.colors().text.l);
    }
}
