//! 内置主题

use super::{Appearance, Theme, ThemeColors};
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
        icon: hsla(0.0, 0.0, 0.80, 1.0),       // 默认图标颜色
        icon_muted: hsla(0.0, 0.0, 0.50, 1.0), // 次要图标颜色

        // 语义化颜色
        danger: hsla(355.0 / 360.0, 0.75, 0.65, 1.0), // 柔和红色
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),  // 白色

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
        menu_border: hsla(220.0 / 360.0, 0.13, 0.30, 1.0),     // 与 border 一致
        menu_item_hover_background: hsla(207.0 / 360.0, 0.82, 0.66, 1.0), // 蓝色高亮
        menu_item_hover_text: hsla(0.0, 0.0, 1.0, 1.0),        // 白色文本
        menu_item_disabled_text: hsla(0.0, 0.0, 0.45, 1.0),    // 灰色禁用文本

        // 滚动条颜色 - 更明显的可见度
        scrollbar_thumb_background: hsla(0.0, 0.0, 0.55, 0.5),           // 更亮的灰色，更高透明度
        scrollbar_thumb_hover_background: hsla(0.0, 0.0, 0.60, 0.7),     // 悬停时更明显
        scrollbar_thumb_active_background: hsla(0.0, 0.0, 0.70, 0.9),    // 拖拽时高亮
        scrollbar_thumb_border: hsla(0.0, 0.0, 0.0, 0.0),                // 透明边框
        scrollbar_track_background: hsla(0.0, 0.0, 0.15, 0.3),           // 轻微可见的轨道背景
        scrollbar_track_border: hsla(220.0 / 360.0, 0.13, 0.30, 1.0),    // 与 border 一致
    };

    Theme::new("Default Dark", Appearance::Dark, colors)
}

/// 创建 GitHub Dark 主题
pub fn github_dark() -> Theme {
    let colors = ThemeColors {
        // GitHub Dark 基础颜色
        background: hsla(220.0 / 360.0, 0.13, 0.09, 1.0), // #0d1117
        surface_background: hsla(220.0 / 360.0, 0.13, 0.13, 1.0), // #161b22
        border: hsla(215.0 / 360.0, 0.12, 0.22, 1.0),     // #30363d
        border_variant: hsla(215.0 / 360.0, 0.10, 0.18, 1.0),
        text: hsla(210.0 / 360.0, 0.24, 0.88, 1.0), // #c9d1d9
        text_muted: hsla(217.0 / 360.0, 0.10, 0.60, 1.0), // #8b949e
        text_placeholder: hsla(215.0 / 360.0, 0.08, 0.47, 1.0),

        // 图标颜色
        icon: hsla(210.0 / 360.0, 0.24, 0.88, 1.0), // 与文本颜色一致
        icon_muted: hsla(217.0 / 360.0, 0.10, 0.60, 1.0), // 与 text_muted 一致

        // 语义化颜色
        danger: hsla(0.0, 0.73, 0.62, 1.0), // #ff7b72 GitHub 红
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0), // 白色

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
        menu_border: hsla(215.0 / 360.0, 0.12, 0.22, 1.0),     // 与 border 一致
        menu_item_hover_background: hsla(212.0 / 360.0, 0.92, 0.62, 1.0), // GitHub 蓝色高亮
        menu_item_hover_text: hsla(0.0, 0.0, 1.0, 1.0),        // 白色文本
        menu_item_disabled_text: hsla(217.0 / 360.0, 0.10, 0.50, 1.0), // 灰色禁用文本

        // 滚动条颜色
        scrollbar_thumb_background: hsla(215.0 / 360.0, 0.12, 0.35, 0.4),
        scrollbar_thumb_hover_background: hsla(215.0 / 360.0, 0.12, 0.40, 0.6),
        scrollbar_thumb_active_background: hsla(215.0 / 360.0, 0.12, 0.45, 0.8),
        scrollbar_thumb_border: hsla(0.0, 0.0, 0.0, 0.0),
        scrollbar_track_background: hsla(0.0, 0.0, 0.0, 0.0),
        scrollbar_track_border: hsla(215.0 / 360.0, 0.12, 0.22, 1.0),
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
        icon: hsla(213.0 / 360.0, 0.18, 0.20, 1.0), // 与文本颜色一致
        icon_muted: hsla(213.0 / 360.0, 0.12, 0.45, 1.0), // 与 text_muted 一致

        // 语义化颜色
        danger: hsla(0.0, 0.67, 0.42, 1.0), // #cf222e GitHub 红
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0), // 白色

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

        // 滚动条颜色
        scrollbar_thumb_background: hsla(210.0 / 360.0, 0.18, 0.60, 0.3),
        scrollbar_thumb_hover_background: hsla(210.0 / 360.0, 0.18, 0.50, 0.5),
        scrollbar_thumb_active_background: hsla(210.0 / 360.0, 0.18, 0.40, 0.7),
        scrollbar_thumb_border: hsla(0.0, 0.0, 0.0, 0.0),
        scrollbar_track_background: hsla(0.0, 0.0, 0.0, 0.0),
        scrollbar_track_border: hsla(210.0 / 360.0, 0.18, 0.85, 1.0),
    };

    Theme::new("GitHub Light", Appearance::Light, colors)
}

/// 创建 Tokyo Night Dark 主题
pub fn tokyo_night_dark() -> Theme {
    let colors = ThemeColors {
        // Tokyo Night Dark 基础颜色
        background: hsla(234.0 / 360.0, 0.17, 0.13, 1.0), // #1a1b26
        surface_background: hsla(234.0 / 360.0, 0.20, 0.16, 1.0), // #16161e
        border: hsla(234.0 / 360.0, 0.15, 0.25, 1.0),     // #363b54
        border_variant: hsla(234.0 / 360.0, 0.13, 0.20, 1.0),
        text: hsla(232.0 / 360.0, 0.74, 0.85, 1.0), // #c0caf5
        text_muted: hsla(225.0 / 360.0, 0.27, 0.64, 1.0), // #787c99
        text_placeholder: hsla(225.0 / 360.0, 0.20, 0.50, 1.0),

        // 图标颜色
        icon: hsla(232.0 / 360.0, 0.74, 0.85, 1.0), // 与文本颜色一致
        icon_muted: hsla(225.0 / 360.0, 0.27, 0.64, 1.0), // 与 text_muted 一致

        // 语义化颜色
        danger: hsla(343.0 / 360.0, 0.88, 0.75, 1.0), // #f7768e Tokyo Night 红
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),  // 白色

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
        menu_border: hsla(234.0 / 360.0, 0.15, 0.25, 1.0),     // 与 border 一致
        menu_item_hover_background: hsla(217.0 / 360.0, 0.92, 0.78, 1.0), // Tokyo Night 蓝色
        menu_item_hover_text: hsla(234.0 / 360.0, 0.17, 0.13, 1.0), // 深色文本以保持对比
        menu_item_disabled_text: hsla(225.0 / 360.0, 0.27, 0.64, 1.0), // text_muted 颜色

        // 滚动条颜色
        scrollbar_thumb_background: hsla(234.0 / 360.0, 0.17, 0.35, 0.4),
        scrollbar_thumb_hover_background: hsla(234.0 / 360.0, 0.17, 0.40, 0.6),
        scrollbar_thumb_active_background: hsla(217.0 / 360.0, 0.92, 0.78, 0.8),
        scrollbar_thumb_border: hsla(0.0, 0.0, 0.0, 0.0),
        scrollbar_track_background: hsla(0.0, 0.0, 0.0, 0.0),
        scrollbar_track_border: hsla(234.0 / 360.0, 0.15, 0.25, 1.0),
    };

    Theme::new("Tokyo Night", Appearance::Dark, colors)
}

/// 创建 Tokyo Night Light 主题
pub fn tokyo_night_light() -> Theme {
    let colors = ThemeColors {
        // Tokyo Night Light 基础颜色
        background: hsla(240.0 / 360.0, 0.09, 0.85, 1.0), // #d5d6db
        surface_background: hsla(240.0 / 360.0, 0.09, 0.89, 1.0), // #e1e2e7
        border: hsla(240.0 / 360.0, 0.08, 0.73, 1.0),     // #b4b5b9
        border_variant: hsla(240.0 / 360.0, 0.08, 0.80, 1.0),
        text: hsla(228.0 / 360.0, 0.14, 0.39, 1.0), // #565a6e
        text_muted: hsla(228.0 / 360.0, 0.10, 0.56, 1.0), // #8990b3
        text_placeholder: hsla(228.0 / 360.0, 0.08, 0.65, 1.0),

        // 图标颜色
        icon: hsla(228.0 / 360.0, 0.14, 0.39, 1.0), // 与文本颜色一致
        icon_muted: hsla(228.0 / 360.0, 0.10, 0.56, 1.0), // 与 text_muted 一致

        // 语义化颜色
        danger: hsla(349.0 / 360.0, 0.35, 0.41, 1.0), // #8c4351 Tokyo Night Light 红
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),  // 白色

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
        menu_border: hsla(240.0 / 360.0, 0.08, 0.73, 1.0),     // 与 border 一致
        menu_item_hover_background: hsla(221.0 / 360.0, 0.44, 0.37, 1.0), // Tokyo Night Light 蓝色
        menu_item_hover_text: hsla(0.0, 0.0, 1.0, 1.0),        // 白色文本
        menu_item_disabled_text: hsla(228.0 / 360.0, 0.10, 0.56, 1.0), // text_muted 颜色

        // 滚动条颜色
        scrollbar_thumb_background: hsla(240.0 / 360.0, 0.09, 0.50, 0.3),
        scrollbar_thumb_hover_background: hsla(240.0 / 360.0, 0.09, 0.45, 0.5),
        scrollbar_thumb_active_background: hsla(221.0 / 360.0, 0.44, 0.37, 0.7),
        scrollbar_thumb_border: hsla(0.0, 0.0, 0.0, 0.0),
        scrollbar_track_background: hsla(0.0, 0.0, 0.0, 0.0),
        scrollbar_track_border: hsla(240.0 / 360.0, 0.08, 0.73, 1.0),
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


#[cfg(test)]
mod scrollbar_tests {
    use super::*;

    #[test]
    fn test_default_dark_scrollbar_colors() {
        let theme = default_dark();
        let colors = theme.colors();

        // 验证滚动条颜色已定义且具有合理的 alpha 值
        assert!(colors.scrollbar_thumb_background.a > 0.0);
        assert!(colors.scrollbar_thumb_background.a < 1.0); // 应该是半透明
        assert!(colors.scrollbar_thumb_hover_background.a > colors.scrollbar_thumb_background.a);
        assert!(colors.scrollbar_thumb_active_background.a > colors.scrollbar_thumb_hover_background.a);

        // 轨道背景应该是透明的
        // Track background is now slightly visible for better UX
        assert!(colors.scrollbar_track_background.a > 0.0 && colors.scrollbar_track_background.a < 0.5);
    }

    #[test]
    fn test_github_dark_scrollbar_colors() {
        let theme = github_dark();
        let colors = theme.colors();

        assert!(colors.scrollbar_thumb_background.a > 0.0);
        assert!(colors.scrollbar_thumb_background.a < 1.0);
    }

    #[test]
    fn test_github_light_scrollbar_colors() {
        let theme = github_light();
        let colors = theme.colors();

        assert!(colors.scrollbar_thumb_background.a > 0.0);
        assert!(colors.scrollbar_thumb_background.a < 1.0);
    }

    #[test]
    fn test_tokyo_night_dark_scrollbar_colors() {
        let theme = tokyo_night_dark();
        let colors = theme.colors();

        assert!(colors.scrollbar_thumb_background.a > 0.0);
        assert!(colors.scrollbar_thumb_background.a < 1.0);
    }

    #[test]
    fn test_tokyo_night_light_scrollbar_colors() {
        let theme = tokyo_night_light();
        let colors = theme.colors();

        assert!(colors.scrollbar_thumb_background.a > 0.0);
        assert!(colors.scrollbar_thumb_background.a < 1.0);
    }

    #[test]
    fn test_all_themes_have_scrollbar_colors() {
        let registry = create_builtin_registry();

        for theme in registry.all() {
            let colors = theme.colors();

            // 验证所有主题都有滚动条颜色定义
            // 滑块颜色应该是半透明的 (0 < alpha < 1)
            assert!(
                colors.scrollbar_thumb_background.a > 0.0,
                "Theme {} should have scrollbar_thumb_background with alpha > 0",
                theme.name()
            );
            assert!(
                colors.scrollbar_thumb_background.a <= 1.0,
                "Theme {} should have scrollbar_thumb_background with alpha <= 1",
                theme.name()
            );

            // 悬停颜色应该比基础颜色更明显
            assert!(
                colors.scrollbar_thumb_hover_background.a >= colors.scrollbar_thumb_background.a,
                "Theme {} hover alpha should be >= base alpha",
                theme.name()
            );

            // 激活颜色应该比悬停颜色更明显
            assert!(
                colors.scrollbar_thumb_active_background.a >= colors.scrollbar_thumb_hover_background.a,
                "Theme {} active alpha should be >= hover alpha",
                theme.name()
            );
        }
    }
}