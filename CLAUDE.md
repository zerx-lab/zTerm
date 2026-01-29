# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

zTerm 是一个使用 Rust 和 GPUI 框架构建的现代跨平台终端模拟器。

**当前状态**:
- ✅ 终端核心 (`zterm_terminal`) - PTY 管理、VTE 解析、Shell Integration
- ✅ 主题系统 (`axon_ui`) - 5 个内置主题 + 自定义主题热重载
- ✅ UI 框架 (`z_term`, `zterm_ui`) - 标题栏、标签栏、窗口管理
- ⏳ **待集成**: 将终端核心连接到 UI 渲染

## 核心要求

- **语言**: 所有交互使用中文简体
- **可利用能力**: 充分利用已知skills和各种mcp能力
- **Rust开发**: 善于调用各种skills进行工作
- **任务规划**: 充分使用task进行规划和执行任务,每次执行前需要思考是否需要设计task,并且需要检查是否存在未完成task,对于不理解的地方需要停下来询问
- **单元测试**: 每个task或者todo完成需要编写对应的单元测试进行验证,并且单元测试需要完全考虑各种边界与功能,而不是单纯的写完代码和测试通过
- **验证**: 使用 `cargo check` 进行快速验证
- **参考源码**:
  - Zed: `C:\Users\zero\Desktop\code\github\zed` (GPUI 用法)
  - WezTerm: `C:\Users\zero\Desktop\code\github\wezterm` (计划中的迁移目标)
  - 其他: `C:\Users\zero\Desktop\code\github` 目录下可自行clone任何项目进行调研

## 常用命令

### 编译和运行
```bash
# 快速检查 (推荐)
cargo check --workspace --all-targets --all-features

# 运行主应用
cargo run -p z_term

# 运行终端核心示例
cargo run --example basic -p zterm_terminal
```

### 代码质量
```bash
cargo fmt --all                    # 格式化
cargo clippy --workspace          # 静态分析
```

### 测试
```bash
cargo test --workspace --all-features
cargo test -p zterm_terminal       # 测试特定 crate
```

## Workspace 结构

```
crates/
├── z_term/          - 应用入口、窗口管理、workspace
├── zterm_terminal/  - ✅ 终端核心 (PTY, VTE, 事件, Shell Integration)
├── zterm_ui/        - UI 组件 (TitleBar, TabBar)
├── zterm_common/    - 公共工具 (配置热重载、日志、错误)
└── axon_ui/         - 主题系统
```

**依赖关系**:
```
z_term
  ├── zterm_terminal (终端核心)
  ├── zterm_ui (UI) → axon_ui (主题)
  └── zterm_common (配置、日志)
```

## 核心架构

### 1. 终端核心 (`zterm_terminal`)

**主要文件**:
- `terminal.rs` - Terminal 实例,管理 PTY、VTE 解析器、事件
- `pty/mod.rs` - PTY 系统初始化、Shell 检测
- `pty/event_loop.rs` - PTY 读写循环
- `vte_bridge.rs` - VTE Performer,处理转义序列
- `shell_integration/scanner.rs` - OSC 133/531/7 序列解析
- `event.rs` - TerminalEvent 定义

**关键 API**:
```rust
// 创建终端
let terminal = Terminal::new(pty_config, term_config)?;

// 写入数据
terminal.write(b"echo hello\n")?;

// 监听事件
let rx = terminal.event_receiver();
while let Ok(event) = rx.recv() {
    match event {
        TerminalEvent::Wakeup => { /* 重绘 */ }
        TerminalEvent::TitleChanged(title) => { /* 更新标题 */ }
        // ...
    }
}

// Resize
terminal.resize(TerminalSize::new(24, 80))?;
```

### 2. 主题系统 (`axon_ui`)

- **ThemeManager** (Global): 管理内置和自定义主题
- **ThemeContext**: `cx.current_theme()` 获取当前主题
- **ThemeLoader**: 从 `~/.config/zterm/themes/*.json` 加载
- **内置主题**: Default Dark, GitHub Dark/Light, Tokyo Night/Light

### 3. 配置系统 (`zterm_common`)

- **AppSettings** (GPUI Global): 配置热重载
- **Config**: `~/.config/zterm/config.toml`
  ```toml
  [ui]
  theme = "Default Dark"

  [terminal]
  font_size = 14.0
  scrollback_lines = 10000
  ```
- **LogGuard**: 日志到 `~/.local/share/zterm/logs/` (Unix) 或 `%LOCALAPPDATA%\zterm\logs\` (Windows)

### 4. UI 层 (`z_term`, `zterm_ui`)

**当前实现**:
- `Workspace` Entity: 管理标签页
- `MainWindow`: 渲染标题栏、标签栏
- `TitleBar`, `TerminalTabBar`: UI 组件

**⚠️ 待实现**:
- `TerminalView` 组件: 渲染终端内容
- 将 `Terminal` 实例集成到 `Workspace`
- 处理键盘/鼠标输入到 PTY

## GPUI 核心模式

### Entity 系统
```rust
// Workspace 是 GPUI Entity
let workspace = cx.new(|cx| Workspace::new(cx));

// 通过 Context<T> 访问
impl Workspace {
    fn add_tab(&mut self, cx: &mut Context<Self>) {
        self.tabs.push(...);
        cx.notify();  // 触发重绘
    }
}
```

### Global 系统
```rust
// 初始化
cx.set_global(ThemeManager::new());

// 访问
let theme = cx.global::<ThemeManager>().current_theme();

// 更新
cx.update_global::<ThemeManager, _>(|mgr, cx| {
    mgr.set_theme("Tokyo Night");
    cx.refresh_windows();
});
```

### Context 类型
- `App`: 全局应用上下文
- `Window`: 窗口上下文,用于 UI 渲染
- `Context<T>`: Entity 上下文,结合 Window + Entity 状态

## 当前开发优先级

### 🔥 最优先: 集成终端核心到 UI

**Step 1**: 创建 `TerminalView` 组件
```rust
// crates/zterm_ui/src/components/terminal_view.rs
pub struct TerminalView {
    terminal: Arc<Terminal>,
    // ... 渲染状态
}

impl Render for TerminalView {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // 从 terminal 读取内容并渲染
    }
}
```

**Step 2**: 修改 `Workspace`
```rust
// crates/z_term/src/workspace/mod.rs
pub struct Workspace {
    tabs: Vec<TabInfo>,
    terminals: HashMap<TabId, Arc<Terminal>>,  // 新增
    // ...
}

impl Workspace {
    fn new_tab(&mut self, cx: &mut Context<Self>) {
        let terminal = Terminal::new(pty_config, term_config).unwrap();
        // 监听事件
        // 添加到 terminals map
    }
}
```

**Step 3**: 替换占位符
```rust
// crates/z_term/src/window/main_window.rs
// 将占位符 div() 替换为 TerminalView::new(terminal)
```

## 技术栈

**核心**:
- Rust 1.85+ (Edition 2024)
- GPUI (Git: zed-industries/zed)
- gpui-component (Git: longbridge/gpui-component)

**终端**:
- portable-pty 0.9.0 - 跨平台 PTY
- vte 0.15.0 - VT 解析
- alacritty_terminal 0.25 (⚠️ 计划迁移到 WezTerm,见 `MIGRATION_TO_WEZTERM.md`)

**异步**:
- tokio 1.0, smol 2.0, flume 0.11

**其他**:
- notify 8.0 - 文件监听
- tracing 0.1 - 日志
- parking_lot 0.12 - 同步原语

## Shell Integration

**协议**: OSC 133/531/7

**示例脚本**: `examples/shell-integration/zterm-integration.ps1`

**测试**:
```powershell
. .\examples\shell-integration\zterm-integration.ps1
# 执行命令会发送 OSC 序列
```

**文档**: `examples/shell-integration/README.md`

## 平台特定

### Windows
- 配置: `%APPDATA%\zterm\config.toml`
- 主题: `%APPDATA%\zterm\themes\`
- 日志: `%LOCALAPPDATA%\zterm\logs\`
- Release 构建隐藏控制台窗口

### Linux/macOS
- 配置: `~/.config/zterm/config.toml`
- 主题: `~/.config/zterm/themes/`
- 日志: `~/.local/share/zterm/logs/`
- Linux 依赖: `libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev`

## 开发约定

- **GPUI 技能**: 参考 `.claude/skills/gpui-*` 目录
- **组件创建**: 使用 `/new-component` skill
- **代码风格**: 使用 `/gpui-style-guide` skill
- **测试**: 添加单元测试和示例
- **文档**: 为新功能添加注释和 README

## 调试

### 查看日志
```bash
# Unix
tail -f ~/.local/share/zterm/logs/zterm-*.log

# Windows
Get-Content "$env:LOCALAPPDATA\zterm\logs\zterm-*.log" -Wait
```

### 启用详细日志
```bash
RUST_LOG=debug cargo run -p z_term
RUST_LOG=zterm_terminal=trace cargo run -p z_term
```

### 性能分析
启动性能追踪已内置,查看日志中的 "Startup phases" 输出。

## 相关文档

- `README.md` - 项目介绍和功能说明
- `examples/themes/README.md` - 主题创建指南
- `.claude/skills/` - GPUI 开发技能文档
