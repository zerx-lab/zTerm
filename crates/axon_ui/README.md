# Axon UI - 主题系统

基于 TDD 开发的主题系统，提供完整的颜色管理和终端主题支持。

## 功能特性

- ✅ 完整的主题颜色定义
- ✅ 终端 ANSI 16 色支持
- ✅ 内置 3 个主题：Default Dark、GitHub Dark、GitHub Light
- ✅ 主题注册表管理
- ✅ 99%+ 测试覆盖率

## 集成到应用

主题系统已经集成到 zTerm 应用中：

1. **自动初始化**: 应用启动时自动加载主题系统
2. **配置驱动**: 通过 `config.toml` 的 `ui.theme` 字段选择主题
3. **热重载**: 修改配置文件后主题自动更新，无需重启

### 配置主题

编辑 `~/.config/zterm/config.toml`:

```toml
[ui]
theme = "GitHub Dark"  # 可选: "Default Dark", "GitHub Dark", "GitHub Light"
```

保存后应用会自动重新加载主题并刷新所有窗口。

## 快速开始

### 使用内置主题

```rust
use axon_ui::builtin;

// 创建包含所有内置主题的注册表
let registry = builtin::create_builtin_registry();

// 获取主题
let dark_theme = registry.get("Default Dark").unwrap();
let github_dark = registry.get("GitHub Dark").unwrap();
let github_light = registry.get("GitHub Light").unwrap();

// 使用主题颜色
let bg_color = dark_theme.colors().background;
let text_color = dark_theme.colors().text;
let terminal_red = dark_theme.colors().terminal.ansi.red;
```

### 创建自定义主题

```rust
use axon_ui::{Theme, ThemeColors, TerminalColors, TerminalAnsiColors, Appearance};
use gpui::hsla;

let colors = ThemeColors {
    background: hsla(0.0, 0.0, 0.1, 1.0),
    surface_background: hsla(0.0, 0.0, 0.15, 1.0),
    border: hsla(0.0, 0.0, 0.3, 1.0),
    border_variant: hsla(0.0, 0.0, 0.2, 1.0),
    text: hsla(0.0, 0.0, 0.9, 1.0),
    text_muted: hsla(0.0, 0.0, 0.6, 1.0),
    text_placeholder: hsla(0.0, 0.0, 0.4, 1.0),
    terminal: TerminalColors {
        background: hsla(0.0, 0.0, 0.1, 1.0),
        foreground: hsla(0.0, 0.0, 0.9, 1.0),
        cursor: hsla(0.5, 0.8, 0.6, 1.0),
        selection_background: hsla(0.5, 0.8, 0.6, 0.3),
        ansi: TerminalAnsiColors {
            // ... 定义 16 种 ANSI 颜色
            black: hsla(0.0, 0.0, 0.0, 1.0),
            red: hsla(0.0, 0.8, 0.5, 1.0),
            // ... 更多颜色
        },
    },
};

let theme = Theme::new("My Custom Theme", Appearance::Dark, colors);
```

### 主题注册表

```rust
use axon_ui::ThemeRegistry;

let mut registry = ThemeRegistry::new();

// 注册主题
registry.register(my_theme);

// 查询主题
let theme = registry.get("My Custom Theme");

// 根据外观模式过滤
let dark_themes = registry.by_appearance(Appearance::Dark);
let light_themes = registry.by_appearance(Appearance::Light);

// 获取所有主题
let all_themes = registry.all();
```

## 主题结构

### ThemeColors

主题的基础颜色定义：

- `background` - 主背景色
- `surface_background` - 表面背景色（面板、卡片等）
- `border` - 边框颜色
- `border_variant` - 边框变体颜色（分隔线）
- `text` - 主文本颜色
- `text_muted` - 次要文本颜色
- `text_placeholder` - 占位符文本颜色
- `terminal` - 终端颜色配置

### TerminalColors

终端特定的颜色：

- `background` - 终端背景色
- `foreground` - 终端前景色（默认文本）
- `cursor` - 光标颜色
- `selection_background` - 选中文本背景色
- `ansi` - 16 色 ANSI 颜色表

### TerminalAnsiColors

完整的 ANSI 16 色支持：

**标准颜色 (8 色)**:
- black, red, green, yellow, blue, magenta, cyan, white

**亮色 (8 色)**:
- bright_black, bright_red, bright_green, bright_yellow
- bright_blue, bright_magenta, bright_cyan, bright_white

## 内置主题

### Default Dark
经典深色主题，适合长时间编码使用。

### GitHub Dark
GitHub 官方深色主题配色，与 GitHub 界面保持一致。

### GitHub Light
GitHub 官方浅色主题配色，清新明亮。

## 测试

```bash
# 运行测试
cargo test -p axon_ui

# 查看测试覆盖率
cargo llvm-cov --package axon_ui --lib
```

当前测试覆盖率：**99%+**

## License

CC-BY-NC-SA-4.0
