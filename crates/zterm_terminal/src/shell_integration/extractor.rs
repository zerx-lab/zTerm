//! Text extraction from terminal buffer
//!
//! This module provides utilities for extracting text from specific zones
//! in the terminal buffer.

use super::zone::CommandZone;

/// A trait for terminal buffers that support text extraction
pub trait TextBuffer {
    /// Get the text content of a specific line
    fn line_text(&self, line: usize) -> Option<String>;

    /// Get the total number of lines in the buffer
    fn total_lines(&self) -> usize;
}

/// Extracts text from terminal buffer regions
pub struct TextExtractor;

impl TextExtractor {
    /// Extract text from a range of lines
    pub fn extract_lines<B: TextBuffer>(
        buffer: &B,
        start_line: usize,
        end_line: Option<usize>,
    ) -> Vec<String> {
        let end = end_line.unwrap_or_else(|| buffer.total_lines());
        let mut lines = Vec::new();

        for line in start_line..end.min(buffer.total_lines()) {
            if let Some(text) = buffer.line_text(line) {
                lines.push(text);
            }
        }

        lines
    }

    /// Extract all text from a zone as a single string
    pub fn extract_zone_text<B: TextBuffer>(buffer: &B, zone: &CommandZone) -> String {
        let lines = Self::extract_lines(buffer, zone.start_line, zone.end_line);
        lines.join("\n")
    }

    /// Extract command output (everything after the command line)
    pub fn extract_output<B: TextBuffer>(buffer: &B, zone: &CommandZone) -> Option<String> {
        // Output starts after the command line (typically start_line + 1)
        let output_start = zone.start_line.saturating_add(1);

        let end = zone.end_line?;
        if output_start >= end {
            return Some(String::new());
        }

        let lines = Self::extract_lines(buffer, output_start, Some(end));
        Some(lines.join("\n"))
    }

    /// Extract the command text from a zone
    ///
    /// If the zone has a captured command (from OSC 633 E), return that.
    /// Otherwise, try to extract it from the command line in the buffer.
    pub fn extract_command<B: TextBuffer>(buffer: &B, zone: &CommandZone) -> Option<String> {
        // Prefer the captured command if available
        if let Some(cmd) = &zone.command {
            return Some(cmd.clone());
        }

        // Try to extract from the first line of the zone
        // This is a heuristic - the prompt line usually contains the command
        buffer.line_text(zone.start_line).map(|line| {
            // Remove common prompt patterns
            Self::strip_prompt(&line)
        })
    }

    /// Strip common prompt patterns from a line
    fn strip_prompt(line: &str) -> String {
        let trimmed = line.trim();

        // Common prompt endings: $, >, #, %
        let prompt_chars = ['$', '>', '#', '%'];

        for (i, c) in trimmed.char_indices() {
            if prompt_chars.contains(&c) {
                // Take everything after the prompt character
                let rest = &trimmed[i + c.len_utf8()..].trim_start();
                if !rest.is_empty() {
                    return rest.to_string();
                }
            }
        }

        // No prompt found, return as-is
        trimmed.to_string()
    }

    /// Get a context summary for AI integration
    pub fn get_context_summary<B: TextBuffer>(
        buffer: &B,
        zone: &CommandZone,
        max_output_lines: usize,
    ) -> ContextSummary {
        let command = Self::extract_command(buffer, zone);
        let working_dir = zone.working_directory.clone();
        let exit_code = zone.state.exit_code();

        let output = if let Some(end_line) = zone.end_line {
            let output_start = zone.start_line.saturating_add(1);
            let output_lines = Self::extract_lines(buffer, output_start, Some(end_line));
            let total_line_count = output_lines.len();

            // Truncate if necessary
            let truncated = total_line_count > max_output_lines;
            let lines: Vec<_> = if truncated {
                // Keep first and last portions
                let half = max_output_lines / 2;
                let mut result: Vec<String> = output_lines.iter().take(half).cloned().collect();
                result.push(format!(
                    "... ({} lines omitted) ...",
                    total_line_count - max_output_lines
                ));
                result.extend(
                    output_lines
                        .iter()
                        .skip(total_line_count.saturating_sub(half))
                        .cloned(),
                );
                result
            } else {
                output_lines
            };

            Some(OutputSummary {
                text: lines.join("\n"),
                line_count: total_line_count,
                truncated,
            })
        } else {
            None
        };

        ContextSummary {
            command,
            working_dir,
            exit_code,
            output,
        }
    }
}

/// Summary of command output
#[derive(Debug, Clone)]
pub struct OutputSummary {
    /// The output text (possibly truncated)
    pub text: String,
    /// Total number of output lines
    pub line_count: usize,
    /// Whether the output was truncated
    pub truncated: bool,
}

/// Context summary for AI integration
#[derive(Debug, Clone)]
pub struct ContextSummary {
    /// The command that was executed
    pub command: Option<String>,
    /// Working directory when command ran
    pub working_dir: Option<String>,
    /// Exit code (if finished)
    pub exit_code: Option<i32>,
    /// Output summary (if available)
    pub output: Option<OutputSummary>,
}

impl ContextSummary {
    /// Format as a string suitable for AI context
    pub fn to_ai_context(&self) -> String {
        let mut parts = Vec::new();

        if let Some(cmd) = &self.command {
            parts.push(format!("Command: {}", cmd));
        }

        if let Some(dir) = &self.working_dir {
            parts.push(format!("Working Directory: {}", dir));
        }

        if let Some(code) = self.exit_code {
            parts.push(format!("Exit Code: {}", code));
        }

        if let Some(output) = &self.output {
            parts.push(format!("Output ({} lines):", output.line_count));
            parts.push(output.text.clone());
        }

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell_integration::zone::{CommandState, ZoneId};

    /// Mock buffer for testing
    struct MockBuffer {
        lines: Vec<String>,
    }

    impl MockBuffer {
        fn new(lines: Vec<&str>) -> Self {
            Self {
                lines: lines.into_iter().map(String::from).collect(),
            }
        }
    }

    impl TextBuffer for MockBuffer {
        fn line_text(&self, line: usize) -> Option<String> {
            self.lines.get(line).cloned()
        }

        fn total_lines(&self) -> usize {
            self.lines.len()
        }
    }

    fn make_zone(start: usize, end: Option<usize>, command: Option<&str>) -> CommandZone {
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, start);
        zone.end_line = end;
        zone.command = command.map(String::from);
        zone
    }

    // ===== extract_lines Tests =====

    #[test]
    fn test_extract_lines_basic() {
        let buffer = MockBuffer::new(vec!["line 0", "line 1", "line 2", "line 3"]);
        let lines = TextExtractor::extract_lines(&buffer, 1, Some(3));

        assert_eq!(lines, vec!["line 1", "line 2"]);
    }

    #[test]
    fn test_extract_lines_open_ended() {
        let buffer = MockBuffer::new(vec!["line 0", "line 1", "line 2"]);
        let lines = TextExtractor::extract_lines(&buffer, 0, None);

        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_extract_lines_beyond_buffer() {
        let buffer = MockBuffer::new(vec!["line 0", "line 1"]);
        let lines = TextExtractor::extract_lines(&buffer, 0, Some(100));

        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_extract_lines_empty_range() {
        let buffer = MockBuffer::new(vec!["line 0", "line 1"]);
        let lines = TextExtractor::extract_lines(&buffer, 5, Some(10));

        assert!(lines.is_empty());
    }

    #[test]
    fn test_extract_lines_single_line() {
        let buffer = MockBuffer::new(vec!["line 0", "line 1", "line 2"]);
        let lines = TextExtractor::extract_lines(&buffer, 1, Some(2));

        assert_eq!(lines, vec!["line 1"]);
    }

    // ===== extract_zone_text Tests =====

    #[test]
    fn test_extract_zone_text_full() {
        let buffer = MockBuffer::new(vec!["$ ls", "file1", "file2", "$ "]);
        let zone = make_zone(0, Some(4), None);

        let text = TextExtractor::extract_zone_text(&buffer, &zone);
        assert_eq!(text, "$ ls\nfile1\nfile2\n$ ");
    }

    #[test]
    fn test_extract_zone_text_open() {
        let buffer = MockBuffer::new(vec!["$ ls", "file1", "file2"]);
        let zone = make_zone(0, None, None);

        let text = TextExtractor::extract_zone_text(&buffer, &zone);
        assert!(text.contains("file1"));
        assert!(text.contains("file2"));
    }

    // ===== extract_output Tests =====

    #[test]
    fn test_extract_output_basic() {
        let buffer = MockBuffer::new(vec!["$ ls", "file1", "file2", "$ "]);
        let zone = make_zone(0, Some(3), None);

        let output = TextExtractor::extract_output(&buffer, &zone);
        assert_eq!(output, Some("file1\nfile2".to_string()));
    }

    #[test]
    fn test_extract_output_no_output() {
        let buffer = MockBuffer::new(vec!["$ echo", "$ "]);
        let zone = make_zone(0, Some(1), None);

        let output = TextExtractor::extract_output(&buffer, &zone);
        assert_eq!(output, Some(String::new()));
    }

    #[test]
    fn test_extract_output_open_zone() {
        let buffer = MockBuffer::new(vec!["$ ls", "file1"]);
        let zone = make_zone(0, None, None);

        let output = TextExtractor::extract_output(&buffer, &zone);
        assert!(output.is_none());
    }

    // ===== extract_command Tests =====

    #[test]
    fn test_extract_command_from_zone() {
        let buffer = MockBuffer::new(vec!["$ ls -la"]);
        let zone = make_zone(0, Some(1), Some("ls -la"));

        let cmd = TextExtractor::extract_command(&buffer, &zone);
        assert_eq!(cmd, Some("ls -la".to_string()));
    }

    #[test]
    fn test_extract_command_from_buffer() {
        let buffer = MockBuffer::new(vec!["$ ls -la", "output"]);
        let zone = make_zone(0, Some(2), None);

        let cmd = TextExtractor::extract_command(&buffer, &zone);
        assert_eq!(cmd, Some("ls -la".to_string()));
    }

    #[test]
    fn test_extract_command_hash_prompt() {
        let buffer = MockBuffer::new(vec!["# rm -rf /tmp/test"]);
        let zone = make_zone(0, Some(1), None);

        let cmd = TextExtractor::extract_command(&buffer, &zone);
        assert_eq!(cmd, Some("rm -rf /tmp/test".to_string()));
    }

    #[test]
    fn test_extract_command_gt_prompt() {
        let buffer = MockBuffer::new(vec!["PS> Get-Process"]);
        let zone = make_zone(0, Some(1), None);

        let cmd = TextExtractor::extract_command(&buffer, &zone);
        assert_eq!(cmd, Some("Get-Process".to_string()));
    }

    // ===== strip_prompt Tests =====

    #[test]
    fn test_strip_prompt_dollar() {
        assert_eq!(TextExtractor::strip_prompt("$ ls"), "ls");
        assert_eq!(TextExtractor::strip_prompt("user@host$ ls"), "ls");
    }

    #[test]
    fn test_strip_prompt_hash() {
        assert_eq!(TextExtractor::strip_prompt("# rm"), "rm");
        assert_eq!(TextExtractor::strip_prompt("root@host# rm"), "rm");
    }

    #[test]
    fn test_strip_prompt_percent() {
        assert_eq!(TextExtractor::strip_prompt("% ls"), "ls");
    }

    #[test]
    fn test_strip_prompt_no_prompt() {
        assert_eq!(TextExtractor::strip_prompt("just text"), "just text");
    }

    // ===== Context Summary Tests =====

    #[test]
    fn test_context_summary_basic() {
        let buffer = MockBuffer::new(vec!["$ ls", "file1", "file2"]);
        let mut zone = make_zone(0, Some(3), Some("ls"));
        zone.working_directory = Some("/home/user".to_string());
        zone.state = CommandState::CommandFinished(0);

        let summary = TextExtractor::get_context_summary(&buffer, &zone, 100);

        assert_eq!(summary.command, Some("ls".to_string()));
        assert_eq!(summary.working_dir, Some("/home/user".to_string()));
        assert_eq!(summary.exit_code, Some(0));
        assert!(summary.output.is_some());
    }

    #[test]
    fn test_context_summary_truncation() {
        let lines: Vec<&str> = (0..100).map(|_| "output line").collect();
        let mut all_lines = vec!["$ cmd"];
        all_lines.extend(lines);
        let buffer = MockBuffer::new(all_lines);

        let zone = make_zone(0, Some(101), Some("cmd"));

        let summary = TextExtractor::get_context_summary(&buffer, &zone, 10);

        let output = summary.output.unwrap();
        assert!(output.truncated);
        assert_eq!(output.line_count, 100);
    }

    #[test]
    fn test_context_summary_no_truncation() {
        let buffer = MockBuffer::new(vec!["$ ls", "file1", "file2"]);
        let zone = make_zone(0, Some(3), Some("ls"));

        let summary = TextExtractor::get_context_summary(&buffer, &zone, 100);

        let output = summary.output.unwrap();
        assert!(!output.truncated);
    }

    #[test]
    fn test_context_summary_to_ai_context() {
        let buffer = MockBuffer::new(vec!["$ ls", "file1"]);
        let mut zone = make_zone(0, Some(2), Some("ls"));
        zone.working_directory = Some("/home".to_string());
        zone.state = CommandState::CommandFinished(0);

        let summary = TextExtractor::get_context_summary(&buffer, &zone, 100);
        let ai_context = summary.to_ai_context();

        assert!(ai_context.contains("Command: ls"));
        assert!(ai_context.contains("Working Directory: /home"));
        assert!(ai_context.contains("Exit Code: 0"));
    }

    #[test]
    fn test_context_summary_running_command() {
        let buffer = MockBuffer::new(vec!["$ long-running"]);
        let zone = make_zone(0, None, Some("long-running"));

        let summary = TextExtractor::get_context_summary(&buffer, &zone, 100);

        assert!(summary.exit_code.is_none());
        assert!(summary.output.is_none());
    }
}
