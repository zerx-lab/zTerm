# JSON OSC 数据传输协议

## 概述

本协议扩展了 OSC 133/633 标准,支持通过 OSC 序列传输结构化 JSON 数据,实现类似 Warp 的块状渲染。

## 设计原则

1. **向后兼容**: 完全兼容现有 OSC 133/633 序列
2. **渐进增强**: JSON 数据作为可选扩展,非 JSON 客户端仍可正常工作
3. **高效传输**: 使用 Base64 编码避免转义字符冲突
4. **类型安全**: JSON Schema 定义明确的数据结构

## 协议规范

### 1. 扩展 OSC 133 - 块元数据传输

```
OSC 133 ; <command> ; aid=<block_id> ; json=<base64_json> ST
```

**参数说明**:
- `command`: A/B/C/D (与 OSC 133 标准一致)
- `aid`: 块唯一标识符 (必需)
- `json`: Base64 编码的 JSON 元数据 (可选)

**示例**:
```
OSC 133 ; A ; aid=cmd_001 ; json=eyJjd2QiOiIvaG9tZS91c2VyIn0= ST
# 解码后: {"cwd":"/home/user"}
```

### 2. 新增 OSC 531 - 专用 JSON 数据通道

对于大量结构化数据,使用专用 OSC 代码 531 (zTerm 自定义):

```
OSC 531 ; <escaped_json> ST
```

**说明**:
- JSON 通过转义特殊字符直接传输
- 转义规则:控制字符(0x00-0x1f)、反斜杠、换行、分号 → \xHH 格式
- 比 Base64 更易调试,且兼容现有 shell integration 脚本

**示例**:
```
OSC 531 ; {"type":"command_start","command":"ls -la"} ST
```

## JSON 数据结构

### BlockMetadata (块元数据)

```json
{
  "block_id": "cmd_001",
  "start_time": "2026-01-26T10:00:00Z",
  "cwd": "/home/user",
  "env": {
    "PWD": "/home/user",
    "SHELL": "/bin/bash"
  },
  "user": "username",
  "hostname": "machine"
}
```

### CommandMetadata (命令元数据)

```json
{
  "block_id": "cmd_001",
  "command": "ls -la",
  "args": ["-l", "-a"],
  "raw_input": "ls -la",
  "timestamp": "2026-01-26T10:00:01Z"
}
```

### OutputMetadata (输出元数据)

```json
{
  "block_id": "cmd_001",
  "stream": "stdout",
  "line_count": 42,
  "byte_count": 2048,
  "format": "text",
  "encoding": "utf-8"
}
```

### CommandResult (命令结果)

```json
{
  "block_id": "cmd_001",
  "exit_code": 0,
  "end_time": "2026-01-26T10:00:02Z",
  "duration_ms": 1234,
  "signal": null,
  "error": null
}
```

## 完整命令流程示例

```bash
# 1. 开始新块 (OSC 133 A)
OSC 133 ; A ; aid=cmd_001 ; json=eyJjd2QiOiIvaG9tZS91c2VyIn0= ST

# 2. 渲染提示符
$

# 3. 用户输入开始 (OSC 133 B)
OSC 133 ; B ST

# 4. 输入命令
ls -la

# 5. 发送命令元数据 (OSC 51)
OSC 51 ; command_meta ; eyJibG9ja19pZCI6ImNtZF8wMDEiLCJjb21tYW5kIjoibHMgLWxhIn0= ST

# 6. 命令执行 (OSC 133 C)
OSC 133 ; C ; aid=cmd_001 ST

# 7. 输出内容
total 8
drwxr-xr-x  5 user  staff  160 Jan 26 10:00 .
...

# 8. 发送输出元数据 (OSC 51)
OSC 51 ; output_meta ; eyJibG9ja19pZCI6ImNtZF8wMDEiLCJsaW5lX2NvdW50Ijo0Mn0= ST

# 9. 命令完成 (OSC 133 D)
OSC 133 ; D ; 0 ; aid=cmd_001 ; json=eyJkdXJhdGlvbl9tcyI6MTIzNH0= ST
```

## 实现要点

### 解析器扩展

1. 在 `OscScanner` 中支持 `key=value` 参数解析
2. 识别 `json=` 参数并 Base64 解码
3. 添加 OSC 51 解析支持

### 数据模型

```rust
pub enum OscSequence {
    // 现有的 OSC 133/633
    PromptStart { aid: Option<String>, json: Option<BlockMetadata> },
    CommandStart { aid: Option<String> },
    CommandExecuting { aid: Option<String> },
    CommandFinished {
        exit_code: Option<i32>,
        aid: Option<String>,
        json: Option<CommandResult>
    },

    // 新增 OSC 51
    JsonData {
        data_type: JsonDataType,
        payload: serde_json::Value,
    },
}

pub enum JsonDataType {
    BlockMeta,
    CommandMeta,
    OutputMeta,
    Custom,
}
```

### Shell Integration 脚本

PowerShell 示例:

```powershell
function Send-BlockStart {
    param([string]$BlockId)

    $meta = @{
        block_id = $BlockId
        cwd = (Get-Location).Path
        timestamp = (Get-Date).ToUniversalTime().ToString("o")
    } | ConvertTo-Json -Compress

    $base64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($meta))

    Write-Host "`e]133;A;aid=$BlockId;json=$base64`a" -NoNewline
}
```

## 优势

1. **结构化数据**: 支持复杂元数据而无需解析文本
2. **可扩展**: JSON 格式易于添加新字段
3. **类型安全**: Schema 验证确保数据正确性
4. **向后兼容**: 非 JSON 客户端忽略 `json=` 参数
5. **高效**: Base64 编码避免转义问题

## 参考

- [FinalTerm Semantic Prompt](https://gitlab.freedesktop.org/Per_Bothner/specifications/blob/master/proposals/semantic-prompts.md)
- [VSCode Shell Integration](https://code.visualstudio.com/docs/terminal/shell-integration)
- [Warp Terminal Blocks](https://docs.warp.dev/terminal/blocks)
- [WezTerm OSC 133](https://wezfurlong.org/wezterm/shell-integration.html)
