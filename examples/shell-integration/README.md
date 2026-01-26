# zTerm Shell Integration

PowerShell 集成脚本,实现 Warp 风格的命令块系统。

**✅ 已对比 VS Code 实现并修复所有关键问题** - 详见 `COMPARISON.md`

## 功能特性

### 1. 命令块系统 (OSC 133)

使用标准 OSC 133 协议标记命令块边界:

```
┌─ OSC 133;A ─────────────────────────┐
│  提示符开始                          │
├─ OSC 133;B ─────────────────────────┤
│  命令输入开始                        │
├─ OSC 133;C ─────────────────────────┤
│  命令执行开始                        │
│  ... 命令输出 ...                    │
├─ OSC 133;D;exit_code ───────────────┤
│  命令执行完成                        │
└─────────────────────────────────────┘
```

### 2. JSON 元数据传输 (OSC 531)

使用自定义 OSC 531 协议传输丰富的元数据:

```json
{
  "type": "command_start",
  "command": "git status",
  "cwd": "C:/Projects/zterm",
  "timestamp": 1706284800000
}
```

### 3. 工作目录同步 (OSC 7)

使用标准 OSC 7 协议同步当前工作目录:

```
OSC 7 ; file://hostname/C:/Users/zero/Desktop/code/axon_term
```

## 使用方法

### 在 PowerShell 配置文件中加载

1. 编辑 PowerShell 配置文件:
   ```powershell
   notepad $PROFILE
   ```

2. 添加以下行:
   ```powershell
   . "C:\Users\zero\Desktop\code\axon_term\examples\shell-integration\zterm-integration.ps1"
   ```

3. 重新启动 PowerShell 或执行:
   ```powershell
   . $PROFILE
   ```

### 手动加载测试

```powershell
. .\examples\shell-integration\zterm-integration.ps1
```

## 协议详解

### OSC 133: 命令块边界标记

| 标记 | 含义 | 触发时机 |
|------|------|----------|
| `OSC 133;A` | 提示符开始 | 每次显示 prompt |
| `OSC 133;B` | 命令输入开始 | 提示符显示完成 |
| `OSC 133;C` | 命令执行前 | 用户按下 Enter |
| `OSC 133;D;code` | 命令执行完成 | 命令结束,返回退出码 |

### OSC 531: JSON 元数据事件

#### 事件类型

**1. shell_started** - Shell 启动
```json
{
  "type": "shell_started",
  "shell": "PowerShell",
  "version": "7.4.1",
  "pid": 12345,
  "os": "Microsoft Windows NT 10.0...",
  "timestamp": 1706284800000
}
```

**2. prompt_start** - 提示符显示
```json
{
  "type": "prompt_start",
  "cwd": "C:/Projects/zterm",
  "user": "zero",
  "hostname": "DESKTOP-XYZ",
  "git": {
    "branch": "main",
    "has_changes": true,
    "repo_root": "/c/Projects/zterm"
  },
  "timestamp": 1706284800000
}
```

**3. command_start** - 命令开始执行
```json
{
  "type": "command_start",
  "command": "cargo build",
  "cwd": "C:/Projects/zterm",
  "timestamp": 1706284800000
}
```

**4. command_end** - 命令执行完成
```json
{
  "type": "command_end",
  "exit_code": 0,
  "duration_ms": 1234,
  "timestamp": 1706284801234
}
```

**5. directory_changed** - 目录变更
```json
{
  "type": "directory_changed",
  "cwd": "C:/Projects/another-project",
  "git": {
    "branch": "dev",
    "has_changes": false,
    "repo_root": "/c/Projects/another-project"
  },
  "timestamp": 1706284800000
}
```

### OSC 7: 当前工作目录

标准协议,格式:
```
OSC 7 ; file://hostname/path/to/directory
```

终端可以使用此信息实现:
- 新标签页在相同目录打开
- 显示当前目录路径
- 文件路径点击跳转

## OSC 序号说明

### 为什么选择 531?

- **可自定义**: OSC 序号理论上可以是任意数字
- **避免冲突**: 531 不在标准序号范围内
- **易于识别**: 三位数,易记

### 标准 OSC 序号参考

| 序号 | 用途 |
|------|------|
| 0 | 设置窗口标题和图标 |
| 1 | 设置图标名称 |
| 2 | 设置窗口标题 |
| 4 | 设置/查询调色板颜色 |
| 7 | **当前工作目录** (标准) |
| 8 | 超链接 |
| 10-19 | 颜色查询/设置 |
| 52 | 剪贴板操作 |
| 133 | **Shell Integration** (VS Code, Warp) |
| 777 | iTerm2 通知 |
| **531** | **zTerm 自定义元数据** |

## 终端实现指南

zTerm 终端需要实现以下功能:

### 1. OSC 序列解析器

```rust
// 示例伪代码
match osc_number {
    7 => handle_working_directory(params),
    133 => handle_shell_integration(params),
    531 => handle_zterm_metadata(params),
    _ => {} // 忽略未知序号
}
```

### 2. 命令块管理

```rust
struct CommandBlock {
    prompt_start_line: usize,
    command_start_line: usize,
    execution_start_line: usize,
    execution_end_line: usize,
    exit_code: Option<i32>,
    metadata: BlockMetadata,
}
```

### 3. 元数据解析

```rust
fn handle_zterm_metadata(json: &str) {
    let metadata: Value = serde_json::from_str(json)?;
    match metadata["type"].as_str() {
        Some("command_start") => {
            // 记录命令信息
        },
        Some("command_end") => {
            // 更新块状态,显示执行时间
        },
        // ...
    }
}
```

## 测试方法

### 1. 捕获 OSC 序列

在非 zTerm 终端中运行脚本,OSC 序列会被忽略或显示为乱码。可以使用 `script` 命令捕获:

```bash
# Windows (需要 WSL)
script -c "pwsh -NoExit -Command '. ./zterm-integration.ps1'" typescript.log
```

### 2. 手动解析测试

使用 Python 脚本解析捕获的数据:

```python
import re
import json

data = open('typescript.log', 'rb').read()

# 查找 OSC 531 序列
pattern = rb'\x1b\]531;(.+?)\x1b\\'
matches = re.findall(pattern, data)

for match in matches:
    metadata = json.loads(match.decode('utf-8'))
    print(f"Type: {metadata['type']}")
    print(f"Data: {metadata}")
    print()
```

### 3. 在 zTerm 中测试

一旦 zTerm 实现了 OSC 解析:

1. 加载集成脚本
2. 执行命令: `ls`, `cd`, `git status` 等
3. 观察 zTerm 是否正确:
   - 标记命令块边界
   - 显示执行时间
   - 更新工作目录
   - 显示 Git 信息

## 故障排查

### 脚本未加载

检查 PowerShell 执行策略:
```powershell
Get-ExecutionPolicy
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### OSC 序列显示为乱码

正常现象,说明当前终端不支持这些 OSC 序列。在 zTerm 中应该正常工作。

### Git 信息不显示

确保在 Git 仓库中,且 `git` 命令可用:
```powershell
git --version
```

## 扩展开发

### 添加自定义事件

```powershell
# 在脚本中调用
Send-ZTermCustomData -Data @{
    event = "custom_action"
    data = "your data here"
}
```

### 集成到 zTerm UI

终端可以使用元数据实现:
- 命令执行时间显示
- 失败命令高亮 (exit_code != 0)
- Git 分支显示在标签页标题
- 目录导航历史
- 命令搜索和重放

## 相关资源

- [ANSI Escape Codes](https://en.wikipedia.org/wiki/ANSI_escape_code)
- [iTerm2 Shell Integration](https://iterm2.com/documentation-shell-integration.html)
- [VS Code Terminal Shell Integration](https://code.visualstudio.com/docs/terminal/shell-integration)
- [Warp Blocks](https://docs.warp.dev/features/blocks)

## 许可证

与 zTerm 项目相同: CC BY-NC-SA-4.0
