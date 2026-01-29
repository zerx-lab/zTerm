//! OSC 序列扫描器
//!
//! 在 VTE 解析前拦截 OSC 133/633/531 序列，用于 shell integration

use super::json_types::{BlockMetadata, CommandResult, JsonDataType};

/// OSC 扫描器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// 正常状态
    Ground,
    /// 看到 ESC
    Escape,
    /// 看到 ESC ]
    OscStart,
    /// 收集 OSC 内容
    OscCollect,
    /// OSC 中的 ESC
    OscEscape,
}

/// OSC 序列
#[derive(Debug, Clone)]
pub enum OscSequence {
    /// OSC 133;A - 提示符开始 (扩展: 支持 aid 和 JSON 元数据)
    PromptStart {
        aid: Option<String>,
        json: Option<BlockMetadata>,
    },
    /// OSC 133;B - 命令开始
    CommandStart {
        aid: Option<String>,
    },
    /// OSC 133;C - 命令执行中
    CommandExecuting {
        aid: Option<String>,
    },
    /// OSC 133;D - 命令结束 (扩展: 支持 aid 和 JSON 元数据)
    CommandFinished {
        exit_code: Option<i32>,
        aid: Option<String>,
        json: Option<CommandResult>,
    },
    /// OSC 633;E - 命令文本
    CommandText(String),
    /// OSC 633;P;Cwd= - 工作目录
    WorkingDirectory(String),
    /// OSC 7 - 工作目录（另一种格式）
    Osc7WorkingDirectory(String),
    /// OSC 51 - JSON 数据通道
    JsonData {
        data_type: JsonDataType,
        payload: serde_json::Value,
    },
}

/// OSC 扫描器
pub struct OscScanner {
    state: State,
    osc_buffer: Vec<u8>,
    max_osc_len: usize,
    /// 缓冲区已满标志 - 当达到限制时设置,防止继续增长
    buffer_full: bool,
}

impl Default for OscScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl OscScanner {
    /// 创建新的 OSC 扫描器
    ///
    /// 参考 WezTerm 的设计:
    /// - 默认最大长度 4096 字节(可配置)
    /// - 超过限制时会重置状态机并记录警告
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            osc_buffer: Vec::with_capacity(256),
            max_osc_len: 4096,
            buffer_full: false,
        }
    }

    /// 使用自定义最大长度创建扫描器
    pub fn with_max_len(max_len: usize) -> Self {
        Self {
            state: State::Ground,
            osc_buffer: Vec::with_capacity(256.min(max_len)),
            max_osc_len: max_len,
            buffer_full: false,
        }
    }

    /// 扫描数据中的 OSC 序列
    ///
    /// 参考 WezTerm 的实现:
    /// - 当缓冲区达到最大长度时,设置 buffer_full 标志
    /// - buffer_full 后忽略所有后续数据,直到 OSC 结束
    /// - OSC 结束时重置状态并记录警告(如果发生溢出)
    pub fn scan(&mut self, data: &[u8]) -> Vec<OscSequence> {
        let mut sequences = Vec::new();

        for &byte in data {
            match self.state {
                State::Ground => {
                    if byte == 0x1b {
                        // ESC
                        self.state = State::Escape;
                    }
                }
                State::Escape => {
                    if byte == b']' {
                        // OSC start
                        self.state = State::OscStart;
                        self.osc_buffer.clear();
                        self.buffer_full = false; // 重置满标志
                    } else {
                        self.state = State::Ground;
                    }
                }
                State::OscStart => {
                    self.state = State::OscCollect;
                    // 检查缓冲区是否已满
                    if !self.buffer_full && self.osc_buffer.len() < self.max_osc_len {
                        self.osc_buffer.push(byte);
                    } else if !self.buffer_full {
                        // 第一次达到限制,设置标志并记录警告
                        self.buffer_full = true;
                        tracing::warn!(
                            "OSC buffer limit reached ({} bytes), ignoring further data",
                            self.max_osc_len
                        );
                    }
                }
                State::OscCollect => {
                    if byte == 0x07 {
                        // BEL - OSC 结束
                        if self.buffer_full {
                            tracing::debug!(
                                "OSC sequence ended after buffer overflow (discarded data)"
                            );
                        }
                        if let Some(seq) = self.parse_osc() {
                            sequences.push(seq);
                        }
                        // 重置缓冲区和状态,但保留 buffer_full 标志供外部检查
                        // buffer_full 将在下一个 OSC 序列开始时重置
                        self.osc_buffer.clear();
                        self.state = State::Ground;
                    } else if byte == 0x1b {
                        // 可能是 ST (ESC \)
                        self.state = State::OscEscape;
                    } else if !self.buffer_full {
                        // 只在未满时添加
                        if self.osc_buffer.len() < self.max_osc_len {
                            self.osc_buffer.push(byte);
                        } else {
                            // 达到限制
                            self.buffer_full = true;
                            tracing::warn!(
                                "OSC buffer limit reached ({} bytes), ignoring further data",
                                self.max_osc_len
                            );
                        }
                    }
                    // 如果 buffer_full == true,静默忽略后续数据
                }
                State::OscEscape => {
                    if byte == b'\\' {
                        // ST - OSC 结束
                        if self.buffer_full {
                            tracing::debug!(
                                "OSC sequence ended after buffer overflow (discarded data)"
                            );
                        }
                        if let Some(seq) = self.parse_osc() {
                            sequences.push(seq);
                        }
                        // 重置缓冲区和状态,但保留 buffer_full 标志供外部检查
                        self.osc_buffer.clear();
                        self.state = State::Ground;
                    } else {
                        // 不是 ST，继续收集
                        self.state = State::OscCollect;
                        if !self.buffer_full {
                            if self.osc_buffer.len() + 2 <= self.max_osc_len {
                                self.osc_buffer.push(0x1b);
                                self.osc_buffer.push(byte);
                            } else {
                                self.buffer_full = true;
                                tracing::warn!(
                                    "OSC buffer limit reached ({} bytes), ignoring further data",
                                    self.max_osc_len
                                );
                            }
                        }
                    }
                }
            }
        }

        sequences
    }

    /// 反转义 OSC 值 (逆向 __ZTerm-Escape-Value)
    /// 将 \xHH 格式转回原始字符
    fn unescape_value(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.peek() {
                    Some('x') => {
                        chars.next(); // 消费 'x'
                        // 读取两位十六进制数
                        let hex: String = chars.by_ref().take(2).collect();
                        if hex.len() == 2 {
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                result.push(byte as char);
                                continue;
                            }
                        }
                        // 如果解析失败,恢复原始字符
                        result.push('\\');
                        result.push('x');
                        result.push_str(&hex);
                    }
                    Some('n') => {
                        chars.next();
                        result.push('\n');
                    }
                    Some('\\') => {
                        chars.next();
                        result.push('\\');
                    }
                    _ => result.push(ch),
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// 解析 key=value 参数
    fn parse_params(params: &str) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        for part in params.split(';') {
            if let Some((key, value)) = part.split_once('=') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        map
    }

    /// 解析 OSC 缓冲区
    fn parse_osc(&self) -> Option<OscSequence> {
        if self.osc_buffer.is_empty() {
            return None;
        }

        let content = String::from_utf8_lossy(&self.osc_buffer);
        let parts: Vec<&str> = content.splitn(2, ';').collect();

        if parts.is_empty() {
            return None;
        }

        let cmd = parts[0];

        match cmd {
            "133" => {
                // FinalTerm / VSCode shell integration
                // 格式: OSC 133 ; <subcommand> [; <params>] BEL/ST
                // 扩展: 支持 aid=<id> 和 json=<escaped_json> 参数
                if parts.len() < 2 {
                    return None;
                }

                let subcommand = parts[1];
                let cmd_char = subcommand.chars().next()?;

                // 解析参数 (从 subcommand 的剩余部分)
                let params_str = if subcommand.len() > 1 {
                    &subcommand[1..]
                } else {
                    ""
                };
                let params = Self::parse_params(params_str);

                // 提取 aid
                let aid = params.get("aid").map(|s| s.to_string());

                // 检查第一个字符确定子命令类型
                match cmd_char {
                    'A' => {
                        // 提示符开始,可能包含 JSON 元数据
                        let json = params
                            .get("json")
                            .and_then(|s| {
                                let unescaped = Self::unescape_value(s);
                                serde_json::from_str::<BlockMetadata>(&unescaped)
                                    .map_err(|e| {
                                        tracing::debug!("Failed to parse BlockMetadata: {}", e);
                                        e
                                    })
                                    .ok()
                            });
                        Some(OscSequence::PromptStart { aid, json })
                    }
                    'B' => Some(OscSequence::CommandStart { aid }),
                    'C' => Some(OscSequence::CommandExecuting { aid }),
                    'D' => {
                        // 命令结束,可能包含退出码和 JSON
                        // 格式: D 或 D;0 或 D;0;aid=xxx 或 D;exit_code=0;aid=xxx
                        let exit_code = if let Some(code_str) = params.get("exit_code") {
                            // 新格式: exit_code=0
                            code_str.parse::<i32>().ok()
                        } else if !params_str.is_empty() && params_str.starts_with(';') {
                            // 旧格式: ;0 或 ;0;aid=xxx
                            let remaining = &params_str[1..]; // 跳过第一个分号
                            if let Some((first, _rest)) = remaining.split_once(';') {
                                // D;0;aid=xxx
                                first.trim().parse::<i32>().ok()
                            } else {
                                // D;0
                                remaining.trim().parse::<i32>().ok()
                            }
                        } else {
                            None
                        };

                        let json = params
                            .get("json")
                            .and_then(|s| {
                                let unescaped = Self::unescape_value(s);
                                serde_json::from_str::<CommandResult>(&unescaped)
                                    .map_err(|e| {
                                        tracing::debug!("Failed to parse CommandResult: {}", e);
                                        e
                                    })
                                    .ok()
                            });

                        tracing::trace!("OSC 133;D parsed with exit_code: {:?}, aid: {:?}", exit_code, aid);
                        Some(OscSequence::CommandFinished { exit_code, aid, json })
                    }
                    _ => {
                        tracing::debug!("Unknown OSC 133 subcommand: {}", subcommand);
                        None
                    }
                }
            }
            "633" => {
                // VSCode extended shell integration
                if parts.len() < 2 {
                    return None;
                }

                let subcommand = parts[1];
                if let Some(rest) = subcommand.strip_prefix("E;") {
                    // Command text
                    Some(OscSequence::CommandText(rest.to_string()))
                } else if let Some(rest) = subcommand.strip_prefix("P;Cwd=") {
                    // Working directory
                    Some(OscSequence::WorkingDirectory(rest.to_string()))
                } else {
                    None
                }
            }
            "7" => {
                // OSC 7 - Working directory
                if parts.len() < 2 {
                    return None;
                }
                Some(OscSequence::Osc7WorkingDirectory(parts[1].to_string()))
            }
            "531" => {
                // OSC 531 - zTerm 自定义 JSON 数据通道
                // 格式: OSC 531 ; <escaped_json> BEL/ST
                if parts.len() < 2 {
                    return None;
                }

                let escaped_json = parts[1];
                let unescaped = Self::unescape_value(escaped_json);

                // 解析 JSON
                match serde_json::from_str::<serde_json::Value>(&unescaped) {
                    Ok(json) => {
                        // 根据 type 字段确定数据类型
                        let data_type = json
                            .get("type")
                            .and_then(|v| v.as_str())
                            .and_then(|s| match s {
                                "command_start" | "command_end" => Some(JsonDataType::CommandMeta),
                                "prompt_start" => Some(JsonDataType::BlockMeta),
                                "directory_changed" => Some(JsonDataType::OutputMeta),
                                _ => Some(JsonDataType::Custom),
                            })
                            .unwrap_or(JsonDataType::Custom);

                        Some(OscSequence::JsonData {
                            data_type,
                            payload: json,
                        })
                    }
                    Err(e) => {
                        tracing::debug!("Failed to parse OSC 531 JSON: {}", e);
                        None
                    }
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== 基础 OSC 133 测试 ==========

    #[test]
    fn test_osc_133_prompt_start() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;A\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        assert!(matches!(
            sequences[0],
            OscSequence::PromptStart { aid: None, json: None }
        ));
    }

    #[test]
    fn test_osc_133_command_start() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;B\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        assert!(matches!(sequences[0], OscSequence::CommandStart { aid: None }));
    }

    #[test]
    fn test_osc_133_command_executing() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;C\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        assert!(matches!(sequences[0], OscSequence::CommandExecuting { aid: None }));
    }

    #[test]
    fn test_osc_133_command_finished_no_exit_code() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;D\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandFinished { exit_code, aid, json } = &sequences[0] {
            assert_eq!(*exit_code, None);
            assert_eq!(*aid, None);
            assert!(json.is_none());
        } else {
            panic!("Expected CommandFinished, got {:?}", sequences[0]);
        }
    }

    #[test]
    fn test_osc_133_command_finished_with_exit_code() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;D;0\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandFinished { exit_code, aid, json } = &sequences[0] {
            assert_eq!(*exit_code, Some(0));
            assert_eq!(*aid, None);
            assert!(json.is_none());
        } else {
            panic!("Expected CommandFinished, got {:?}", sequences[0]);
        }
    }

    #[test]
    fn test_osc_133_command_finished_non_zero_exit() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;D;127\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandFinished { exit_code, aid, json } = &sequences[0] {
            assert_eq!(*exit_code, Some(127));
            assert_eq!(*aid, None);
            assert!(json.is_none());
        } else {
            panic!("Expected CommandFinished, got {:?}", sequences[0]);
        }
    }

    #[test]
    fn test_osc_133_command_finished_negative_exit() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;D;-1\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandFinished { exit_code, aid, json } = &sequences[0] {
            assert_eq!(*exit_code, Some(-1));
            assert_eq!(*aid, None);
            assert!(json.is_none());
        } else {
            panic!("Expected CommandFinished, got {:?}", sequences[0]);
        }
    }

    // ========== OSC 633 测试 ==========

    #[test]
    fn test_osc_633_working_directory() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]633;P;Cwd=/home/user\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::WorkingDirectory(cwd) = &sequences[0] {
            assert_eq!(cwd, "/home/user");
        } else {
            panic!("Expected WorkingDirectory, got {:?}", sequences[0]);
        }
    }

    #[test]
    fn test_osc_633_command_text() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]633;E;ls -la\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandText(text) = &sequences[0] {
            assert_eq!(text, "ls -la");
        } else {
            panic!("Expected CommandText, got {:?}", sequences[0]);
        }
    }

    #[test]
    fn test_osc_7_working_directory() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]7;file:///home/user/project\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::Osc7WorkingDirectory(cwd) = &sequences[0] {
            assert_eq!(cwd, "file:///home/user/project");
        } else {
            panic!("Expected Osc7WorkingDirectory, got {:?}", sequences[0]);
        }
    }

    // ========== OSC 终止符测试 ==========

    #[test]
    fn test_osc_with_st_terminator() {
        // ST (String Terminator) = ESC \
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;A\x1b\\";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        assert!(matches!(
            sequences[0],
            OscSequence::PromptStart { aid: None, json: None }
        ));
    }

    #[test]
    fn test_osc_with_bel_terminator() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;B\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        assert!(matches!(sequences[0], OscSequence::CommandStart { aid: None }));
    }

    // ========== 缓冲区限制测试 (参考 WezTerm) ==========

    #[test]
    fn test_osc_buffer_limit() {
        // 创建一个只有 10 字节限制的扫描器
        let mut scanner = OscScanner::with_max_len(10);

        // 构造一个超过限制的 OSC 序列
        // OSC 0 用于设置窗口标题,可以包含任意文本
        let mut data = Vec::new();
        data.extend_from_slice(b"\x1b]0;");
        // 添加大量标题文本,超过 10 字节限制
        // 前面已经有 "0;" (2字节),再加 20 个 'A' 肯定超过 10 字节
        for _ in 0..20 {
            data.push(b'A');
        }
        data.push(0x07); // BEL

        scanner.scan(&data);

        // 检查缓冲区大小和 buffer_full 标志
        println!("Buffer len: {}, buffer_full: {}", scanner.osc_buffer.len(), scanner.buffer_full);

        // buffer_full 标志应该被设置
        assert!(scanner.buffer_full, "Buffer should be marked as full after exceeding limit (len={}, max={})",
                scanner.osc_buffer.len(), scanner.max_osc_len);
    }

    #[test]
    fn test_osc_buffer_reset_on_new_sequence() {
        let mut scanner = OscScanner::with_max_len(20);

        // 第一个超长序列
        let mut data1 = Vec::new();
        data1.extend_from_slice(b"\x1b]0;");
        for _ in 0..50 {
            data1.push(b'X');
        }
        data1.push(0x07);

        scanner.scan(&data1);
        assert!(scanner.buffer_full, "First sequence should trigger buffer_full");

        // 第二个正常序列应该重置 buffer_full
        let data2 = b"\x1b]133;B\x07";
        let sequences = scanner.scan(data2);

        assert_eq!(sequences.len(), 1);
        assert!(!scanner.buffer_full, "buffer_full should be reset for new sequence");
        assert!(matches!(sequences[0], OscSequence::CommandStart { aid: None }));
    }

    // ========== 多序列测试 ==========

    #[test]
    fn test_multiple_osc_sequences() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;A\x07\x1b]133;B\x07\x1b]133;C\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 3);
        assert!(matches!(
            sequences[0],
            OscSequence::PromptStart { aid: None, json: None }
        ));
        assert!(matches!(sequences[1], OscSequence::CommandStart { aid: None }));
        assert!(matches!(sequences[2], OscSequence::CommandExecuting { aid: None }));
    }

    // ========== 不完整序列测试 ==========

    #[test]
    fn test_incomplete_osc_sequence() {
        let mut scanner = OscScanner::new();
        // 没有终止符的序列
        let data = b"\x1b]133;A";
        let sequences = scanner.scan(data);

        // 应该不返回任何序列
        assert_eq!(sequences.len(), 0);
        // 但状态应该保持在 OscCollect
        assert_eq!(scanner.state, State::OscCollect);
    }

    #[test]
    fn test_incremental_osc_parsing() {
        let mut scanner = OscScanner::new();

        // 分多次输入
        let part1 = b"\x1b]13";
        let sequences1 = scanner.scan(part1);
        assert_eq!(sequences1.len(), 0);

        let part2 = b"3;D;";
        let sequences2 = scanner.scan(part2);
        assert_eq!(sequences2.len(), 0);

        let part3 = b"42\x07";
        let sequences3 = scanner.scan(part3);
        assert_eq!(sequences3.len(), 1);

        if let OscSequence::CommandFinished { exit_code, aid, json } = &sequences3[0] {
            assert_eq!(*exit_code, Some(42));
            assert_eq!(*aid, None);
            assert!(json.is_none());
        } else {
            panic!("Expected CommandFinished");
        }
    }

    // ========== 无效序列测试 ==========

    #[test]
    fn test_invalid_osc_command() {
        let mut scanner = OscScanner::new();
        // 不支持的 OSC 命令
        let data = b"\x1b]999;invalid\x07";
        let sequences = scanner.scan(data);

        // 应该静默忽略
        assert_eq!(sequences.len(), 0);
    }

    #[test]
    fn test_malformed_osc_133() {
        let mut scanner = OscScanner::new();
        // 缺少子命令
        let data = b"\x1b]133\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 0);
    }

    #[test]
    fn test_osc_with_utf8_content() {
        let mut scanner = OscScanner::new();
        let data = "ESC]633;E;echo 你好世界\x07"
            .replace("ESC", "\x1b")
            .into_bytes();
        let sequences = scanner.scan(&data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandText(text) = &sequences[0] {
            assert_eq!(text, "echo 你好世界");
        } else {
            panic!("Expected CommandText");
        }
    }

    // ========== 边界情况测试 ==========

    #[test]
    fn test_empty_input() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"");
        assert_eq!(sequences.len(), 0);
    }

    #[test]
    fn test_just_escape() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b");
        assert_eq!(sequences.len(), 0);
        assert_eq!(scanner.state, State::Escape);
    }

    #[test]
    fn test_escape_without_osc() {
        let mut scanner = OscScanner::new();
        let data = b"\x1bXsomething";
        let sequences = scanner.scan(data);
        assert_eq!(sequences.len(), 0);
        assert_eq!(scanner.state, State::Ground);
    }

    // ========== OSC 133 扩展测试 (aid 参数) ==========

    #[test]
    fn test_osc_133_prompt_start_with_aid() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;A;aid=cmd_001\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::PromptStart { aid, json } = &sequences[0] {
            assert_eq!(aid.as_deref(), Some("cmd_001"));
            assert!(json.is_none());
        } else {
            panic!("Expected PromptStart, got {:?}", sequences[0]);
        }
    }

    #[test]
    fn test_osc_133_command_finished_with_aid() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;D;0;aid=cmd_001\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandFinished { exit_code, aid, json } = &sequences[0] {
            assert_eq!(*exit_code, Some(0));
            assert_eq!(aid.as_deref(), Some("cmd_001"));
            assert!(json.is_none());
        } else {
            panic!("Expected CommandFinished, got {:?}", sequences[0]);
        }
    }

    // ========== OSC 531 JSON 数据测试 ==========

    #[test]
    fn test_osc_531_simple_json() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]531;{\"type\":\"custom\",\"value\":42}\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::JsonData { data_type, payload } = &sequences[0] {
            assert_eq!(*data_type, JsonDataType::Custom);
            assert_eq!(payload.get("type").and_then(|v| v.as_str()), Some("custom"));
            assert_eq!(payload.get("value").and_then(|v| v.as_i64()), Some(42));
        } else {
            panic!("Expected JsonData, got {:?}", sequences[0]);
        }
    }

    #[test]
    fn test_osc_531_command_start_metadata() {
        let mut scanner = OscScanner::new();
        let json = r#"{"type":"command_start","command":"ls -la","cwd":"/home/user"}"#;
        let data = format!("\x1b]531;{}\x07", json);
        let sequences = scanner.scan(data.as_bytes());

        assert_eq!(sequences.len(), 1);
        if let OscSequence::JsonData { data_type, payload } = &sequences[0] {
            assert_eq!(*data_type, JsonDataType::CommandMeta);
            assert_eq!(payload.get("type").and_then(|v| v.as_str()), Some("command_start"));
            assert_eq!(payload.get("command").and_then(|v| v.as_str()), Some("ls -la"));
        } else {
            panic!("Expected JsonData, got {:?}", sequences[0]);
        }
    }

    #[test]
    fn test_osc_531_escaped_json() {
        let mut scanner = OscScanner::new();
        // PowerShell 流程:
        // 1. ConvertTo-Json: {"text":"line1\nline2"} (JSON中\n是两个字符: \ 和 n)
        // 2. __ZTerm-Escape-Value: 转义反斜杠 \ (0x5c) → \x5c
        // 3. 结果: {"text":"line1\x5cnline2"}

        // 构造转义后的 JSON
        let mut escaped_json = Vec::new();
        escaped_json.extend_from_slice(b"{\"type\":\"custom\",\"text\":\"line1");
        // 添加转义的反斜杠: \x5c (表示JSON中的\字符)
        escaped_json.push(b'\\');
        escaped_json.push(b'x');
        escaped_json.push(b'5');
        escaped_json.push(b'c');
        // 添加 n (JSON中\n的第二个字符)
        escaped_json.push(b'n');
        escaped_json.extend_from_slice(b"line2\"}");

        let mut data = Vec::new();
        data.push(0x1b); // ESC
        data.extend_from_slice(b"]531;");
        data.extend_from_slice(&escaped_json);
        data.push(0x07); // BEL

        let sequences = scanner.scan(&data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::JsonData { payload, .. } = &sequences[0] {
            // 反转义得到 {"text":"line1\nline2"}, JSON解析后文本包含换行符
            assert_eq!(payload.get("text").and_then(|v| v.as_str()), Some("line1\nline2"));
        } else {
            panic!("Expected JsonData, got {:?}", sequences[0]);
        }
    }

    // ========== 反转义功能测试 ==========

    #[test]
    fn test_unescape_value() {
        assert_eq!(OscScanner::unescape_value("hello"), "hello");
        assert_eq!(OscScanner::unescape_value("hello\\x0aworld"), "hello\nworld");
        assert_eq!(OscScanner::unescape_value("hello\\x20world"), "hello world");
        assert_eq!(OscScanner::unescape_value("a\\\\b"), "a\\b");
        assert_eq!(OscScanner::unescape_value("a\\nb"), "a\nb");
    }

    #[test]
    fn test_parse_params() {
        let params = OscScanner::parse_params("aid=cmd_001;json={...}");
        assert_eq!(params.get("aid"), Some(&"cmd_001".to_string()));
        assert_eq!(params.get("json"), Some(&"{...}".to_string()));

        let params = OscScanner::parse_params("0;aid=test");
        assert_eq!(params.get("aid"), Some(&"test".to_string()));
    }
}
