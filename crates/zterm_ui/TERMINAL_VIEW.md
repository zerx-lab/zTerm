# TerminalView 组件

TerminalView 是一个 GPUI 组件,实现了类似 Warp 终端的块状渲染功能。每个命令及其输出被渲染为一个独立的视觉块。

## 功能特性

- ✅ **块状渲染**: 每个命令和输出被分组为独立的视觉块
- ✅ **状态指示**: 通过颜色区分命令执行状态(执行中/成功/失败)
- ✅ **元数据展示**: 显示工作目录、退出码、执行时长等信息
- ✅ **主题集成**: 与 axon_ui 主题系统完全集成
- ✅ **条件编译**: 通过 `shell-integration` feature 控制

## 架构设计

### 数据流

```
PowerShell/Bash Script
    ↓ (发送 OSC 序列)
Terminal PTY
    ↓ (解析 OSC)
BlockManager
    ↓ (管理 CommandBlock)
TerminalView
    ↓ (GPUI 渲染)
用户界面
```

### 核心组件

1. **OSC 序列** (`zterm_terminal::shell_integration`)
   - OSC 133: FinalTerm/VSCode 标准协议
   - OSC 531: zTerm 自定义 JSON 数据传输
   - OSC 7: 工作目录通知

2. **CommandBlock** (`zterm_terminal::shell_integration::block`)
   - 命令块数据模型
   - 包含命令、参数、输出、元数据等

3. **BlockManager** (`zterm_terminal::shell_integration::block`)
   - 自动管理块的生命周期
   - 响应 OSC 序列更新块状态

4. **TerminalView** (`zterm_ui::components::terminal_view`)
   - GPUI RenderOnce 组件
   - 块状渲染实现

## 使用示例

### 基本用法

TerminalView 是一个 RenderOnce 组件,主要用于在其他组件的 render 函数中内嵌渲染:

```rust
use zterm_ui::TerminalView;
use zterm_terminal::shell_integration::CommandBlock;
use gpui::*;

impl Render for MyTerminalPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 从某处获取命令块(例如从 BlockManager)
        let blocks = self.get_command_blocks();

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                // 直接在 render 函数中使用 TerminalView
                TerminalView::new()
                    .blocks(blocks)
                    .block_mode(true)
            )
    }
}
```

**注意**: TerminalView 实现了 `RenderOnce` trait,而不是 Entity。它应该在 render 函数中直接使用,而不是通过 `cx.new()` 创建为 Entity。

### 与 BlockManager 集成

```rust
use zterm_terminal::shell_integration::{BlockManager, OscSequence};

let mut manager = BlockManager::new();

// 处理 OSC 序列
manager.handle_osc_sequence(&OscSequence::PromptStart {
    aid: Some("cmd_001".to_string()),
    json: None,
});

manager.handle_osc_sequence(&OscSequence::CommandStart {
    aid: Some("cmd_001".to_string()),
});

manager.handle_osc_sequence(&OscSequence::CommandExecuting {
    aid: Some("cmd_001".to_string()),
});

manager.handle_osc_sequence(&OscSequence::CommandFinished {
    exit_code: Some(0),
    aid: Some("cmd_001".to_string()),
    json: None,
});

// 获取所有块
let blocks = manager.get_blocks().to_vec();

// 在 UI 中渲染
TerminalView::new()
    .blocks(blocks)
    .block_mode(true)
```

## 块状渲染结构

每个命令块包含三个部分:

### 1. 块头部 (Header)
- **状态指示器**: 圆形图标,颜色表示状态
  - 灰色: 执行中
  - 绿色: 成功 (exit code 0)
  - 红色: 失败 (exit code != 0)
- **工作目录**: 显示命令执行的目录
- **命令文本**: 命令名称和参数
- **退出码**: 右侧显示退出状态

### 2. 块内容 (Output)
- 显示命令的标准输出和错误输出
- 使用等宽字体
- 深色背景,与头部区分

### 3. 块尾部 (Footer)
- **执行时长**: 毫秒或秒
- **输出行数**: 总行数统计

## 样式定制

组件使用 `axon_ui::ThemeColors` 进行样式控制:

```rust
// 主题颜色
colors.background    // 背景色
colors.text          // 文本色
colors.text_muted    // 次要文本色
colors.border        // 边框色

// 状态颜色 (硬编码)
rgb(0x22c55e)        // 成功 (green-500)
rgb(0xef4444)        // 失败 (red-500)
rgb(0x1e293b)        // 块头部背景 (slate-800)
rgb(0x0f172a)        // 块内容背景 (slate-900)
```

## PowerShell 集成

使用 `examples/shell-integration/zterm-integration.ps1` 脚本:

```powershell
# 加载集成脚本
. .\examples\shell-integration\zterm-integration.ps1

# 脚本会自动发送 OSC 序列:
# - Prompt Start (OSC 133;A)
# - Command Start (OSC 133;B)
# - Command Executing (OSC 133;C)
# - Command Finished (OSC 133;D)
# - JSON Metadata (OSC 531)
# - Working Directory (OSC 7)
```

## 测试

### 单元测试

```bash
# 测试 shell integration 解析
cargo test -p zterm_terminal --lib shell_integration

# 测试块管理器
cargo test -p zterm_terminal block_manager

# 测试 JSON 类型
cargo test -p zterm_terminal json_types
```

### 集成测试

TerminalView 组件在主应用中集成使用。运行主应用查看效果:

```bash
# 运行主应用
cargo run -p z_term
```

### PowerShell 集成测试

```powershell
# 运行 OSC 序列测试脚本
.\examples\shell-integration\test_osc_sequences.ps1
```

## Feature Flags

- `shell-integration`: 启用块状渲染功能 (默认启用)
  - 禁用时,TerminalView 显示占位符文本

```toml
[dependencies]
zterm_ui = { version = "0.1.0", default-features = false }  # 禁用
zterm_ui = { version = "0.1.0", features = ["shell-integration"] }  # 启用
```

## 性能考虑

### 块数量限制

使用 `BlockManager::trim_blocks()` 限制内存占用:

```rust
// 保留最近 100 个块
manager.trim_blocks(100);
```

### 滚动渲染

TerminalView 使用 `.overflow_y_scroll()` 实现虚拟滚动,避免渲染所有块。

## 下一步计划

- [ ] 集成到主 Workspace
- [ ] 添加块折叠/展开功能
- [ ] 支持块内容搜索
- [ ] 添加块导出功能 (复制、保存)
- [ ] 实现块级别的右键菜单
- [ ] 支持语法高亮
- [ ] 添加块级别的性能指标
- [ ] 实现块之间的关系可视化

## 相关文档

- [Shell Integration 协议规范](../../crates/zterm_terminal/src/shell_integration/json_osc_protocol.md)
- [PowerShell 集成脚本](../../examples/shell-integration/README.md)
- [主题系统](../../crates/axon_ui/README.md)
- [GPUI 组件开发](.claude/skills/gpui-component/instructions.md)

## 问题排查

### 块不显示

1. 确保 `shell-integration` feature 已启用
2. 检查 `block_mode` 是否为 `true`
3. 确认 `blocks` 列表不为空

### OSC 序列未被识别

1. 检查 PowerShell 脚本是否正确加载
2. 使用 `test_osc_sequences.ps1` 验证序列格式
3. 查看 `zterm_terminal` 日志输出

### 样式问题

1. 确认主题系统已初始化: `cx.set_global(ThemeManager::new())`
2. 检查主题配置文件格式
3. 使用内置主题进行对比测试
