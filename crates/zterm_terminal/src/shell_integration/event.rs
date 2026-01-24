//! Shell integration events
//!
//! This module defines events emitted by the shell integration system.

use super::zone::ZoneId;

/// Events emitted by the shell integration system
#[derive(Debug, Clone, PartialEq)]
pub enum ShellEvent {
    /// A new prompt has started (OSC 133 A)
    PromptStarted {
        zone_id: ZoneId,
        line: usize,
    },

    /// User started entering a command (OSC 133 B)
    CommandStarted {
        zone_id: ZoneId,
        line: usize,
    },

    /// Command execution has begun (OSC 133 C)
    CommandExecuting {
        zone_id: ZoneId,
        line: usize,
    },

    /// Command execution has finished (OSC 133 D)
    CommandFinished {
        zone_id: ZoneId,
        line: usize,
        exit_code: i32,
    },

    /// Command text was captured (OSC 633 E)
    CommandCaptured {
        zone_id: ZoneId,
        command: String,
    },

    /// Working directory changed (OSC 7)
    WorkingDirectoryChanged {
        path: String,
    },
}

impl ShellEvent {
    /// Get the zone ID if this event is associated with a zone
    pub fn zone_id(&self) -> Option<ZoneId> {
        match self {
            ShellEvent::PromptStarted { zone_id, .. } => Some(*zone_id),
            ShellEvent::CommandStarted { zone_id, .. } => Some(*zone_id),
            ShellEvent::CommandExecuting { zone_id, .. } => Some(*zone_id),
            ShellEvent::CommandFinished { zone_id, .. } => Some(*zone_id),
            ShellEvent::CommandCaptured { zone_id, .. } => Some(*zone_id),
            ShellEvent::WorkingDirectoryChanged { .. } => None,
        }
    }

    /// Get the line number if this event has one
    pub fn line(&self) -> Option<usize> {
        match self {
            ShellEvent::PromptStarted { line, .. } => Some(*line),
            ShellEvent::CommandStarted { line, .. } => Some(*line),
            ShellEvent::CommandExecuting { line, .. } => Some(*line),
            ShellEvent::CommandFinished { line, .. } => Some(*line),
            _ => None,
        }
    }

    /// Check if this is a prompt-related event
    pub fn is_prompt_event(&self) -> bool {
        matches!(self, ShellEvent::PromptStarted { .. })
    }

    /// Check if this is a command completion event
    pub fn is_completion_event(&self) -> bool {
        matches!(self, ShellEvent::CommandFinished { .. })
    }

    /// Get the exit code if this is a completion event
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            ShellEvent::CommandFinished { exit_code, .. } => Some(*exit_code),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_started_event() {
        let event = ShellEvent::PromptStarted {
            zone_id: ZoneId::new(1),
            line: 10,
        };

        assert_eq!(event.zone_id(), Some(ZoneId::new(1)));
        assert_eq!(event.line(), Some(10));
        assert!(event.is_prompt_event());
        assert!(!event.is_completion_event());
        assert!(event.exit_code().is_none());
    }

    #[test]
    fn test_command_started_event() {
        let event = ShellEvent::CommandStarted {
            zone_id: ZoneId::new(2),
            line: 15,
        };

        assert_eq!(event.zone_id(), Some(ZoneId::new(2)));
        assert_eq!(event.line(), Some(15));
        assert!(!event.is_prompt_event());
    }

    #[test]
    fn test_command_executing_event() {
        let event = ShellEvent::CommandExecuting {
            zone_id: ZoneId::new(3),
            line: 20,
        };

        assert_eq!(event.zone_id(), Some(ZoneId::new(3)));
        assert_eq!(event.line(), Some(20));
    }

    #[test]
    fn test_command_finished_event() {
        let event = ShellEvent::CommandFinished {
            zone_id: ZoneId::new(4),
            line: 25,
            exit_code: 0,
        };

        assert!(event.is_completion_event());
        assert_eq!(event.exit_code(), Some(0));
        assert_eq!(event.zone_id(), Some(ZoneId::new(4)));
        assert_eq!(event.line(), Some(25));
    }

    #[test]
    fn test_command_captured_event() {
        let event = ShellEvent::CommandCaptured {
            zone_id: ZoneId::new(5),
            command: "ls -la".to_string(),
        };

        assert_eq!(event.zone_id(), Some(ZoneId::new(5)));
        assert!(event.line().is_none());
    }

    #[test]
    fn test_working_directory_changed_event() {
        let event = ShellEvent::WorkingDirectoryChanged {
            path: "/home/user".to_string(),
        };

        assert!(event.zone_id().is_none());
        assert!(event.line().is_none());
        assert!(!event.is_prompt_event());
        assert!(!event.is_completion_event());
    }

    #[test]
    fn test_event_equality() {
        let event1 = ShellEvent::PromptStarted {
            zone_id: ZoneId::new(1),
            line: 10,
        };
        let event2 = ShellEvent::PromptStarted {
            zone_id: ZoneId::new(1),
            line: 10,
        };
        let event3 = ShellEvent::PromptStarted {
            zone_id: ZoneId::new(2),
            line: 10,
        };

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    #[test]
    fn test_event_clone() {
        let event = ShellEvent::CommandCaptured {
            zone_id: ZoneId::new(1),
            command: "echo hello".to_string(),
        };

        let cloned = event.clone();
        assert_eq!(event, cloned);
    }

    #[test]
    fn test_event_debug() {
        let event = ShellEvent::PromptStarted {
            zone_id: ZoneId::new(1),
            line: 0,
        };
        let debug = format!("{:?}", event);
        assert!(debug.contains("PromptStarted"));
    }
}
