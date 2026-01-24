//! Shell integration OSC sequence handler
//!
//! Handles OSC 133 (FinalTerm) and OSC 633 (VS Code) sequences for shell integration.

use super::event::ShellEvent;
use super::zone::{CommandState, ZoneManager};

/// Handler for shell integration OSC sequences
#[derive(Debug)]
pub struct ShellIntegrationHandler {
    /// Zone manager for tracking command zones
    zone_manager: ZoneManager,
    /// Pending events to be dispatched
    pending_events: Vec<ShellEvent>,
    /// Current cursor line
    current_line: usize,
}

impl Default for ShellIntegrationHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellIntegrationHandler {
    /// Create a new shell integration handler
    pub fn new() -> Self {
        Self {
            zone_manager: ZoneManager::new(),
            pending_events: Vec::new(),
            current_line: 0,
        }
    }

    /// Update the current cursor line
    pub fn set_current_line(&mut self, line: usize) {
        self.current_line = line;
    }

    /// Get the zone manager
    pub fn zone_manager(&self) -> &ZoneManager {
        &self.zone_manager
    }

    /// Get mutable zone manager
    pub fn zone_manager_mut(&mut self) -> &mut ZoneManager {
        &mut self.zone_manager
    }

    /// Take pending events
    pub fn take_events(&mut self) -> Vec<ShellEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Parse and handle an OSC sequence
    ///
    /// Returns true if the sequence was handled as a shell integration sequence.
    pub fn handle_osc(&mut self, osc: &[u8]) -> bool {
        let osc_str = match std::str::from_utf8(osc) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // OSC 133 - FinalTerm shell integration
        if let Some(rest) = osc_str.strip_prefix("133;") {
            return self.handle_osc_133(rest);
        }

        // OSC 633 - VS Code shell integration
        if let Some(rest) = osc_str.strip_prefix("633;") {
            return self.handle_osc_633(rest);
        }

        // OSC 7 - Working directory
        if let Some(rest) = osc_str.strip_prefix("7;") {
            return self.handle_osc_7(rest);
        }

        false
    }

    /// Handle OSC 133 (FinalTerm) sequences
    fn handle_osc_133(&mut self, data: &str) -> bool {
        let mut parts = data.splitn(2, ';');
        let cmd = parts.next().unwrap_or("");
        let params = parts.next().unwrap_or("");

        match cmd {
            "A" => {
                // Prompt start
                let zone_id = self.zone_manager.start_zone(CommandState::PromptStart, self.current_line);
                tracing::info!(
                    "Shell integration: PromptStart at line {}, zone_id={:?}, total_zones={}",
                    self.current_line,
                    zone_id,
                    self.zone_manager.len()
                );
                self.pending_events.push(ShellEvent::PromptStarted {
                    zone_id,
                    line: self.current_line,
                });
                true
            }
            "B" => {
                // Command start (user input area)
                self.zone_manager.transition_state(CommandState::CommandStart, self.current_line);
                if let Some(zone) = self.zone_manager.active_zone() {
                    self.pending_events.push(ShellEvent::CommandStarted {
                        zone_id: zone.id,
                        line: self.current_line,
                    });
                }
                true
            }
            "C" => {
                // Command executing
                self.zone_manager.transition_state(CommandState::CommandExecuting, self.current_line);
                if let Some(zone) = self.zone_manager.active_zone() {
                    self.pending_events.push(ShellEvent::CommandExecuting {
                        zone_id: zone.id,
                        line: self.current_line,
                    });
                }
                true
            }
            "D" => {
                // Command finished
                let exit_code = self.parse_exit_code(params);
                let zone_id = self.zone_manager.active_zone().map(|z| z.id);
                let zone_start = self.zone_manager.active_zone().map(|z| z.start_line);
                self.zone_manager.finish_zone(self.current_line, exit_code);
                tracing::info!(
                    "Shell integration: CommandFinished zone_id={:?}, start={:?}, end={}, exit_code={}, total_zones={}",
                    zone_id,
                    zone_start,
                    self.current_line,
                    exit_code,
                    self.zone_manager.len()
                );
                if let Some(id) = zone_id {
                    self.pending_events.push(ShellEvent::CommandFinished {
                        zone_id: id,
                        line: self.current_line,
                        exit_code,
                    });
                }
                true
            }
            _ => false,
        }
    }

    /// Handle OSC 633 (VS Code) sequences
    fn handle_osc_633(&mut self, data: &str) -> bool {
        let mut parts = data.splitn(2, ';');
        let cmd = parts.next().unwrap_or("");
        let params = parts.next().unwrap_or("");

        match cmd {
            "A" => self.handle_osc_133("A"),
            "B" => self.handle_osc_133("B"),
            "C" => self.handle_osc_133("C"),
            "D" => {
                let params_for_133 = if params.is_empty() { "" } else { params };
                self.handle_osc_133(&format!("D;{}", params_for_133))
            }
            "E" => {
                // Command line (VS Code specific)
                let command = self.decode_command(params);
                self.zone_manager.set_command(command.clone());
                if let Some(zone) = self.zone_manager.active_zone() {
                    self.pending_events.push(ShellEvent::CommandCaptured {
                        zone_id: zone.id,
                        command,
                    });
                }
                true
            }
            "P" => {
                // Property (e.g., Cwd=...)
                if let Some(cwd) = params.strip_prefix("Cwd=") {
                    self.zone_manager.set_working_directory(cwd.to_string());
                }
                true
            }
            _ => false,
        }
    }

    /// Handle OSC 7 (working directory)
    fn handle_osc_7(&mut self, data: &str) -> bool {
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

        let path = Self::decode_percent(path);
        self.zone_manager.set_working_directory(path.clone());
        self.pending_events.push(ShellEvent::WorkingDirectoryChanged { path });
        true
    }

    /// Parse exit code from OSC 133 D parameters
    fn parse_exit_code(&self, params: &str) -> i32 {
        // Format can be: empty, just a number, or key=value pairs
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

    /// Decode command from OSC 633 E (may be base64 or percent encoded)
    fn decode_command(&self, encoded: &str) -> String {
        // Try percent decoding first
        Self::decode_percent(encoded)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_osc(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }

    // ===== OSC 133 A Tests =====

    #[test]
    fn test_osc_133_a_prompt_start() {
        let mut handler = ShellIntegrationHandler::new();
        handler.set_current_line(10);

        let result = handler.handle_osc(&make_osc("133;A"));
        assert!(result);

        let events = handler.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ShellEvent::PromptStarted { line: 10, .. }));
    }

    #[test]
    fn test_osc_133_a_creates_zone() {
        let mut handler = ShellIntegrationHandler::new();
        handler.set_current_line(5);

        handler.handle_osc(&make_osc("133;A"));

        let zone = handler.zone_manager().active_zone();
        assert!(zone.is_some());
        assert_eq!(zone.unwrap().start_line, 5);
        assert_eq!(zone.unwrap().state, CommandState::PromptStart);
    }

    // ===== OSC 133 B Tests =====

    #[test]
    fn test_osc_133_b_command_start() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("133;A"));
        handler.take_events();

        handler.set_current_line(11);
        let result = handler.handle_osc(&make_osc("133;B"));
        assert!(result);

        let events = handler.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ShellEvent::CommandStarted { line: 11, .. }));
    }

    #[test]
    fn test_osc_133_b_transitions_state() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("133;A"));
        handler.handle_osc(&make_osc("133;B"));

        let zone = handler.zone_manager().active_zone().unwrap();
        assert_eq!(zone.state, CommandState::CommandStart);
    }

    // ===== OSC 133 C Tests =====

    #[test]
    fn test_osc_133_c_command_executing() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("133;A"));
        handler.handle_osc(&make_osc("133;B"));
        handler.take_events();

        handler.set_current_line(12);
        let result = handler.handle_osc(&make_osc("133;C"));
        assert!(result);

        let events = handler.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ShellEvent::CommandExecuting { line: 12, .. }));
    }

    #[test]
    fn test_osc_133_c_transitions_state() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("133;A"));
        handler.handle_osc(&make_osc("133;B"));
        handler.handle_osc(&make_osc("133;C"));

        let zone = handler.zone_manager().active_zone().unwrap();
        assert!(zone.state.is_running());
    }

    // ===== OSC 133 D Tests =====

    #[test]
    fn test_osc_133_d_command_finished() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("133;A"));
        handler.handle_osc(&make_osc("133;B"));
        handler.handle_osc(&make_osc("133;C"));
        handler.take_events();

        handler.set_current_line(20);
        let result = handler.handle_osc(&make_osc("133;D"));
        assert!(result);

        let events = handler.take_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            ShellEvent::CommandFinished { line, exit_code, .. } => {
                assert_eq!(*line, 20);
                assert_eq!(*exit_code, 0);
            }
            _ => panic!("Expected CommandFinished event"),
        }
    }

    #[test]
    fn test_osc_133_d_with_exit_code() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("133;A"));
        handler.handle_osc(&make_osc("133;C"));
        handler.take_events();

        handler.handle_osc(&make_osc("133;D;1"));

        let events = handler.take_events();
        assert_eq!(events[0].exit_code(), Some(1));
    }

    #[test]
    fn test_osc_133_d_with_negative_exit_code() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("133;A"));
        handler.handle_osc(&make_osc("133;C"));
        handler.take_events();

        handler.handle_osc(&make_osc("133;D;-1"));

        let events = handler.take_events();
        assert_eq!(events[0].exit_code(), Some(-1));
    }

    #[test]
    fn test_osc_133_d_finishes_zone() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("133;A"));
        handler.handle_osc(&make_osc("133;C"));
        handler.set_current_line(15);
        handler.handle_osc(&make_osc("133;D;0"));

        // Zone should no longer be active
        assert!(handler.zone_manager().active_zone().is_none());

        // But we can still find it
        let zone = handler.zone_manager().zone_at_line(0);
        assert!(zone.is_some());
        assert!(zone.unwrap().is_finished());
    }

    // ===== OSC 633 Tests =====

    #[test]
    fn test_osc_633_a_maps_to_133_a() {
        let mut handler = ShellIntegrationHandler::new();

        let result = handler.handle_osc(&make_osc("633;A"));
        assert!(result);

        let events = handler.take_events();
        assert!(matches!(events[0], ShellEvent::PromptStarted { .. }));
    }

    #[test]
    fn test_osc_633_e_command_capture() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("633;A"));
        handler.handle_osc(&make_osc("633;B"));
        handler.take_events();

        let result = handler.handle_osc(&make_osc("633;E;ls -la"));
        assert!(result);

        let zone = handler.zone_manager().active_zone().unwrap();
        assert_eq!(zone.command.as_deref(), Some("ls -la"));

        let events = handler.take_events();
        assert!(matches!(
            &events[0],
            ShellEvent::CommandCaptured { command, .. } if command == "ls -la"
        ));
    }

    #[test]
    fn test_osc_633_e_percent_encoded() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("633;A"));
        handler.take_events();

        // "hello world" with space encoded as %20
        handler.handle_osc(&make_osc("633;E;hello%20world"));

        let zone = handler.zone_manager().active_zone().unwrap();
        assert_eq!(zone.command.as_deref(), Some("hello world"));
    }

    #[test]
    fn test_osc_633_e_special_chars() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("633;A"));

        handler.handle_osc(&make_osc("633;E;echo%20%22test%22"));

        let zone = handler.zone_manager().active_zone().unwrap();
        assert_eq!(zone.command.as_deref(), Some("echo \"test\""));
    }

    #[test]
    fn test_osc_633_d_with_exit_code() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("633;A"));
        handler.handle_osc(&make_osc("633;C"));
        handler.take_events();

        handler.handle_osc(&make_osc("633;D;127"));

        let events = handler.take_events();
        assert_eq!(events[0].exit_code(), Some(127));
    }

    #[test]
    fn test_osc_633_p_cwd() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("633;A"));

        let result = handler.handle_osc(&make_osc("633;P;Cwd=/home/user"));
        assert!(result);

        let zone = handler.zone_manager().active_zone().unwrap();
        assert_eq!(zone.working_directory.as_deref(), Some("/home/user"));
    }

    // ===== OSC 7 Tests =====

    #[test]
    fn test_osc_7_working_directory() {
        let mut handler = ShellIntegrationHandler::new();
        handler.handle_osc(&make_osc("633;A"));
        handler.take_events();

        let result = handler.handle_osc(&make_osc("7;file://localhost/home/user"));
        assert!(result);

        let events = handler.take_events();
        match &events[0] {
            ShellEvent::WorkingDirectoryChanged { path } => {
                assert_eq!(path, "/home/user");
            }
            _ => panic!("Expected WorkingDirectoryChanged"),
        }
    }

    #[test]
    fn test_osc_7_without_host() {
        let mut handler = ShellIntegrationHandler::new();

        handler.handle_osc(&make_osc("7;file:///home/user"));

        let events = handler.take_events();
        match &events[0] {
            ShellEvent::WorkingDirectoryChanged { path } => {
                assert_eq!(path, "/home/user");
            }
            _ => panic!("Expected WorkingDirectoryChanged"),
        }
    }

    #[test]
    fn test_osc_7_percent_encoded() {
        let mut handler = ShellIntegrationHandler::new();

        handler.handle_osc(&make_osc("7;file:///home/user/my%20folder"));

        let events = handler.take_events();
        match &events[0] {
            ShellEvent::WorkingDirectoryChanged { path } => {
                assert_eq!(path, "/home/user/my folder");
            }
            _ => panic!("Expected WorkingDirectoryChanged"),
        }
    }

    // ===== Full Lifecycle Tests =====

    #[test]
    fn test_full_command_lifecycle() {
        let mut handler = ShellIntegrationHandler::new();

        // A: Prompt starts
        handler.set_current_line(0);
        handler.handle_osc(&make_osc("133;A"));

        // B: User starts typing
        handler.set_current_line(1);
        handler.handle_osc(&make_osc("133;B"));

        // E: Command captured
        handler.handle_osc(&make_osc("633;E;ls -la"));

        // C: Command executes
        handler.set_current_line(1);
        handler.handle_osc(&make_osc("133;C"));

        // D: Command finishes
        handler.set_current_line(10);
        handler.handle_osc(&make_osc("133;D;0"));

        let events = handler.take_events();
        assert_eq!(events.len(), 5);
        assert!(matches!(events[0], ShellEvent::PromptStarted { .. }));
        assert!(matches!(events[1], ShellEvent::CommandStarted { .. }));
        assert!(matches!(events[2], ShellEvent::CommandCaptured { .. }));
        assert!(matches!(events[3], ShellEvent::CommandExecuting { .. }));
        assert!(matches!(events[4], ShellEvent::CommandFinished { .. }));
    }

    #[test]
    fn test_unknown_osc_not_handled() {
        let mut handler = ShellIntegrationHandler::new();

        let result = handler.handle_osc(&make_osc("999;unknown"));
        assert!(!result);
        assert!(handler.take_events().is_empty());
    }

    #[test]
    fn test_invalid_utf8_not_handled() {
        let mut handler = ShellIntegrationHandler::new();

        let result = handler.handle_osc(&[0x80, 0x81, 0x82]);
        assert!(!result);
    }

    #[test]
    fn test_handler_default() {
        let handler = ShellIntegrationHandler::default();
        assert!(handler.zone_manager().is_empty());
    }

    // ===== Parse Exit Code Tests =====

    #[test]
    fn test_parse_exit_code_empty() {
        let handler = ShellIntegrationHandler::new();
        assert_eq!(handler.parse_exit_code(""), 0);
    }

    #[test]
    fn test_parse_exit_code_number() {
        let handler = ShellIntegrationHandler::new();
        assert_eq!(handler.parse_exit_code("0"), 0);
        assert_eq!(handler.parse_exit_code("1"), 1);
        assert_eq!(handler.parse_exit_code("127"), 127);
    }

    #[test]
    fn test_parse_exit_code_err_format() {
        let handler = ShellIntegrationHandler::new();
        assert_eq!(handler.parse_exit_code("err=1"), 1);
        assert_eq!(handler.parse_exit_code("err=127"), 127);
    }
}
