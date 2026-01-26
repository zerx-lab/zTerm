//! OSC 序列扫描器
//!
//! 在 VTE 解析前拦截 OSC 133/633 序列，用于 shell integration

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
    /// OSC 133;A - 提示符开始
    PromptStart,
    /// OSC 133;B - 命令开始
    CommandStart,
    /// OSC 133;C - 命令执行中
    CommandExecuting,
    /// OSC 133;D - 命令结束
    CommandFinished { exit_code: Option<i32> },
    /// OSC 633;E - 命令文本
    CommandText(String),
    /// OSC 633;P;Cwd= - 工作目录
    WorkingDirectory(String),
    /// OSC 7 - 工作目录（另一种格式）
    Osc7WorkingDirectory(String),
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
                // 格式: OSC 133 ; <subcommand> [; <args>] BEL/ST
                if parts.len() < 2 {
                    return None;
                }

                let subcommand = parts[1];

                // 检查第一个字符确定子命令类型
                match subcommand.chars().next()? {
                    'A' => Some(OscSequence::PromptStart),
                    'B' => Some(OscSequence::CommandStart),
                    'C' => Some(OscSequence::CommandExecuting),
                    'D' => {
                        // 命令结束,可能包含退出码
                        // 格式: D 或 D;<exit_code>
                        // 正确的解析方式: 分割分号
                        let exit_code = if subcommand.len() > 1 {
                            // 跳过 'D',查找分号
                            subcommand[1..]
                                .split_once(';')
                                .and_then(|(_, code)| code.parse::<i32>().ok())
                        } else {
                            None
                        };

                        tracing::trace!("OSC 133;D parsed with exit_code: {:?}", exit_code);
                        Some(OscSequence::CommandFinished { exit_code })
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
        assert!(matches!(sequences[0], OscSequence::PromptStart));
    }

    #[test]
    fn test_osc_133_command_start() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;B\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        assert!(matches!(sequences[0], OscSequence::CommandStart));
    }

    #[test]
    fn test_osc_133_command_executing() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;C\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        assert!(matches!(sequences[0], OscSequence::CommandExecuting));
    }

    #[test]
    fn test_osc_133_command_finished_no_exit_code() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;D\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        if let OscSequence::CommandFinished { exit_code } = &sequences[0] {
            assert_eq!(*exit_code, None);
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
        if let OscSequence::CommandFinished { exit_code } = &sequences[0] {
            assert_eq!(*exit_code, Some(0));
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
        if let OscSequence::CommandFinished { exit_code } = &sequences[0] {
            assert_eq!(*exit_code, Some(127));
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
        if let OscSequence::CommandFinished { exit_code } = &sequences[0] {
            assert_eq!(*exit_code, Some(-1));
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
        assert!(matches!(sequences[0], OscSequence::PromptStart));
    }

    #[test]
    fn test_osc_with_bel_terminator() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;B\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 1);
        assert!(matches!(sequences[0], OscSequence::CommandStart));
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
        assert!(matches!(sequences[0], OscSequence::CommandStart));
    }

    // ========== 多序列测试 ==========

    #[test]
    fn test_multiple_osc_sequences() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;A\x07\x1b]133;B\x07\x1b]133;C\x07";
        let sequences = scanner.scan(data);

        assert_eq!(sequences.len(), 3);
        assert!(matches!(sequences[0], OscSequence::PromptStart));
        assert!(matches!(sequences[1], OscSequence::CommandStart));
        assert!(matches!(sequences[2], OscSequence::CommandExecuting));
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

        if let OscSequence::CommandFinished { exit_code } = &sequences3[0] {
            assert_eq!(*exit_code, Some(42));
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
}
