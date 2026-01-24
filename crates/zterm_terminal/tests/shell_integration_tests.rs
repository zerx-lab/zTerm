//! Comprehensive tests for shell integration functionality
//!
//! These tests verify:
//! - OSC 133/633 sequence scanning
//! - Zone lifecycle management
//! - Shell integration handler
//! - Text extraction
//! - Event emission

use zterm_terminal::shell_integration::{
    CommandState, CommandZone, OscScanner, OscSequence, ShellEvent, ShellIntegrationHandler,
    TextBuffer, TextExtractor, ZoneId, ZoneManager,
};

/// Mock buffer implementing TextBuffer trait for testing
struct MockBuffer {
    lines: Vec<String>,
}

impl MockBuffer {
    fn new(lines: Vec<&str>) -> Self {
        Self {
            lines: lines.into_iter().map(String::from).collect(),
        }
    }

    fn from_strings(lines: Vec<String>) -> Self {
        Self { lines }
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

// ============================================================================
// OscScanner Tests - Comprehensive Coverage
// ============================================================================

mod osc_scanner_tests {
    use super::*;

    #[test]
    fn test_scanner_empty_input() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"");
        assert!(sequences.is_empty());
    }

    #[test]
    fn test_scanner_no_osc_sequences() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"Hello World\r\nThis is normal text\r\n");
        assert!(sequences.is_empty());
    }

    #[test]
    fn test_scanner_osc_133_a() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]133;A\x07");
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0], OscSequence::PromptStart);
    }

    #[test]
    fn test_scanner_osc_133_b() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]133;B\x07");
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0], OscSequence::CommandStart);
    }

    #[test]
    fn test_scanner_osc_133_c() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]133;C\x07");
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0], OscSequence::CommandExecuting);
    }

    #[test]
    fn test_scanner_osc_133_d_success() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]133;D;0\x07");
        assert_eq!(sequences.len(), 1);
        assert!(matches!(
            sequences[0],
            OscSequence::CommandFinished { exit_code: 0 }
        ));
    }

    #[test]
    fn test_scanner_osc_133_d_failure() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]133;D;1\x07");
        assert_eq!(sequences.len(), 1);
        assert!(matches!(
            sequences[0],
            OscSequence::CommandFinished { exit_code: 1 }
        ));
    }

    #[test]
    fn test_scanner_osc_133_d_negative_exit_code() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]133;D;-1\x07");
        assert_eq!(sequences.len(), 1);
        assert!(matches!(
            sequences[0],
            OscSequence::CommandFinished { exit_code: -1 }
        ));
    }

    #[test]
    fn test_scanner_osc_133_d_large_exit_code() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]133;D;255\x07");
        assert_eq!(sequences.len(), 1);
        assert!(matches!(
            sequences[0],
            OscSequence::CommandFinished { exit_code: 255 }
        ));
    }

    #[test]
    fn test_scanner_osc_633_e_simple() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]633;E;ls\x07");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            OscSequence::CommandText { command } => assert_eq!(command, "ls"),
            _ => panic!("Expected CommandText"),
        }
    }

    #[test]
    fn test_scanner_osc_633_e_with_args() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]633;E;ls%20-la\x07");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            OscSequence::CommandText { command } => assert_eq!(command, "ls -la"),
            _ => panic!("Expected CommandText"),
        }
    }

    #[test]
    fn test_scanner_osc_633_e_complex_command() {
        let mut scanner = OscScanner::new();
        // echo "hello world" | grep hello
        let sequences =
            scanner.scan(b"\x1b]633;E;echo%20%22hello%20world%22%20%7C%20grep%20hello\x07");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            OscSequence::CommandText { command } => {
                assert_eq!(command, "echo \"hello world\" | grep hello");
            }
            _ => panic!("Expected CommandText"),
        }
    }

    #[test]
    fn test_scanner_osc_633_p_cwd() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]633;P;Cwd=/home/user\x07");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            OscSequence::WorkingDirectory { path } => assert_eq!(path, "/home/user"),
            _ => panic!("Expected WorkingDirectory"),
        }
    }

    #[test]
    fn test_scanner_osc_7_unix_path() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]7;file://localhost/home/user/project\x07");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            OscSequence::Osc7WorkingDirectory { path } => {
                assert_eq!(path, "/home/user/project");
            }
            _ => panic!("Expected Osc7WorkingDirectory"),
        }
    }

    #[test]
    fn test_scanner_osc_7_windows_path() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]7;file://localhost/C:/Users/test\x07");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            OscSequence::Osc7WorkingDirectory { path } => {
                assert_eq!(path, "/C:/Users/test");
            }
            _ => panic!("Expected Osc7WorkingDirectory"),
        }
    }

    #[test]
    fn test_scanner_osc_7_percent_encoded() {
        let mut scanner = OscScanner::new();
        let sequences = scanner.scan(b"\x1b]7;file://localhost/home/user/my%20project\x07");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            OscSequence::Osc7WorkingDirectory { path } => {
                assert_eq!(path, "/home/user/my project");
            }
            _ => panic!("Expected Osc7WorkingDirectory"),
        }
    }

    #[test]
    fn test_scanner_multiple_sequences() {
        let mut scanner = OscScanner::new();
        let data = b"\x1b]133;A\x07\x1b]133;B\x07\x1b]133;C\x07\x1b]133;D;0\x07";
        let sequences = scanner.scan(data);
        assert_eq!(sequences.len(), 4);
        assert_eq!(sequences[0], OscSequence::PromptStart);
        assert_eq!(sequences[1], OscSequence::CommandStart);
        assert_eq!(sequences[2], OscSequence::CommandExecuting);
        assert!(matches!(
            sequences[3],
            OscSequence::CommandFinished { exit_code: 0 }
        ));
    }

    #[test]
    fn test_scanner_sequences_with_text_between() {
        let mut scanner = OscScanner::new();
        let data = b"user@host:~$ \x1b]133;A\x07prompt> \x1b]133;B\x07ls -la\r\n\x1b]133;C\x07output\r\n\x1b]133;D;0\x07";
        let sequences = scanner.scan(data);
        assert_eq!(sequences.len(), 4);
    }

    #[test]
    fn test_scanner_split_across_chunks() {
        let mut scanner = OscScanner::new();

        // First chunk: partial escape sequence
        let seq1 = scanner.scan(b"hello\x1b");
        assert!(seq1.is_empty());

        // Second chunk: continue sequence
        let seq2 = scanner.scan(b"]133;A\x07world");
        assert_eq!(seq2.len(), 1);
        assert_eq!(seq2[0], OscSequence::PromptStart);
    }

    #[test]
    fn test_scanner_consecutive_escapes() {
        let mut scanner = OscScanner::new();
        // ESC ESC ] 133;A BEL - should handle consecutive ESC
        let sequences = scanner.scan(b"\x1b\x1b]133;A\x07");
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0], OscSequence::PromptStart);
    }

    #[test]
    fn test_scanner_st_terminator() {
        let mut scanner = OscScanner::new();
        // OSC with ST (ESC \) terminator instead of BEL
        let sequences = scanner.scan(b"\x1b]133;A\x1b\\");
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0], OscSequence::PromptStart);
    }

    #[test]
    fn test_scanner_ignores_other_osc() {
        let mut scanner = OscScanner::new();
        // OSC 0 (set title) should be ignored
        let sequences = scanner.scan(b"\x1b]0;My Title\x07");
        assert!(sequences.is_empty());

        // OSC 52 (clipboard) should be ignored
        let sequences = scanner.scan(b"\x1b]52;c;SGVsbG8=\x07");
        assert!(sequences.is_empty());
    }

    #[test]
    fn test_scanner_reset_between_sessions() {
        let mut scanner = OscScanner::new();

        let seq1 = scanner.scan(b"\x1b]133;A\x07");
        assert_eq!(seq1.len(), 1);

        // New session should work independently
        let seq2 = scanner.scan(b"\x1b]133;B\x07");
        assert_eq!(seq2.len(), 1);
        assert_eq!(seq2[0], OscSequence::CommandStart);
    }
}

// ============================================================================
// ZoneManager Tests - Lifecycle and State
// ============================================================================

mod zone_manager_tests {
    use super::*;

    #[test]
    fn test_zone_manager_full_lifecycle() {
        let mut manager = ZoneManager::new();

        // Start prompt at line 0
        let zone_id = manager.start_zone(CommandState::PromptStart, 0);
        assert!(manager.get(zone_id).is_some());

        // Transition through states
        manager.transition_state(CommandState::CommandStart, 0);
        manager.set_command("ls -la".to_string());
        manager.transition_state(CommandState::CommandExecuting, 1);
        manager.finish_zone(5, 0);

        let zone = manager.get(zone_id).unwrap();
        assert!(zone.is_finished());
        assert_eq!(zone.state.exit_code(), Some(0));
        assert_eq!(zone.command.as_deref(), Some("ls -la"));
    }

    #[test]
    fn test_zone_manager_multiple_commands() {
        let mut manager = ZoneManager::new();

        // First command
        let zone1 = manager.start_zone(CommandState::PromptStart, 0);
        manager.transition_state(CommandState::CommandStart, 0);
        manager.transition_state(CommandState::CommandExecuting, 1);
        manager.finish_zone(5, 0);

        // Second command
        let zone2 = manager.start_zone(CommandState::PromptStart, 6);
        manager.transition_state(CommandState::CommandStart, 6);
        manager.transition_state(CommandState::CommandExecuting, 7);
        manager.finish_zone(10, 1);

        assert_eq!(manager.zones().count(), 2);

        // Check zone_at_line
        let z1 = manager.zone_at_line(3);
        assert!(z1.is_some());
        assert_eq!(z1.unwrap().id, zone1);

        let z2 = manager.zone_at_line(8);
        assert!(z2.is_some());
        assert_eq!(z2.unwrap().id, zone2);
    }

    #[test]
    fn test_zone_manager_previous_next_prompt() {
        let mut manager = ZoneManager::new();

        // Create zones at lines 0, 10, 20
        manager.start_zone(CommandState::PromptStart, 0);
        manager.finish_zone(9, 0);

        manager.start_zone(CommandState::PromptStart, 10);
        manager.finish_zone(19, 0);

        manager.start_zone(CommandState::PromptStart, 20);
        manager.finish_zone(29, 0);

        // Test previous_prompt
        let prev = manager.previous_prompt(25);
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().start_line, 10);

        let prev = manager.previous_prompt(15);
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().start_line, 0);

        // Test next_prompt
        let next = manager.next_prompt(5);
        assert!(next.is_some());
        assert_eq!(next.unwrap().start_line, 10);

        let next = manager.next_prompt(15);
        assert!(next.is_some());
        assert_eq!(next.unwrap().start_line, 20);

        let next = manager.next_prompt(25);
        assert!(next.is_none());
    }

    #[test]
    fn test_zone_manager_working_directory() {
        let mut manager = ZoneManager::new();
        manager.start_zone(CommandState::PromptStart, 0);
        manager.set_working_directory("/home/user".to_string());

        let zone = manager.active_zone().unwrap();
        assert_eq!(zone.working_directory.as_deref(), Some("/home/user"));
    }

    #[test]
    fn test_zone_manager_auto_close() {
        let mut manager = ZoneManager::new();

        // Start first zone without finishing
        let z1 = manager.start_zone(CommandState::PromptStart, 0);
        manager.transition_state(CommandState::CommandExecuting, 1);

        // Start second zone - should auto-close first
        let _z2 = manager.start_zone(CommandState::PromptStart, 10);

        let zone1 = manager.get(z1).unwrap();
        assert_eq!(zone1.end_line, Some(10));
    }

    #[test]
    fn test_zone_contains_line() {
        let mut manager = ZoneManager::new();

        let zone_id = manager.start_zone(CommandState::PromptStart, 5);
        manager.finish_zone(15, 0);

        let zone = manager.get(zone_id).unwrap();

        assert!(!zone.contains_line(4));
        assert!(zone.contains_line(5));
        assert!(zone.contains_line(10));
        assert!(zone.contains_line(14)); // end is exclusive
        assert!(!zone.contains_line(15));
    }

    #[test]
    fn test_zone_line_range() {
        let mut manager = ZoneManager::new();

        let zone_id = manager.start_zone(CommandState::PromptStart, 5);
        manager.finish_zone(15, 0);

        let zone = manager.get(zone_id).unwrap();
        assert_eq!(zone.line_range(), (5, Some(15)));
        assert_eq!(zone.line_count(), Some(10));
    }
}

// ============================================================================
// ShellIntegrationHandler Tests
// ============================================================================

mod handler_tests {
    use super::*;

    #[test]
    fn test_handler_osc_parsing() {
        let mut handler = ShellIntegrationHandler::new();

        handler.handle_osc(b"133;A");
        handler.handle_osc(b"133;B");
        handler.handle_osc(b"133;C");
        handler.handle_osc(b"133;D;0");

        let events = handler.take_events();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_handler_command_capture() {
        let mut handler = ShellIntegrationHandler::new();

        handler.set_current_line(0);
        handler.handle_osc(b"133;A");
        handler.handle_osc(b"633;E;echo%20hello");
        handler.handle_osc(b"133;B");
        handler.set_current_line(1);
        handler.handle_osc(b"133;C");
        handler.set_current_line(5);
        handler.handle_osc(b"133;D;0");

        // Verify command was captured - use zones() since zone_at_line may not find it
        let zones: Vec<_> = handler.zone_manager().zones().collect();
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].command.as_deref(), Some("echo hello"));
    }

    #[test]
    fn test_handler_events_emission() {
        let mut handler = ShellIntegrationHandler::new();
        handler.set_current_line(0);

        handler.handle_osc(b"133;A");
        let events = handler.take_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, ShellEvent::PromptStarted { .. })));

        handler.handle_osc(b"133;B");
        let events = handler.take_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, ShellEvent::CommandStarted { .. })));

        handler.handle_osc(b"133;C");
        let events = handler.take_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, ShellEvent::CommandExecuting { .. })));

        handler.handle_osc(b"133;D;0");
        let events = handler.take_events();
        assert!(events.iter().any(
            |e| matches!(e, ShellEvent::CommandFinished { exit_code: 0, .. })
        ));
    }

    #[test]
    fn test_handler_working_directory() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(b"133;A"); // Start zone first

        handler.handle_osc(b"633;P;Cwd=/home/user/project");

        // Verify working directory was set on zone
        let zone = handler.zone_manager().active_zone();
        assert!(zone.is_some());
        assert_eq!(
            zone.unwrap().working_directory.as_deref(),
            Some("/home/user/project")
        );
    }
}

// ============================================================================
// TextExtractor Tests
// ============================================================================

mod extractor_tests {
    use super::*;

    fn create_test_buffer() -> MockBuffer {
        MockBuffer::new(vec![
            "user@host:~$ ",
            "ls -la",
            "total 0",
            "drwxr-xr-x  2 user user  40 Jan  1 00:00 .",
            "drwxr-xr-x 10 user user 200 Jan  1 00:00 ..",
            "user@host:~$ ",
        ])
    }

    #[test]
    fn test_extract_lines_basic() {
        let buffer = create_test_buffer();

        let lines = TextExtractor::extract_lines(&buffer, 0, Some(3));
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "user@host:~$ ");
        assert_eq!(lines[1], "ls -la");
        assert_eq!(lines[2], "total 0");
    }

    #[test]
    fn test_extract_lines_open_ended() {
        let buffer = create_test_buffer();

        let lines = TextExtractor::extract_lines(&buffer, 3, None);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_extract_lines_beyond_buffer() {
        let buffer = create_test_buffer();

        let lines = TextExtractor::extract_lines(&buffer, 0, Some(100));
        assert_eq!(lines.len(), 6);
    }

    #[test]
    fn test_extract_command() {
        let buffer = MockBuffer::new(vec!["$ ls -la"]);

        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        zone.end_line = Some(1);

        let command = TextExtractor::extract_command(&buffer, &zone);
        assert_eq!(command, Some("ls -la".to_string()));
    }

    #[test]
    fn test_extract_command_various_prompts() {
        // Dollar prompt
        let buffer = MockBuffer::new(vec!["$ echo hello"]);
        let zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        assert_eq!(
            TextExtractor::extract_command(&buffer, &zone),
            Some("echo hello".to_string())
        );

        // Hash prompt (root)
        let buffer = MockBuffer::new(vec!["# rm -rf /"]);
        let zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        assert_eq!(
            TextExtractor::extract_command(&buffer, &zone),
            Some("rm -rf /".to_string())
        );

        // Greater-than prompt
        let buffer = MockBuffer::new(vec!["> continued"]);
        let zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        assert_eq!(
            TextExtractor::extract_command(&buffer, &zone),
            Some("continued".to_string())
        );

        // Percent prompt (csh)
        let buffer = MockBuffer::new(vec!["% ls"]);
        let zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        assert_eq!(
            TextExtractor::extract_command(&buffer, &zone),
            Some("ls".to_string())
        );
    }

    #[test]
    fn test_extract_output() {
        let buffer = create_test_buffer();

        // Create a zone that spans lines 0-4 (prompt + command + output)
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        zone.end_line = Some(5);

        let output = TextExtractor::extract_output(&buffer, &zone);
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(output.contains("ls -la"));
        assert!(output.contains("total 0"));
    }

    #[test]
    fn test_context_summary() {
        let buffer = create_test_buffer();

        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        zone.command = Some("ls -la".to_string());
        zone.state = CommandState::CommandFinished(0);
        zone.end_line = Some(5);

        let summary = TextExtractor::get_context_summary(&buffer, &zone, 1000);

        assert!(summary.command.is_some());
        assert_eq!(summary.command.as_deref(), Some("ls -la"));
        assert!(summary.output.is_some());
        assert_eq!(summary.exit_code, Some(0));
    }
}

// ============================================================================
// Integration Tests - Full Flow
// ============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_full_command_flow() {
        let mut scanner = OscScanner::new();
        let mut handler = ShellIntegrationHandler::new();

        // Simulate a complete command session
        let pty_output = b"\
            \x1b]133;A\x07\
            user@host:~$ \
            \x1b]133;B\x07\
            \x1b]633;E;ls%20-la\x07\
            \x1b]133;C\x07\
            total 0\r\n\
            \x1b]133;D;0\x07\
        ";

        // Scan for OSC sequences
        let sequences = scanner.scan(pty_output);
        assert_eq!(sequences.len(), 5);

        // Process sequences
        for seq in sequences {
            match seq {
                OscSequence::PromptStart => {
                    handler.handle_osc(b"133;A");
                }
                OscSequence::CommandStart => {
                    handler.handle_osc(b"133;B");
                }
                OscSequence::CommandExecuting => {
                    handler.handle_osc(b"133;C");
                }
                OscSequence::CommandFinished { exit_code } => {
                    handler.handle_osc(format!("133;D;{}", exit_code).as_bytes());
                }
                OscSequence::CommandText { command } => {
                    let encoded: String = command
                        .chars()
                        .map(|c| {
                            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                                c.to_string()
                            } else {
                                format!("%{:02X}", c as u8)
                            }
                        })
                        .collect();
                    handler.handle_osc(format!("633;E;{}", encoded).as_bytes());
                }
                _ => {}
            }
        }

        // Verify zone was created and has correct state
        let zones: Vec<_> = handler.zone_manager().zones().collect();
        assert_eq!(zones.len(), 1);

        let zone = zones[0];
        assert!(zone.is_finished());
        assert_eq!(zone.state.exit_code(), Some(0));
        assert_eq!(zone.command.as_deref(), Some("ls -la"));
    }

    #[test]
    fn test_multiple_commands_session() {
        let mut handler = ShellIntegrationHandler::new();

        // Command 1: successful
        handler.set_current_line(0);
        handler.handle_osc(b"133;A");
        handler.set_current_line(1);
        handler.handle_osc(b"633;E;echo%20hello");
        handler.handle_osc(b"133;B");
        handler.handle_osc(b"133;C");
        handler.set_current_line(3);
        handler.handle_osc(b"133;D;0");

        // Command 2: failed
        handler.set_current_line(4);
        handler.handle_osc(b"133;A");
        handler.set_current_line(5);
        handler.handle_osc(b"633;E;cat%20nonexistent");
        handler.handle_osc(b"133;B");
        handler.handle_osc(b"133;C");
        handler.set_current_line(7);
        handler.handle_osc(b"133;D;1");

        let zones: Vec<_> = handler.zone_manager().zones().collect();
        assert_eq!(zones.len(), 2);
    }

    #[test]
    fn test_interrupted_command() {
        let mut handler = ShellIntegrationHandler::new();

        // Start command but don't finish
        handler.set_current_line(0);
        handler.handle_osc(b"133;A");
        handler.handle_osc(b"133;B");
        handler.handle_osc(b"133;C");

        // Start new command (simulating Ctrl+C)
        handler.set_current_line(5);
        handler.handle_osc(b"133;A");

        let zones: Vec<_> = handler.zone_manager().zones().collect();
        assert_eq!(zones.len(), 2);

        // First zone should be auto-closed
        let first_zone = zones.iter().find(|z| z.start_line == 0).unwrap();
        assert_eq!(first_zone.end_line, Some(5));
    }
}

// ============================================================================
// Edge Cases and Regression Tests
// ============================================================================

mod regression_tests {
    use super::*;

    #[test]
    fn test_empty_command() {
        let mut handler = ShellIntegrationHandler::new();

        handler.set_current_line(0);
        handler.handle_osc(b"133;A");
        handler.handle_osc(b"633;E;"); // Empty command
        handler.handle_osc(b"133;B");
        handler.set_current_line(1);
        handler.handle_osc(b"133;C");
        handler.set_current_line(3);
        handler.handle_osc(b"133;D;0");

        // Use zones() since zone_at_line requires proper line range
        let zones: Vec<_> = handler.zone_manager().zones().collect();
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].command.as_deref(), Some(""));
    }

    #[test]
    fn test_zone_id_uniqueness() {
        let mut manager = ZoneManager::new();

        let ids: Vec<ZoneId> = (0..1000)
            .map(|i| manager.start_zone(CommandState::PromptStart, i))
            .collect();

        // All IDs should be unique
        let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 1000);
    }

    #[test]
    fn test_large_output() {
        let lines: Vec<String> = (0..10000).map(|i| format!("line {}", i)).collect();
        let buffer = MockBuffer::from_strings(lines);

        let extracted = TextExtractor::extract_lines(&buffer, 0, Some(10000));
        assert_eq!(extracted.len(), 10000);
    }

    #[test]
    fn test_rapid_zone_creation() {
        let mut manager = ZoneManager::new();

        for i in 0..1000 {
            manager.start_zone(CommandState::PromptStart, i * 2);
            manager.transition_state(CommandState::CommandExecuting, i * 2);
            manager.finish_zone(i * 2 + 1, 0);
        }

        assert_eq!(manager.zones().count(), 1000);
    }
}
