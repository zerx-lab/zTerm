<p align="center">
  <img src="assets/icons/logo.svg" alt="zTerm Logo" width="128" height="128">
</p>

<h1 align="center">zTerm</h1>

<p align="center">
  <strong>下一代终端体验，为效率而生</strong>
</p>

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=flat-square" alt="Platform"></a>
  <a href="#"><img src="https://img.shields.io/badge/rust-1.85+-orange?style=flat-square&logo=rust" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-CC%20BY--NC%204.0-green?style=flat-square" alt="License"></a>
  <a href="#"><img src="https://img.shields.io/badge/status-WIP-red?style=flat-square" alt="Status"></a>
</p>

<p align="center">
  <a href="#-已实现功能">已实现功能</a> •
  <a href="#-从源码构建">从源码构建</a> •
  <a href="#-路线图">路线图</a> •
  <a href="#-许可证">许可证</a>
</p>

---

> **注意**: zTerm 目前正处于积极开发阶段，**暂不提供预编译的发布版本**。如需体验，请从源码构建。

---

## ✅ 已实现功能

### 终端核心

| 功能 | 描述 |
|---|---|
| **VT 终端模拟** | 基于 alacritty_terminal 实现完整的 VT100/VT220/xterm 转义序列支持 |
| **PTY 管理** | Windows ConPTY 和 Unix PTY 跨平台支持 |
| **事件批处理** | 4ms 事件批处理优化，减少 UI 更新频率，提升性能 |
| **滚动回滚** | 支持 10,000 行历史滚动缓冲区 |
| **自动 Shell 检测** | 自动检测系统默认 Shell (PowerShell/CMD/bash/zsh 等) |

### 用户界面

| 功能 | 描述 |
|---|---|
| **GPU 加速渲染** | 基于 GPUI 框架的高性能 GPU 加速渲染 |
| **自定义标题栏** | 美观的原生自定义标题栏，支持窗口控制按钮 |
| **多标签页** | 支持创建、关闭、切换多个终端标签页 |
| **自定义滚动条** | 平滑的终端滚动条，支持拖拽和点击定位 |

### 输入处理

| 功能 | 描述 |
|---|---|
| **完整键盘支持** | 支持所有常见按键、Ctrl 组合键、F1-F12 功能键 |
| **特殊键映射** | 正确处理 Home/End/PageUp/PageDown/Delete/Insert 等 |
| **IME 输入法** | 支持中文、日文、韩文等输入法组合输入 |
| **鼠标选择** | 支持鼠标拖拽选择终端文本 |

### 工作区管理

| 功能 | 描述 |
|---|---|
| **标签管理** | 新建标签 (`Ctrl+T`)、关闭标签 (`Ctrl+W`) |
| **标签切换** | 支持 `Ctrl+Tab` / `Ctrl+Shift+Tab` 切换标签 |
| **Shell 显示** | 标签标题显示当前 Shell 名称 |

---

## 🔧 从源码构建

**前置要求:**
- Rust 1.85+ (建议使用 rustup 安装)
- Git

```bash
# 克隆仓库
git clone https://github.com/user/zterm.git
cd zterm

# 构建项目
cargo build --release

# 运行
cargo run -p axon_app --release
```

---

## ⌨️ 快捷键

| 快捷键 | 功能 |
|---|---|
| `Ctrl + T` | 新建标签页 |
| `Ctrl + W` | 关闭当前标签页 |
| `Ctrl + Tab` | 切换到下一个标签页 |
| `Ctrl + Shift + Tab` | 切换到上一个标签页 |

---

## 🛣️ 路线图

| 状态 | 功能 | 描述 |
|:---:|---|---|
| ✅ | 终端核心 | PTY 管理、VT 解析、事件处理 |
| ✅ | GPU 渲染 | GPUI 框架集成、高性能渲染 |
| ✅ | 多标签页 | 标签创建、关闭、切换 |
| ✅ | 输入处理 | 键盘输入、IME 支持、鼠标选择 |
| ✅ | 滚动支持 | 历史滚动、滚动条交互 |
| 🔄 | 分屏布局 | 水平/垂直分屏 |
| 📋 | 主题切换 | 终端主题切换功能 |
| 📋 | 智能补全 | 命令自动补全和建议 |
| 📋 | 命令面板 | 快速访问所有功能 |
| 📋 | SSH 集成 | 内置 SSH 连接管理 |
| 📋 | AI 助手 | AI 驱动的命令帮助 |
| 📋 | 设置界面 | 图形化配置界面 |

<sub>✅ 已完成 &nbsp; 🔄 进行中 &nbsp; 📋 计划中</sub>

---

## 🏗️ 项目结构

```
zterm/
├── crates/
│   ├── axon_app/        # 主应用入口
│   ├── axon_terminal/   # 终端核心逻辑 (PTY、VT解析)
│   ├── axon_ui/         # UI 组件 (标签栏、终端视图、滚动条)
│   ├── axon_input/      # 输入处理
│   └── axon_common/     # 公共工具
└── assets/              # 资源文件 (字体、图标、主题)
```

---

## 🔧 技术栈

- **语言**: Rust (Edition 2024)
- **UI 框架**: [GPUI](https://github.com/zed-industries/zed) (来自 Zed 编辑器)
- **组件库**: [gpui-component](https://github.com/longbridge/gpui-component)
- **终端模拟**: [alacritty_terminal](https://github.com/alacritty/alacritty)

---

## 📄 许可证

zTerm 采用 [CC BY-NC 4.0](LICENSE) 许可证，允许非商业使用、修改和分发，禁止商业用途。
