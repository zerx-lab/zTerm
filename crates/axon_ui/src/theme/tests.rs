//! 主题系统测试

use super::*;
use gpui::hsla;

/// 创建测试用的终端颜色
fn test_terminal_colors() -> TerminalColors {
    TerminalColors {
        background: hsla(0.0, 0.0, 0.1, 1.0),
        foreground: hsla(0.0, 0.0, 0.9, 1.0),
        cursor: hsla(0.0, 0.0, 1.0, 1.0),
        selection_background: hsla(0.6, 0.5, 0.5, 0.3),
        ansi: TerminalAnsiColors {
            black: hsla(0.0, 0.0, 0.0, 1.0),
            red: hsla(0.0, 0.8, 0.5, 1.0),
            green: hsla(0.33, 0.8, 0.5, 1.0),
            yellow: hsla(0.16, 0.8, 0.5, 1.0),
            blue: hsla(0.6, 0.8, 0.5, 1.0),
            magenta: hsla(0.83, 0.8, 0.5, 1.0),
            cyan: hsla(0.5, 0.8, 0.5, 1.0),
            white: hsla(0.0, 0.0, 0.9, 1.0),
            bright_black: hsla(0.0, 0.0, 0.5, 1.0),
            bright_red: hsla(0.0, 0.9, 0.6, 1.0),
            bright_green: hsla(0.33, 0.9, 0.6, 1.0),
            bright_yellow: hsla(0.16, 0.9, 0.6, 1.0),
            bright_blue: hsla(0.6, 0.9, 0.6, 1.0),
            bright_magenta: hsla(0.83, 0.9, 0.6, 1.0),
            bright_cyan: hsla(0.5, 0.9, 0.6, 1.0),
            bright_white: hsla(0.0, 0.0, 1.0, 1.0),
        },
    }
}

/// 创建测试用的主题颜色
fn test_theme_colors() -> ThemeColors {
    ThemeColors {
        background: hsla(0.0, 0.0, 0.1, 1.0),
        surface_background: hsla(0.0, 0.0, 0.15, 1.0),
        border: hsla(0.0, 0.0, 0.3, 1.0),
        border_variant: hsla(0.0, 0.0, 0.2, 1.0),
        text: hsla(0.0, 0.0, 0.9, 1.0),
        text_muted: hsla(0.0, 0.0, 0.6, 1.0),
        text_placeholder: hsla(0.0, 0.0, 0.4, 1.0),
        terminal: test_terminal_colors(),

        // 图标颜色
        icon: hsla(0.0, 0.0, 0.8, 1.0),
        icon_muted: hsla(0.0, 0.0, 0.5, 1.0),

        // 语义化颜色
        danger: hsla(0.0, 0.75, 0.65, 1.0),
        danger_foreground: hsla(0.0, 0.0, 1.0, 1.0),

        // UI 组件颜色
        titlebar_background: hsla(0.0, 0.0, 0.12, 1.0),
        tab_bar_background: hsla(0.0, 0.0, 0.12, 1.0),
        tab_active_background: hsla(0.0, 0.0, 0.18, 1.0),
        tab_inactive_background: hsla(0.0, 0.0, 0.15, 1.0),
        tab_hover_background: hsla(0.0, 0.0, 0.24, 1.0),
        tab_active_indicator: hsla(0.6, 0.82, 0.66, 1.0),
        button_hover_background: hsla(0.0, 0.0, 0.24, 1.0),
        button_active_background: hsla(0.0, 0.0, 0.30, 1.0),
        statusbar_background: hsla(0.0, 0.0, 0.14, 1.0),

        // 菜单颜色
        menu_background: hsla(0.0, 0.0, 0.15, 1.0),
        menu_border: hsla(0.0, 0.0, 0.3, 1.0),
        menu_item_hover_background: hsla(0.6, 0.82, 0.66, 1.0),
        menu_item_hover_text: hsla(0.0, 0.0, 1.0, 1.0),
        menu_item_disabled_text: hsla(0.0, 0.0, 0.4, 1.0),
    }
}

#[test]
fn test_appearance_is_light() {
    assert!(Appearance::Light.is_light());
    assert!(!Appearance::Dark.is_light());
}

#[test]
fn test_appearance_is_dark() {
    assert!(!Appearance::Light.is_dark());
    assert!(Appearance::Dark.is_dark());
}

#[test]
fn test_theme_creation() {
    let theme = Theme::new("Test Theme", Appearance::Dark, test_theme_colors());

    assert_eq!(theme.name(), "Test Theme");
    assert_eq!(theme.appearance(), Appearance::Dark);
    assert!(theme.appearance().is_dark());
}

#[test]
fn test_theme_colors_access() {
    let theme = Theme::new("Test Theme", Appearance::Dark, test_theme_colors());
    let colors = theme.colors();

    // 验证基本颜色
    assert_eq!(colors.background, hsla(0.0, 0.0, 0.1, 1.0));
    assert_eq!(colors.text, hsla(0.0, 0.0, 0.9, 1.0));
}

#[test]
fn test_terminal_colors_in_theme() {
    let theme = Theme::new("Test Theme", Appearance::Dark, test_theme_colors());
    let terminal_colors = &theme.colors().terminal;

    // 验证终端基本颜色
    assert_eq!(terminal_colors.background, hsla(0.0, 0.0, 0.1, 1.0));
    assert_eq!(terminal_colors.foreground, hsla(0.0, 0.0, 0.9, 1.0));
    assert_eq!(terminal_colors.cursor, hsla(0.0, 0.0, 1.0, 1.0));

    // 验证 ANSI 颜色
    assert_eq!(terminal_colors.ansi.black, hsla(0.0, 0.0, 0.0, 1.0));
    assert_eq!(terminal_colors.ansi.red, hsla(0.0, 0.8, 0.5, 1.0));
    assert_eq!(terminal_colors.ansi.white, hsla(0.0, 0.0, 0.9, 1.0));
}

#[test]
fn test_registry_creation() {
    let registry = ThemeRegistry::new();
    assert_eq!(registry.all().len(), 0);
}

#[test]
fn test_registry_register_theme() {
    let mut registry = ThemeRegistry::new();
    let theme = Theme::new("Dark Theme", Appearance::Dark, test_theme_colors());

    registry.register(theme.clone());

    assert_eq!(registry.all().len(), 1);
    assert_eq!(registry.all()[0].name(), "Dark Theme");
}

#[test]
fn test_registry_get_theme_by_name() {
    let mut registry = ThemeRegistry::new();
    let theme = Theme::new("Dark Theme", Appearance::Dark, test_theme_colors());

    registry.register(theme.clone());

    let retrieved = registry.get("Dark Theme");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "Dark Theme");

    let not_found = registry.get("Non-existent Theme");
    assert!(not_found.is_none());
}

#[test]
fn test_registry_filter_by_appearance() {
    let mut registry = ThemeRegistry::new();

    // 添加深色主题
    registry.register(Theme::new(
        "Dark Theme 1",
        Appearance::Dark,
        test_theme_colors(),
    ));
    registry.register(Theme::new(
        "Dark Theme 2",
        Appearance::Dark,
        test_theme_colors(),
    ));

    // 添加浅色主题
    registry.register(Theme::new(
        "Light Theme 1",
        Appearance::Light,
        test_theme_colors(),
    ));

    let dark_themes = registry.by_appearance(Appearance::Dark);
    assert_eq!(dark_themes.len(), 2);
    assert!(dark_themes.iter().all(|t| t.appearance().is_dark()));

    let light_themes = registry.by_appearance(Appearance::Light);
    assert_eq!(light_themes.len(), 1);
    assert!(light_themes.iter().all(|t| t.appearance().is_light()));
}

#[test]
fn test_registry_multiple_themes() {
    let mut registry = ThemeRegistry::new();

    registry.register(Theme::new("Theme 1", Appearance::Dark, test_theme_colors()));
    registry.register(Theme::new(
        "Theme 2",
        Appearance::Light,
        test_theme_colors(),
    ));
    registry.register(Theme::new("Theme 3", Appearance::Dark, test_theme_colors()));

    assert_eq!(registry.all().len(), 3);

    // 验证可以通过名称获取每个主题
    assert!(registry.get("Theme 1").is_some());
    assert!(registry.get("Theme 2").is_some());
    assert!(registry.get("Theme 3").is_some());
}

#[test]
fn test_ansi_colors_completeness() {
    let terminal_colors = test_terminal_colors();
    let ansi = &terminal_colors.ansi;

    // 验证所有 16 种 ANSI 颜色都已定义
    let _colors = [
        ansi.black,
        ansi.red,
        ansi.green,
        ansi.yellow,
        ansi.blue,
        ansi.magenta,
        ansi.cyan,
        ansi.white,
        ansi.bright_black,
        ansi.bright_red,
        ansi.bright_green,
        ansi.bright_yellow,
        ansi.bright_blue,
        ansi.bright_magenta,
        ansi.bright_cyan,
        ansi.bright_white,
    ];

    // 测试通过编译即表示所有颜色都已定义
    assert_eq!(_colors.len(), 16);
}

#[test]
fn test_ui_colors_defined() {
    let colors = test_theme_colors();

    // 验证所有 UI 颜色都已定义且不透明
    assert_eq!(colors.titlebar_background.a, 1.0);
    assert_eq!(colors.tab_bar_background.a, 1.0);
    assert_eq!(colors.tab_active_background.a, 1.0);
    assert_eq!(colors.tab_inactive_background.a, 1.0);
    assert_eq!(colors.tab_hover_background.a, 1.0);
    assert_eq!(colors.tab_active_indicator.a, 1.0);
    assert_eq!(colors.button_hover_background.a, 1.0);
    assert_eq!(colors.button_active_background.a, 1.0);
    assert_eq!(colors.statusbar_background.a, 1.0);
}

#[test]
fn test_semantic_colors_defined() {
    let colors = test_theme_colors();

    // 验证图标颜色都已定义且不透明
    assert_eq!(colors.icon.a, 1.0);
    assert_eq!(colors.icon_muted.a, 1.0);

    // 验证危险操作颜色都已定义且不透明
    assert_eq!(colors.danger.a, 1.0);
    assert_eq!(colors.danger_foreground.a, 1.0);
}

#[test]
fn test_menu_colors_defined() {
    let colors = test_theme_colors();

    // 验证菜单颜色都已定义且不透明
    assert_eq!(colors.menu_background.a, 1.0);
    assert_eq!(colors.menu_border.a, 1.0);
    assert_eq!(colors.menu_item_hover_background.a, 1.0);
    assert_eq!(colors.menu_item_hover_text.a, 1.0);
    assert_eq!(colors.menu_item_disabled_text.a, 1.0);
}
