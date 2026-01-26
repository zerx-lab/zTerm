# zterm_terminal

终端后端核心库，为 zTerm 提供底层终端支持。

## 特性

- ✅ **PTY 管理**：基于 `portable-pty` 的跨平台伪终端支持
- ✅ **VTE 解析**：基于 `vte` crate 的 ANSI 转义序列解析
- ✅ **Shell Integration**：支持 OSC 133/633 shell integration 协议
- ✅ **Event 系统**：使用 `flume` channel 的高效事件通知
- ✅ **多线程架构**：独立的读、写、监控线程

## 架构设计

### 核心组件

```
┌─────────────────────────────────────────────────────────┐
│                      Terminal                            │
│  - VTE Parser                                            │
│  - VTE Performer                                         │
│  - Event Receiver                                        │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ├─ PTY Message Channel (Input/Resize/Shutdown)
                   │
                   ├─ Event Channel (Wakeup/Title/Bell/Exit)
                   │
┌──────────────────▼──────────────────────────────────────┐
│                  PtyEventLoop                            │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────┐│
│  │  Reader    │  │  Writer    │  │  Child Monitor     ││
│  │  Thread    │  │  Thread    │  │  Thread            ││
│  └─────┬──────┘  └─────┬──────┘  └──────┬─────────────┘│
└────────┼───────────────┼────────────────┼──────────────┘
         │               │                │
         │               │                │
┌────────▼───────────────▼────────────────▼──────────────┐
│                    Pty (portable-pty)                   │
│  ┌──────────┐                         ┌──────────────┐ │
│  │  Reader  │ ◄────────────────────► │    Writer    │ │
│  └──────────┘                         └──────────────┘ │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              Shell Process (bash/zsh/...)               │
└─────────────────────────────────────────────────────────┘
```

### 数据流

#### 1. 输入流（用户 → Shell）

```
User Input
    ↓
Terminal::write()
    ↓
PtyMessage::Input
    ↓
Writer Thread
    ↓
Pty Writer
    ↓
Shell Process
```

#### 2. 输出流（Shell → 用户）

```
Shell Process
    ↓
Pty Reader
    ↓
Reader Thread
    ↓
[可选] OscScanner (拦截 OSC 133/633)
    ↓
TerminalEvent::Wakeup
    ↓
Terminal::process_pty_data()
    ↓
VTE Parser → VTE Performer
    ↓
TerminalEvent (TitleChanged/BellRing/...)
    ↓
UI Layer (GPUI)
```

## 使用示例

### 基本使用

```rust
use zterm_terminal::{Terminal, PtyConfig, TerminalConfig, TerminalSize};

// 创建 PTY 配置
let pty_config = PtyConfig {
    shell: None,  // 使用系统默认 shell
    shell_args: vec![],
    working_directory: None,
    env: vec![],
    initial_size: TerminalSize::new(24, 80),
};

// 创建终端配置
let term_config = TerminalConfig::default();

// 创建终端实例
let terminal = Terminal::new(pty_config, term_config)?;

// 获取事件接收器
let event_rx = terminal.event_receiver();

// 写入数据
terminal.write(b"ls -la\n")?;

// 调整尺寸
terminal.resize(TerminalSize::new(40, 120))?;

// 监听事件
while let Ok(event) = event_rx.recv() {
    match event {
        TerminalEvent::Wakeup => {
            // 重绘终端
        }
        TerminalEvent::TitleChanged(title) => {
            println!("Title: {}", title);
        }
        TerminalEvent::ProcessExit { exit_code } => {
            println!("Process exited: {:?}", exit_code);
            break;
        }
        _ => {}
    }
}
```

### Shell Integration

启用 `shell-integration` feature (默认启用):

```toml
[dependencies]
zterm_terminal = { version = "0.1", features = ["shell-integration"] }
```

然后在 shell 配置中添加：

```bash
# bash/zsh
source /path/to/zterm/shell-integration/zterm.sh

# fish
source /path/to/zterm/shell-integration/zterm.fish
```

监听 shell integration 事件：

```rust
if let TerminalEvent::ShellIntegration(shell_event) = event {
    match shell_event {
        ShellIntegrationEvent::PromptStart { line } => {
            println!("Prompt started at line {}", line);
        }
        ShellIntegrationEvent::CommandFinished { line, exit_code } => {
            println!("Command finished at line {} with code {:?}", line, exit_code);
        }
        _ => {}
    }
}
```

## API 文档

### `Terminal`

主终端结构，管理 PTY 和 VTE 解析器。

**方法**：
- `new(pty_config, term_config) -> Result<Self>` - 创建新终端
- `write(&self, data: &[u8]) -> Result<()>` - 写入数据到 PTY
- `resize(&self, size: TerminalSize) -> Result<()>` - 调整终端尺寸
- `size(&self) -> TerminalSize` - 获取当前尺寸
- `event_receiver(&self) -> Receiver<TerminalEvent>` - 获取事件接收器
- `process_pty_data(&self, data: &[u8])` - 处理从 PTY 读取的数据
- `shutdown(&self) -> Result<()>` - 关闭终端

### `PtyConfig`

PTY 配置。

**字段**：
- `shell: Option<PathBuf>` - Shell 程序路径
- `shell_args: Vec<String>` - Shell 参数
- `working_directory: Option<PathBuf>` - 工作目录
- `env: Vec<(String, String)>` - 环境变量
- `initial_size: TerminalSize` - 初始尺寸

### `TerminalConfig`

终端配置。

**字段**：
- `scrollback_lines: usize` - 最大滚动回溯行数（0 = 无限制，默认 10000）
- `enable_shell_integration: bool` - 是否启用 shell integration（默认 true）
- `input_batch_ms: u64` - 输入批处理间隔（默认 4ms）
- `event_batch_ms: u64` - 事件批处理间隔（默认 4ms）

### `TerminalEvent`

终端事件枚举。

**变体**：
- `Wakeup` - 终端需要重绘
- `TitleChanged(String)` - 窗口标题改变
- `BellRing` - 响铃
- `ClipboardCopy(String)` - 剪贴板复制
- `ClipboardPaste(String)` - 剪贴板粘贴
- `Resized { rows, cols }` - 终端尺寸改变
- `ShellIntegration(ShellIntegrationEvent)` - Shell integration 事件（需要 feature）
- `ProcessExit { exit_code }` - PTY 进程退出
- `Error(Arc<anyhow::Error>)` - 错误事件

## 线程模型

### Reader Thread

- 从 PTY 读取数据（8KB 缓冲区）
- TODO: 调用 OscScanner 拦截 shell integration
- 发送 `Wakeup` 事件通知 UI
- 处理 EOF 和错误

### Writer Thread

- 接收 `PtyMessage`（Input/Resize/Shutdown）
- 写入数据到 PTY
- 处理 PTY resize
- 响应 shutdown 信号

### Child Monitor Thread

- 定期检查子进程状态（100ms 间隔）
- 发送 `ProcessExit` 事件
- 清理资源

## 性能优化

- ✅ **批处理**：输入和事件批处理降低系统调用频率
- ✅ **零拷贝**：OscScanner 不分配非 OSC 数据内存
- ✅ **单遍扫描**：O(n) 复杂度的 OSC 扫描
- ✅ **独立线程**：读写分离，避免阻塞
- ✅ **非阻塞 I/O**：使用 `WouldBlock` 处理

## TODO

- [ ] 实现完整的终端 Grid 管理（目前只有 VTE Performer 框架）
- [ ] 实现光标管理
- [ ] 实现选择和复制
- [ ] 完善 OSC 处理（剪贴板、颜色等）
- [ ] 添加 Sixel/Kitty 图像协议支持
- [ ] 实现 Zone 管理（shell integration 块功能）
- [ ] 添加更多单元测试
- [ ] 性能基准测试

## 许可证

CC BY-NC-SA-4.0
