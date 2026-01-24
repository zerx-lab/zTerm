//! Integration tests for shell integration OSC scanning

use zterm_terminal::shell_integration::{OscScanner, OscSequence};

#[test]
fn test_osc_scanner_integration() {
    let mut scanner = OscScanner::new();

    // Simulate PTY output with OSC 133 sequences embedded
    // ESC ] 133 ; A BEL
    let data = b"\x1b]133;A\x07Hello World\x1b]133;B\x07ls -la\x1b]133;C\x07";

    let sequences = scanner.scan(data);

    assert_eq!(sequences.len(), 3);
    assert_eq!(sequences[0], OscSequence::PromptStart);
    assert_eq!(sequences[1], OscSequence::CommandStart);
    assert_eq!(sequences[2], OscSequence::CommandExecuting);
}

#[test]
fn test_osc_scanner_with_command_text() {
    let mut scanner = OscScanner::new();

    // OSC 633;E;echo%20hello (VS Code command capture)
    let data = b"\x1b]633;E;echo%20hello\x07";

    let sequences = scanner.scan(data);

    assert_eq!(sequences.len(), 1);
    match &sequences[0] {
        OscSequence::CommandText { command } => {
            assert_eq!(command, "echo hello");
        }
        _ => panic!("Expected CommandText"),
    }
}

#[test]
fn test_osc_scanner_with_exit_code() {
    let mut scanner = OscScanner::new();

    // OSC 133;D;1 (command finished with error)
    let data = b"\x1b]133;D;1\x07";

    let sequences = scanner.scan(data);

    assert_eq!(sequences.len(), 1);
    match &sequences[0] {
        OscSequence::CommandFinished { exit_code } => {
            assert_eq!(*exit_code, 1);
        }
        _ => panic!("Expected CommandFinished"),
    }
}

#[test]
fn test_osc_scanner_mixed_with_normal_output() {
    let mut scanner = OscScanner::new();

    // Normal terminal output mixed with OSC sequences
    let data = b"user@host:~$ \x1b]133;A\x07\x1b]133;B\x07echo hello\r\nhello\r\n\x1b]133;D;0\x07";

    let sequences = scanner.scan(data);

    assert_eq!(sequences.len(), 3);
    assert_eq!(sequences[0], OscSequence::PromptStart);
    assert_eq!(sequences[1], OscSequence::CommandStart);
    assert!(matches!(
        sequences[2],
        OscSequence::CommandFinished { exit_code: 0 }
    ));
}

#[test]
fn test_osc7_working_directory() {
    let mut scanner = OscScanner::new();

    // OSC 7 ; file://hostname/path/to/dir
    let data = b"\x1b]7;file://localhost/home/user/project\x07";

    let sequences = scanner.scan(data);

    assert_eq!(sequences.len(), 1);
    match &sequences[0] {
        OscSequence::Osc7WorkingDirectory { path } => {
            assert_eq!(path, "/home/user/project");
        }
        _ => panic!("Expected Osc7WorkingDirectory"),
    }
}
