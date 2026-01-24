<p align="center">
  <img src="assets/icons/logo.svg" alt="zTerm Logo" width="128" height="128">
</p>

<h1 align="center">zTerm</h1>

<p align="center">
  <strong>高性能跨平台终端模拟器</strong>
</p>

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=flat-square" alt="Platform"></a>
  <a href="#"><img src="https://img.shields.io/badge/rust-1.85+-orange?style=flat-square&logo=rust" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-CC%20BY--NC%204.0-green?style=flat-square" alt="License"></a>
  <a href="#"><img src="https://img.shields.io/badge/status-Development-red?style=flat-square" alt="Status"></a>
</p>

<p align="center">
  <a href="#已实现功能">已实现功能</a> |
  <a href="#构建指南">构建指南</a> |
  <a href="#开发路线">开发路线</a> |
  <a href="#许可证">许可证</a>
</p>

---

> **声明**: 项目当前处于活跃开发阶段，暂未提供预编译版本。如需使用请参照构建指南从源码编译。

---

## 已实现功能

### 终端核心

| 功能 | 说明 |
|---|---|
| VT 终端模拟 | 基于 alacritty_terminal，完整支持 VT100/VT220/xterm 转义序列 |
| PTY 管理 | 跨平台伪终端支持 (Windows ConPTY / Unix PTY) |
| 事件批处理 | 4ms 批处理间隔，降低 UI 刷新频率，优化渲染性能 |
| 滚动缓冲 | 10,000 行历史记录缓冲区 |
| Shell 检测 | 自动识别系统默认 Shell (PowerShell/CMD/Bash/Zsh 等) |

### 用户界面

| 功能 | 说明 |
|---|---|
| GPU 加速渲染 | 基于 GPUI 框架实现硬件加速渲染 |
| 自定义标题栏 | 原生风格标题栏，集成窗口控制按钮 |
| 多标签页 | 支持标签页的创建、关闭、切换操作 |
| 自定义滚动条 | 支持拖拽定位和点击跳转 |
| 主题系统 | 5 个内置主题，支持完整的 ANSI 16 色配置，热重载 |

### 输入处理

| 功能 | 说明 |
|---|---|
| 键盘输入 | 完整支持常规按键、Ctrl 组合键、F1-F12 功能键 |
| 特殊按键 | 正确映射 Home/End/PageUp/PageDown/Delete/Insert 等按键 |
| 输入法支持 | 支持中文、日文、韩文等 IME 组合输入 |
| 鼠标选择 | 支持拖拽选择终端文本内容 |

### 工作区管理

| 功能 | 说明 |
|---|---|
| 标签管理 | 新建标签 (`Ctrl+T`)、关闭标签 (`Ctrl+W`) |
| 标签切换 | `Ctrl+Tab` / `Ctrl+Shift+Tab` 切换标签 |
| 状态显示 | 标签标题显示当前 Shell 名称 |

---

## 构建指南

### 环境要求

- Rust 1.85 或更高版本 (推荐使用 rustup 安装)
- Git

### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/user/zterm.git
cd zterm

# 编译 Release 版本
cargo build --release

# 运行
cargo run -p z_term --release
```

---

## 快捷键

| 按键 | 功能 |
|---|---|
| `Ctrl + T` | 新建标签页 |
| `Ctrl + W` | 关闭当前标签页 |
| `Ctrl + Tab` | 切换至下一标签页 |
| `Ctrl + Shift + Tab` | 切换至上一标签页 |

---

## 主题配置

zTerm 内置 5 个精心设计的主题，支持配置热重载，无需重启应用即可切换主题。

### 可用主题

| 主题名称 | 类型 | 说明 |
|---|---|---|
| `Default Dark` | 深色 | 经典深色主题，中性色调 |
| `GitHub Dark` | 深色 | GitHub 官方深色配色 |
| `GitHub Light` | 浅色 | GitHub 官方浅色配色 |
| `Tokyo Night` | 深色 | 紫蓝色调，柔和对比度 |
| `Tokyo Night Light` | 浅色 | Tokyo Night 日间变体 |

### 配置方法

编辑配置文件 `~/.config/zterm/config.toml`：

```toml
[ui]
theme = "Tokyo Night"  # 修改为你喜欢的主题名称
```

保存后应用会自动检测配置变更并重新加载主题，所有窗口会立即应用新主题。

---

## 开发路线

| 状态 | 功能 | 说明 |
|:---:|---|---|
| ✅ Done | 终端核心 | PTY 管理、VT 解析、事件处理 |
| ✅ Done | GPU 渲染 | GPUI 框架集成 |
| ✅ Done | 多标签页 | 标签创建、关闭、切换 |
| ✅ Done | 输入处理 | 键盘输入、IME 支持、鼠标选择 |
| ✅ Done | 滚动支持 | 历史滚动、滚动条交互 |
| ✅ Done | 主题系统 | 5 个内置主题，完整 ANSI 色彩，热重载支持 |
| 🚧 WIP | 分屏布局 | 水平/垂直分屏 |
| 📋 Planned | 智能补全 | 命令自动补全与建议 |
| 📋 Planned | 命令面板 | 快捷命令访问入口 |
| 📋 Planned | SSH 集成 | 内置 SSH 连接管理 |
| 📋 Planned | AI 辅助 | AI 驱动的命令帮助 |
| 📋 Planned | 配置界面 | 图形化设置面板 |

---

## 项目结构

```
zterm/
├── crates/
│   ├── z_term/          # 应用入口
│   ├── zterm_terminal/  # 终端核心 (PTY, VT 解析)
│   ├── zterm_ui/        # UI 组件 (标题栏, 终端视图, 滚动条)
│   ├── zterm_input/     # 输入处理
│   ├── zterm_common/    # 公共模块
│   └── axon_ui/         # 主题系统 (颜色管理, 内置主题)
└── assets/              # 静态资源 (字体, 图标)
```

---

## 技术栈

| 类别 | 技术 |
|---|---|
| 开发语言 | Rust (Edition 2024) |
| UI 框架 | [GPUI](https://github.com/zed-industries/zed) (Zed 编辑器) |
| 组件库 | [gpui-component](https://github.com/longbridge/gpui-component) |
| 终端模拟 | [alacritty_terminal](https://github.com/alacritty/alacritty) |

---

## 许可证

本项目采用 [CC BY-NC 4.0](LICENSE) 许可证，允许非商业性使用、修改和分发。
