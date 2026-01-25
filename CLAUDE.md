## 项目概述

zTerm 是一个使用 Rust 和 GPUI 框架构建的现代跨平台终端模拟器。基于 Zed 编辑器的 UI 框架,集成了 Alacritty 的终端核心。

## 核心要求

- **语言**: 所有交互使用中文简体
- **验证方式**: 使用 `cargo check` 而非 `cargo build` 进行快速验证
- **Zed 源码路径**: C:\Users\zero\Desktop\code\github\zed
- **wezterm 源码路径**: C:\Users\zero\Desktop\code\github\wezterm

## 开发命令

### 编译和运行
```bash
# 快速检查编译错误 (推荐用于验证)
cargo check --workspace --all-targets --all-features

# 开发模式运行
cargo run -p z_term

# Release 模式运行
cargo run -p z_term --release

# 构建 Release 版本
cargo build --release

# 构建带调试信息的 Release 版本
cargo build --profile release-with-debug
```

### 代码质量
```bash
# 代码格式化
cargo fmt --all

# 格式检查 (不修改代码)
cargo fmt --all -- --check

# Clippy 静态分析
cargo clippy --workspace --all-targets --all-features

# 严格模式 Clippy
cargo clippy --workspace --all-targets --all-features -- \
  -D warnings \
  -W clippy::all \
  -W clippy::pedantic \
  -W clippy::nursery \
  -W clippy::cargo
```

### 测试
```bash
# 运行所有测试
cargo test --workspace --all-features

# 运行特定 crate 的测试
cargo test -p zterm_terminal
cargo test -p zterm_ui
cargo test -p axon_ui

# 带详细输出的测试
cargo test --workspace --all-features -- --nocapture

# 生成测试覆盖率 (需要 cargo-llvm-cov)
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

### 依赖管理
```bash
# 检查安全漏洞和许可证 (需要 cargo-deny)
cargo deny check advisories
cargo deny check licenses
cargo deny check sources
cargo deny check bans

# 检测未使用的依赖 (需要 cargo-machete)
cargo machete
```

### 文档
```bash
# 生成文档 (包括私有项)
cargo doc --workspace --all-features --no-deps --document-private-items

# 生成并打开文档
cargo doc --workspace --all-features --no-deps --document-private-items --open
```

## 代码架构

### Workspace 结构

zTerm 使用 Cargo workspace 管理多个 crate,各 crate 职责明确:

```
crates/
├── z_term/          - 应用入口,窗口管理,workspace 管理
├── zterm_terminal/  - 终端核心引擎 (PTY, VT 解析, shell 集成)
├── zterm_ui/        - UI 组件 (TerminalView, TitleBar, TabBar, Scrollbar)
├── zterm_input/     - 输入处理 (keybindings, history, completion)
├── zterm_common/    - 公共工具 (配置, 日志, 错误处理)
└── axon_ui/         - 主题系统 (颜色管理, 内置主题, 热重载)
```

### 关键架构组件

#### 1. 终端核心 (`zterm_terminal`)

- **Terminal Entity**: 终端状态管理,基于 GPUI 的 Entity 系统
- **PtyEventLoop**: 自定义 PTY 事件循环,扩展 Alacritty 的实现
  - 集成 OSC 133/633 扫描器用于 shell 集成
  - 支持事件批处理 (4ms 间隔) 降低 UI 刷新频率
  - 使用 `polling` crate 处理 PTY I/O
- **Shell Integration**: VSCode shell integration 支持
  - OscScanner: 扫描 OSC 133/633 序列
  - ZoneInfo: 跟踪命令执行区域 (prompt/command/output)
  - 支持右键菜单、AI 上下文提取等功能

#### 2. UI 组件 (`zterm_ui`)

- **TerminalView**: 主终端视图组件
  - 管理滚动状态、文本选择、IME 输入
  - 使用 TerminalElement 进行 GPU 渲染
  - 输入批处理 (4ms 间隔) 优化性能
  - 集成 ScrollbarElement 和 ContextMenu
- **TitleBar**: 自定义标题栏,跨平台窗口控制
- **TerminalTabBar**: 标签页管理 (新建、关闭、切换)
- **TerminalElement**: 低级渲染元素,使用 GPUI 的 paint API

#### 3. 主题系统 (`axon_ui`)

- **ThemeManager**: 全局主题管理器
  - 启动时自动加载用户自定义主题
  - 内置 5 个主题 (Default Dark, GitHub Dark/Light, Tokyo Night/Light)
  - 用户主题目录: `~/.config/zterm/themes/` (Linux/macOS) 或 `%APPDATA%\zterm\themes\` (Windows)
- **ThemeLoader**: 主题加载器
  - 支持从 JSON 文件加载主题
  - 支持三种颜色格式:HEX、RGBA 数组、HSLA 对象
  - 自动扫描主题目录并加载所有 `.json` 文件
- **ThemeContext**: GPUI 全局上下文,提供主题访问
- **Theme**: 完整的颜色定义
  - TerminalColors: 终端特定颜色 (background, foreground, cursor)
  - TerminalAnsiColors: 完整 ANSI 16 色
  - ThemeColors: UI 组件颜色 (titlebar, tabbar, menu 等)

#### 4. Workspace 管理 (`z_term`)

- **Workspace**: 管理多个终端标签页
  - TabInfo: 封装 TerminalView + Terminal entity
  - 标签生命周期管理 (创建、关闭、切换)
  - 默认终端尺寸管理
- **ZTermApp**: 应用入口
  - GPUI Application 初始化
  - 设置热重载
  - 窗口生命周期管理

### 核心依赖关系

```
z_term (入口)
  ├── zterm_ui (UI 组件)
  │   ├── zterm_terminal (终端核心)
  │   └── axon_ui (主题)
  ├── zterm_terminal
  ├── zterm_input
  ├── zterm_common
  └── axon_ui

外部核心依赖:
- gpui: Zed 的 UI 框架 (Git 依赖)
- gpui-component: UI 组件库 (Git 依赖)
- alacritty_terminal: 终端模拟 (VT 解析)
- portable-pty: 跨平台 PTY 支持
```

### 配置系统

- **位置**: `~/.config/zterm/config.toml`
- **AppSettings**: GPUI 全局设置,支持热重载
- **配置热重载**: 使用 `notify` crate 监听文件变化
- **主题切换**: 修改 `[ui] theme = "主题名"` 后自动重载

### 日志系统

- **位置**: `~/.local/share/zterm/logs/` (或 Windows: `AppData/Local/zterm/logs/`)
- **LogGuard**: 管理文件和控制台日志
- **启动性能追踪**: 使用 `mark_phase()` 和 `log_startup_phases()`

## 开发约定

### GPUI 模式

1. **Entity 系统**: Terminal, TerminalView 等都是 GPUI Entity
2. **Context 类型**:
   - `Context<T>`: Entity 上下文
   - `WindowContext`: 窗口上下文
   - `AppContext`: 应用全局上下文
3. **事件订阅**: 使用 `cx.subscribe()` 订阅 Entity 事件
4. **通知更新**: `cx.notify()` 触发 UI 重绘

### 终端渲染流程

1. PtyEventLoop 从 PTY 读取数据
2. OscScanner 扫描 shell integration 序列
3. VTE parser 解析 ANSI 转义序列,更新 Term 状态
4. TerminalEvent 通过 EventListener 发送到 TerminalView
5. TerminalView 触发重绘,TerminalElement 执行 GPU 渲染

### 输入处理流程

1. GPUI KeyDown/KeyUp/Input 事件
2. TerminalView 批处理输入 (4ms 间隔)
3. 通过 PtyEventLoop 发送 Msg::Input
4. PtyEventLoop 写入 PTY
5. Shell 接收并处理输入

### 测试策略

- **单元测试**: 每个 crate 的 `tests/` 目录或 `#[cfg(test)]` 模块
- **集成测试**: `z_term/tests/`
- **主题测试**: `axon_ui/src/theme/tests.rs` 验证颜色定义

## 常见任务

### 添加新主题

#### 方式一:创建 JSON 主题文件 (推荐)

1. 在主题目录创建 `.json` 文件:
   - Linux/macOS: `~/.config/zterm/themes/my-theme.json`
   - Windows: `%APPDATA%\zterm\themes\my-theme.json`
2. 参考 `examples/themes/` 中的示例编写主题
3. 支持三种颜色格式:
   - HEX: `"#282c34"`
   - RGBA: `[40, 44, 52, 1.0]`
   - HSLA: `{"h": 220, "s": 0.13, "l": 0.18, "a": 1.0}`
4. 在 `config.toml` 中设置 `theme = "你的主题名称"`

示例主题:
```json
{
  "name": "My Custom Theme",
  "appearance": "Dark",
  "colors": {
    "background": "#282c34",
    "text": "#abb2bf",
    "terminal": {
      "background": "#282c34",
      "foreground": "#abb2bf",
      "ansi": { ... }
    }
  }
}
```

完整文档见: `examples/themes/README.md`

#### 方式二:添加内置主题

1. 在 `axon_ui/src/theme/builtin.rs` 定义新主题
2. 注册到 `builtin::create_builtin_registry()`
3. 运行 `cargo test -p axon_ui` 验证

### 修改快捷键

1. 编辑 `~/.config/zterm/config.toml` 的 `[keybindings]` 部分
2. 或在代码中修改 `z_term/src/window/main_window.rs` 的 action 绑定

### 添加新 UI 组件

1. 在 `zterm_ui/src/components/` 创建新组件
2. 实现 `Render` trait
3. 在 `components/mod.rs` 导出
4. 在需要的地方集成 (如 TerminalView 或 Workspace)

### 调试终端解析问题

1. 查看日志: 日志文件在 `~/.local/share/zterm/logs/`
2. 启用 VT 解析日志: 在 `zterm_terminal/src/terminal/pty_loop.rs` 添加 trace 日志
3. 使用 `RUST_LOG=trace cargo run -p z_term` 运行

## 平台特定

### Windows
- 使用 ConPTY (platform/windows.rs)
- 窗口子系统: Release 构建时使用 `windows_subsystem = "windows"`
- 图标: build.rs 使用 winresource 嵌入图标

### Unix (Linux/macOS)
- 使用 PTY (platform/unix.rs)
- Shell 检测: 读取 `$SHELL` 环境变量

### Linux 额外依赖
```bash
sudo apt-get install -y libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

## CI/CD

GitHub Actions workflow 位于 `.github/workflows/`:

### 自动化流程
- `release.yml`: 发布构建流程

### Claude 集成
- `claude.yml`: 通过 `@claude` 提及触发的通用 Claude 助手
- `claude-code-review.yml`: PR 代码自动审查

### 本地开发检查
推荐在提交前本地运行以下命令确保代码质量：
```bash
# 代码格式检查
cargo fmt --all -- --check

# 静态分析
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 运行测试
cargo test --workspace --all-features

# 快速编译检查
cargo check --workspace --all-targets --all-features
```

## 技术栈版本

- Rust: 1.85+ (Edition 2024)
- GPUI: Git
- Alacritty Terminal: 0.25
- Portable PTY: 0.8

## 许可证

CC BY-NC-SA-4.0 (非商业使用)
