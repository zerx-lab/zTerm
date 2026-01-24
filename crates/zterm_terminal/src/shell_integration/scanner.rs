//! High-performance OSC sequence scanner for PTY output
//!
//! This module provides a state-machine based scanner that efficiently
//! identifies OSC 133 and OSC 633 sequences in the PTY output stream
//! before passing data to the terminal emulator.

/// OSC sequence types we care about
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OscSequence {
    /// OSC 133;A - Prompt start
    PromptStart,
    /// OSC 133;B - Command start
    CommandStart,
    /// OSC 133;C - Command executing
    CommandExecuting,
    /// OSC 133;D;exit_code - Command finished
    CommandFinished { exit_code: i32 },
    /// OSC 633;E;command - Command text (VS Code style)
    CommandText { command: String },
    /// OSC 633;P;Cwd=path - Working directory property
    WorkingDirectory { path: String },
    /// OSC 7;file://host/path - Working directory (standard)
    Osc7WorkingDirectory { path: String },
}

/// Scanner state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Normal state, looking for ESC
    Ground,
    /// Saw ESC (\x1b)
    Escape,
    /// Saw ESC ]  (OSC start)
    OscStart,
    /// Collecting OSC content
    OscCollect,
    /// Saw ESC in OSC (possible ST)
    OscEscape,
}

/// High-performance OSC scanner using state machine
///
/// Scans PTY output for OSC 133/633 sequences with O(n) complexity.
/// Does not allocate for non-OSC data.
#[derive(Debug)]
pub struct OscScanner {
    state: State,
    /// Buffer for collecting OSC content
    osc_buffer: Vec<u8>,
    /// Maximum OSC buffer size (prevent memory issues)
    max_osc_len: usize,
}

impl Default for OscScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl OscScanner {
    /// Create a new scanner with default settings
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            osc_buffer: Vec::with_capacity(256),
            max_osc_len: 4096,
        }
    }

    /// Create a scanner with custom max OSC length
    pub fn with_max_len(max_osc_len: usize) -> Self {
        Self {
            state: State::Ground,
            osc_buffer: Vec::with_capacity(256),
            max_osc_len,
        }
    }

    /// Scan input data and extract OSC sequences
    ///
    /// Returns a list of found OSC sequences. The input data should still
    /// be passed to the terminal emulator as-is (we don't strip sequences).
    ///
    /// This is O(n) single-pass scanning.
    pub fn scan(&mut self, data: &[u8]) -> Vec<OscSequence> {
        let mut sequences = Vec::new();

        for &byte in data {
            match self.state {
                State::Ground => {
                    if byte == 0x1b {
                        self.state = State::Escape;
                    }
                }
                State::Escape => {
                    if byte == b']' {
                        self.state = State::OscStart;
                        self.osc_buffer.clear();
                    } else if byte == 0x1b {
                        // Another ESC, stay in Escape state
                        // This handles ESC ESC ] correctly
                    } else {
                        self.state = State::Ground;
                    }
                }
                State::OscStart => {
                    // First byte of OSC content
                    if byte == 0x07 {
                        // BEL terminator with empty content
                        self.state = State::Ground;
                    } else if byte == 0x1b {
                        self.state = State::OscEscape;
                    } else {
                        self.osc_buffer.push(byte);
                        self.state = State::OscCollect;
                    }
                }
                State::OscCollect => {
                    if byte == 0x07 {
                        // BEL terminator
                        if let Some(seq) = self.parse_osc() {
                            sequences.push(seq);
                        }
                        self.osc_buffer.clear();
                        self.state = State::Ground;
                    } else if byte == 0x1b {
                        self.state = State::OscEscape;
                    } else if self.osc_buffer.len() < self.max_osc_len {
                        self.osc_buffer.push(byte);
                    } else {
                        // OSC too long, abort
                        self.osc_buffer.clear();
                        self.state = State::Ground;
                    }
                }
                State::OscEscape => {
                    if byte == b'\\' {
                        // ST terminator (ESC \)
                        if let Some(seq) = self.parse_osc() {
                            sequences.push(seq);
                        }
                        self.osc_buffer.clear();
                        self.state = State::Ground;
                    } else if byte == b']' {
                        // New OSC starting
                        self.osc_buffer.clear();
                        self.state = State::OscStart;
                    } else {
                        // Invalid, back to ground
                        self.osc_buffer.clear();
                        self.state = State::Ground;
                    }
                }
            }
        }

        sequences
    }

    /// Parse the collected OSC buffer
    fn parse_osc(&self) -> Option<OscSequence> {
        let content = std::str::from_utf8(&self.osc_buffer).ok()?;

        // OSC 133 - FinalTerm shell integration
        if let Some(rest) = content.strip_prefix("133;") {
            return self.parse_osc_133(rest);
        }

        // OSC 633 - VS Code shell integration
        if let Some(rest) = content.strip_prefix("633;") {
            return self.parse_osc_633(rest);
        }

        // OSC 7 - Working directory
        if let Some(rest) = content.strip_prefix("7;") {
            return self.parse_osc_7(rest);
        }

        None
    }

    /// Parse OSC 133 sequence
    fn parse_osc_133(&self, data: &str) -> Option<OscSequence> {
        let mut parts = data.splitn(2, ';');
        let cmd = parts.next()?;
        let params = parts.next().unwrap_or("");

        match cmd {
            "A" => Some(OscSequence::PromptStart),
            "B" => Some(OscSequence::CommandStart),
            "C" => Some(OscSequence::CommandExecuting),
            "D" => {
                let exit_code = self.parse_exit_code(params);
                Some(OscSequence::CommandFinished { exit_code })
            }
            _ => None,
        }
    }

    /// Parse OSC 633 sequence
    fn parse_osc_633(&self, data: &str) -> Option<OscSequence> {
        let mut parts = data.splitn(2, ';');
        let cmd = parts.next()?;
        let params = parts.next().unwrap_or("");

        match cmd {
            "A" => Some(OscSequence::PromptStart),
            "B" => Some(OscSequence::CommandStart),
            "C" => Some(OscSequence::CommandExecuting),
            "D" => {
                let exit_code = self.parse_exit_code(params);
                Some(OscSequence::CommandFinished { exit_code })
            }
            "E" => {
                let command = Self::decode_percent(params);
                Some(OscSequence::CommandText { command })
            }
            "P" => params
                .strip_prefix("Cwd=")
                .map(|path| OscSequence::WorkingDirectory {
                    path: path.to_string(),
                }),
            _ => None,
        }
    }

    /// Parse OSC 7 sequence (working directory)
    fn parse_osc_7(&self, data: &str) -> Option<OscSequence> {
        // Format: file://host/path or just path
        let path = if let Some(rest) = data.strip_prefix("file://") {
            // Skip host part
            if let Some(slash_idx) = rest.find('/') {
                &rest[slash_idx..]
            } else {
                rest
            }
        } else {
            data
        };

        Some(OscSequence::Osc7WorkingDirectory {
            path: Self::decode_percent(path),
        })
    }

    /// Parse exit code from OSC 133 D parameters
    fn parse_exit_code(&self, params: &str) -> i32 {
        if params.is_empty() {
            return 0;
        }

        // Try direct number first
        if let Ok(code) = params.trim().parse::<i32>() {
            return code;
        }

        // Try to find exit code in key=value format (e.g., "err=1")
        for part in params.split(';') {
            if let Some(value) = part.strip_prefix("err=") {
                if let Ok(code) = value.parse::<i32>() {
                    return code;
                }
            }
        }

        0
    }

    /// Decode percent-encoded string
    fn decode_percent(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                result.push('%');
                result.push_str(&hex);
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Reset scanner state (e.g., after terminal reset)
    pub fn reset(&mut self) {
        self.state = State::Ground;
        self.osc_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_osc_bel(content: &str) -> Vec<u8> {
        let mut data = vec![0x1b, b']'];
        data.extend_from_slice(content.as_bytes());
        data.push(0x07);
        data
    }

    fn make_osc_st(content: &str) -> Vec<u8> {
        let mut data = vec![0x1b, b']'];
        data.extend_from_slice(content.as_bytes());
        data.extend_from_slice(&[0x1b, b'\\']);
        data
    }

    // ===== Basic Parsing Tests =====

    #[test]
    fn test_scan_osc_133_a() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("133;A");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(seqs[0], OscSequence::PromptStart);
    }

    #[test]
    fn test_scan_osc_133_b() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("133;B");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(seqs[0], OscSequence::CommandStart);
    }

    #[test]
    fn test_scan_osc_133_c() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("133;C");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(seqs[0], OscSequence::CommandExecuting);
    }

    #[test]
    fn test_scan_osc_133_d() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("133;D;0");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(seqs[0], OscSequence::CommandFinished { exit_code: 0 });
    }

    #[test]
    fn test_scan_osc_133_d_failure() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("133;D;127");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs[0], OscSequence::CommandFinished { exit_code: 127 });
    }

    #[test]
    fn test_scan_osc_133_d_no_code() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("133;D");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs[0], OscSequence::CommandFinished { exit_code: 0 });
    }

    // ===== OSC 633 Tests =====

    #[test]
    fn test_scan_osc_633_e() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("633;E;ls%20-la");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(
            seqs[0],
            OscSequence::CommandText {
                command: "ls -la".to_string()
            }
        );
    }

    #[test]
    fn test_scan_osc_633_p_cwd() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("633;P;Cwd=/home/user");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(
            seqs[0],
            OscSequence::WorkingDirectory {
                path: "/home/user".to_string()
            }
        );
    }

    // ===== OSC 7 Tests =====

    #[test]
    fn test_scan_osc_7() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("7;file://localhost/home/user");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(
            seqs[0],
            OscSequence::Osc7WorkingDirectory {
                path: "/home/user".to_string()
            }
        );
    }

    #[test]
    fn test_scan_osc_7_percent_encoded() {
        let mut scanner = OscScanner::new();
        let data = make_osc_bel("7;file://host/home/user/my%20folder");
        let seqs = scanner.scan(&data);

        assert_eq!(
            seqs[0],
            OscSequence::Osc7WorkingDirectory {
                path: "/home/user/my folder".to_string()
            }
        );
    }

    // ===== Terminator Tests =====

    #[test]
    fn test_scan_st_terminator() {
        let mut scanner = OscScanner::new();
        let data = make_osc_st("133;A");
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(seqs[0], OscSequence::PromptStart);
    }

    // ===== Mixed Content Tests =====

    #[test]
    fn test_scan_mixed_content() {
        let mut scanner = OscScanner::new();
        let mut data = b"hello world\r\n".to_vec();
        data.extend(make_osc_bel("133;A"));
        data.extend(b"more text");
        data.extend(make_osc_bel("133;B"));
        data.extend(b"\r\n");

        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 2);
        assert_eq!(seqs[0], OscSequence::PromptStart);
        assert_eq!(seqs[1], OscSequence::CommandStart);
    }

    #[test]
    fn test_scan_full_lifecycle() {
        let mut scanner = OscScanner::new();

        // Prompt
        let seqs = scanner.scan(&make_osc_bel("133;A"));
        assert_eq!(seqs[0], OscSequence::PromptStart);

        // Command input
        let seqs = scanner.scan(&make_osc_bel("133;B"));
        assert_eq!(seqs[0], OscSequence::CommandStart);

        // Command text
        let seqs = scanner.scan(&make_osc_bel("633;E;echo%20hello"));
        assert_eq!(
            seqs[0],
            OscSequence::CommandText {
                command: "echo hello".to_string()
            }
        );

        // Executing
        let seqs = scanner.scan(&make_osc_bel("133;C"));
        assert_eq!(seqs[0], OscSequence::CommandExecuting);

        // Finished
        let seqs = scanner.scan(&make_osc_bel("133;D;0"));
        assert_eq!(seqs[0], OscSequence::CommandFinished { exit_code: 0 });
    }

    #[test]
    fn test_scan_no_osc() {
        let mut scanner = OscScanner::new();
        let data = b"regular terminal output\r\n";
        let seqs = scanner.scan(data);

        assert!(seqs.is_empty());
    }

    #[test]
    fn test_scan_partial_osc_across_chunks() {
        let mut scanner = OscScanner::new();

        // First chunk: ESC ]
        let seqs1 = scanner.scan(&[0x1b, b']']);
        assert!(seqs1.is_empty());

        // Second chunk: 133;A BEL
        let seqs2 = scanner.scan(b"133;A\x07");
        assert_eq!(seqs2.len(), 1);
        assert_eq!(seqs2[0], OscSequence::PromptStart);
    }

    #[test]
    fn test_scan_ignored_osc() {
        let mut scanner = OscScanner::new();
        // OSC 0 (window title) should be ignored
        let data = make_osc_bel("0;My Terminal Title");
        let seqs = scanner.scan(&data);

        assert!(seqs.is_empty());
    }

    #[test]
    fn test_scan_reset() {
        let mut scanner = OscScanner::new();

        // Start OSC but don't finish
        scanner.scan(&[0x1b, b']', b'1', b'3', b'3']);

        // Reset
        scanner.reset();

        // Now scan a complete sequence
        let seqs = scanner.scan(&make_osc_bel("133;A"));
        assert_eq!(seqs.len(), 1);
    }

    #[test]
    fn test_scan_osc_too_long() {
        let mut scanner = OscScanner::with_max_len(10);

        let mut data = vec![0x1b, b']'];
        data.extend(std::iter::repeat_n(b'x', 100));
        data.push(0x07);

        let seqs = scanner.scan(&data);
        assert!(seqs.is_empty());
    }

    // ===== Performance/Edge Cases =====

    #[test]
    fn test_scan_many_sequences() {
        let mut scanner = OscScanner::new();
        let mut data = Vec::new();

        for _ in 0..100 {
            data.extend(make_osc_bel("133;A"));
            data.extend(b"output\r\n");
            data.extend(make_osc_bel("133;D;0"));
        }

        let seqs = scanner.scan(&data);
        assert_eq!(seqs.len(), 200);
    }

    #[test]
    fn test_scan_consecutive_esc() {
        let mut scanner = OscScanner::new();
        // ESC ESC ] 133;A BEL - consecutive ESCs should still work
        // First ESC -> Escape, Second ESC -> stay in Escape, ] -> OscStart
        let data = [0x1b, 0x1b, b']', b'1', b'3', b'3', b';', b'A', 0x07];
        let seqs = scanner.scan(&data);

        assert_eq!(seqs.len(), 1);
        assert_eq!(seqs[0], OscSequence::PromptStart);
    }

    #[test]
    fn test_default() {
        let scanner = OscScanner::default();
        assert_eq!(scanner.max_osc_len, 4096);
    }
}
