# zTerm 主题配置指南

zTerm 支持通过 JSON 文件自定义主题,让你可以创建符合个人喜好的配色方案。

## 快速开始

### 1. 主题目录位置

用户主题文件存放在以下目录:

- **Linux/macOS**: `~/.config/zterm/themes/`
- **Windows**: `%APPDATA%\zterm\themes\`

首次运行 zTerm 时会自动创建该目录。

### 2. 创建自定义主题

1. 在主题目录中创建一个 `.json` 文件,例如 `my-theme.json`
2. 参考示例主题编写配置
3. 重启 zTerm 或等待主题热重载生效
4. 在配置文件 `config.toml` 中设置 `theme = "你的主题名称"`

## 主题文件结构

```json
{
  "name": "主题名称",
  "appearance": "Dark",  // 或 "Light"
  "colors": {
    // 基础颜色
    "background": "#282c34",
    "surface_background": "#21252b",
    "border": "#181a1f",
    "text": "#abb2bf",

    // 终端颜色
    "terminal": {
      "background": "#282c34",
      "foreground": "#abb2bf",
      "cursor": "#528bff",
      "selection_background": "#67769660",
      "ansi": {
        "black": "#3f4451",
        "red": "#e06c75",
        // ... 更多颜色
      }
    }
  }
}
```

## 支持的颜色格式

zTerm 支持三种颜色格式,可以混合使用:

### 1. HEX 格式 (推荐)

最常见的格式,支持 6 位和 8 位(带透明度):

```json
{
  "background": "#282c34",           // 6位 HEX
  "selection_background": "#67769660" // 8位 HEX (带透明度)
}
```

### 2. RGBA 数组

使用数组格式,RGB 值为 0-255,Alpha 值为 0-1:

```json
{
  "background": [40, 44, 52, 1.0],
  "selection_background": [103, 118, 150, 0.38]
}
```

### 3. HSLA 对象

便于调整明度和饱和度:

```json
{
  "background": {"h": 220, "s": 0.13, "l": 0.18, "a": 1.0},
  "border": {"h": 220, "s": 0.13, "l": 0.25, "a": 1.0}
}
```

- `h` (色相): 0-360
- `s` (饱和度): 0-1
- `l` (亮度): 0-1
- `a` (透明度): 0-1

## 完整颜色定义

### UI 组件颜色

```json
{
  "colors": {
    // 基础颜色
    "background": "#282c34",              // 主背景色
    "surface_background": "#21252b",      // 表面背景(面板、卡片)
    "border": "#181a1f",                  // 边框颜色
    "border_variant": "#3e4451",          // 边框变体(分隔线)
    "text": "#abb2bf",                    // 主文本颜色
    "text_muted": "#5c6370",              // 次要文本
    "text_placeholder": "#4b5263",        // 占位符文本

    // 图标颜色
    "icon": "#abb2bf",                    // 默认图标
    "icon_muted": "#5c6370",              // 次要图标

    // 语义化颜色
    "danger": "#e06c75",                  // 危险操作(删除、关闭等)
    "danger_foreground": "#ffffff",       // 危险操作前景色

    // UI 组件
    "titlebar_background": "#21252b",     // 标题栏背景
    "tab_bar_background": "#21252b",      // 标签栏背景
    "tab_active_background": "#282c34",   // 激活标签背景
    "tab_inactive_background": "#21252b", // 非激活标签背景
    "tab_hover_background": "#2c313a",    // 标签悬停背景
    "tab_active_indicator": "#528bff",    // 激活标签指示器
    "button_hover_background": "#2c313a", // 按钮悬停背景
    "button_active_background": "#3e4451",// 按钮激活背景
    "statusbar_background": "#21252b",    // 状态栏背景

    // 菜单颜色
    "menu_background": "#282c34",         // 菜单背景
    "menu_border": "#181a1f",             // 菜单边框
    "menu_item_hover_background": "#2c313a",     // 菜单项悬停背景
    "menu_item_hover_text": "#ffffff",           // 菜单项悬停文本
    "menu_item_disabled_text": "#4b5263"         // 禁用菜单项文本
  }
}
```

### 终端颜色

```json
{
  "terminal": {
    "background": "#282c34",              // 终端背景
    "foreground": "#abb2bf",              // 终端前景(默认文本)
    "cursor": "#528bff",                  // 光标颜色
    "selection_background": "#67769660",  // 选中文本背景

    // ANSI 16 色调色板
    "ansi": {
      // 标准色
      "black": "#3f4451",
      "red": "#e06c75",
      "green": "#98c379",
      "yellow": "#e5c07b",
      "blue": "#61afef",
      "magenta": "#c678dd",
      "cyan": "#56b6c2",
      "white": "#dcdfe4",

      // 亮色
      "bright_black": "#4f5666",
      "bright_red": "#e06c75",
      "bright_green": "#98c379",
      "bright_yellow": "#e5c07b",
      "bright_blue": "#61afef",
      "bright_magenta": "#c678dd",
      "bright_cyan": "#56b6c2",
      "bright_white": "#abb2bf"
    }
  }
}
```

## 示例主题

本目录提供了三个示例主题供参考:

1. **example-hex-theme.json** - One Dark Pro 风格,使用 HEX 格式
2. **example-rgba-theme.json** - GitHub Light 风格,使用 RGBA 数组格式
3. **example-mixed-theme.json** - Tokyo Night 风格,混合使用三种格式

## 主题开发技巧

### 1. 使用取色工具

推荐使用以下工具获取颜色值:
- [Coolors](https://coolors.co/) - 在线配色方案生成器
- [Adobe Color](https://color.adobe.com/) - Adobe 配色工具
- VS Code 内置取色器

### 2. 保持对比度

确保文本和背景之间有足够的对比度,推荐:
- 普通文本对比度 ≥ 4.5:1
- 大号文本对比度 ≥ 3:1

### 3. 测试明暗模式

如果创建浅色主题,确保设置正确的 `appearance`:

```json
{
  "name": "My Light Theme",
  "appearance": "Light",  // 重要!
  "colors": { ... }
}
```

### 4. 从内置主题开始

内置主题包括:
- Default Dark
- GitHub Dark
- GitHub Light
- Tokyo Night
- Tokyo Night Light

你可以基于这些主题进行修改。

## 常见问题

### Q: 如何重新加载主题?

A: 修改主题文件后,重启 zTerm 即可。未来版本将支持主题热重载。

### Q: 主题文件加载失败怎么办?

A: 检查以下几点:
1. JSON 格式是否正确(可以使用 [JSONLint](https://jsonlint.com/) 验证)
2. 颜色值格式是否正确
3. 查看 zTerm 日志了解详细错误信息

### Q: 可以覆盖内置主题吗?

A: 可以。创建与内置主题同名的 JSON 文件,用户主题会优先加载。

### Q: 如何分享我的主题?

A: 将主题 JSON 文件分享给其他用户,放入他们的主题目录即可使用。

## 主题配色参考

### One Dark Pro (推荐)
- 来源: [Binaryify/OneDark-Pro](https://github.com/Binaryify/OneDark-Pro)
- 背景: `#282c34`
- 前景: `#abb2bf`
- 特点: 低对比度,舒适护眼

### Dracula
- 来源: [dracula/dracula-theme](https://draculatheme.com/)
- 背景: `#282a36`
- 前景: `#f8f8f2`
- 特点: 高对比度,鲜艳配色

### Nord
- 来源: [Nord Theme](https://www.nordtheme.com/)
- 背景: `#2e3440`
- 前景: `#d8dee9`
- 特点: 冷色调,极简设计

## 贡献主题

如果你创建了优秀的主题,欢迎通过 PR 贡献到 zTerm 仓库,让更多用户使用!

## 参考资源

- [HSLA 颜色转换器](https://www.w3schools.com/colors/colors_hsl.asp)
- [对比度检查工具](https://webaim.org/resources/contrastchecker/)
- [终端颜色方案集合](https://terminal.sexy/)
