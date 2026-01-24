//! Zone management for shell integration
//!
//! This module provides data structures for tracking command zones in the terminal.
//! A "zone" represents a logical region of terminal output, such as a prompt,
//! command input, or command output.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Unique identifier for a command zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoneId(u64);

impl ZoneId {
    /// Create a new ZoneId with the given value
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the inner value
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// State of a command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandState {
    /// Prompt is being displayed (OSC 133 A)
    PromptStart,
    /// User is entering a command (OSC 133 B)
    CommandStart,
    /// Command is executing (OSC 133 C)
    CommandExecuting,
    /// Command has finished with an exit code (OSC 133 D)
    CommandFinished(i32),
}

impl CommandState {
    /// Check if the command is still running
    pub fn is_running(&self) -> bool {
        matches!(self, CommandState::CommandExecuting)
    }

    /// Check if the command has finished
    pub fn is_finished(&self) -> bool {
        matches!(self, CommandState::CommandFinished(_))
    }

    /// Get the exit code if finished
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            CommandState::CommandFinished(code) => Some(*code),
            _ => None,
        }
    }

    /// Check if command succeeded (exit code 0)
    pub fn is_success(&self) -> bool {
        matches!(self, CommandState::CommandFinished(0))
    }

    /// Check if command failed (non-zero exit code)
    pub fn is_failure(&self) -> bool {
        match self {
            CommandState::CommandFinished(code) => *code != 0,
            _ => false,
        }
    }
}

/// A command zone representing a logical region in the terminal
#[derive(Debug, Clone)]
pub struct CommandZone {
    /// Unique identifier for this zone
    pub id: ZoneId,
    /// Current state of the command
    pub state: CommandState,
    /// Start line (0-indexed, absolute line in scrollback)
    pub start_line: usize,
    /// End line (exclusive, None if zone is still active)
    pub end_line: Option<usize>,
    /// The command text (if captured via OSC 633 E)
    pub command: Option<String>,
    /// Working directory when command started
    pub working_directory: Option<String>,
    /// Timestamp when zone started
    pub started_at: Instant,
    /// Timestamp when zone finished
    pub finished_at: Option<Instant>,
}

impl CommandZone {
    /// Create a new command zone
    pub fn new(id: ZoneId, state: CommandState, start_line: usize) -> Self {
        Self {
            id,
            state,
            start_line,
            end_line: None,
            command: None,
            working_directory: None,
            started_at: Instant::now(),
            finished_at: None,
        }
    }

    /// Check if a line is within this zone
    pub fn contains_line(&self, line: usize) -> bool {
        match self.end_line {
            Some(end) => line >= self.start_line && line < end,
            None => line >= self.start_line,
        }
    }

    /// Get the line range of this zone
    pub fn line_range(&self) -> (usize, Option<usize>) {
        (self.start_line, self.end_line)
    }

    /// Get the duration of command execution
    pub fn duration(&self) -> Option<Duration> {
        self.finished_at
            .map(|end| end.duration_since(self.started_at))
    }

    /// Get the number of lines in this zone
    pub fn line_count(&self) -> Option<usize> {
        self.end_line.map(|end| end.saturating_sub(self.start_line))
    }

    /// Set the zone as finished
    pub fn finish(&mut self, end_line: usize, exit_code: i32) {
        self.end_line = Some(end_line);
        self.state = CommandState::CommandFinished(exit_code);
        self.finished_at = Some(Instant::now());
    }
}

/// Manager for tracking command zones
#[derive(Debug)]
pub struct ZoneManager {
    /// All zones, keyed by their ID
    zones: HashMap<ZoneId, CommandZone>,
    /// Active zone (if any)
    active_zone_id: Option<ZoneId>,
    /// Next zone ID to assign
    next_id: u64,
    /// Zones ordered by start line for efficient lookup
    zones_by_line: Vec<ZoneId>,
}

impl Default for ZoneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoneManager {
    /// Create a new zone manager
    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
            active_zone_id: None,
            next_id: 0,
            zones_by_line: Vec::new(),
        }
    }

    /// Start a new zone with the given state and line
    pub fn start_zone(&mut self, state: CommandState, start_line: usize) -> ZoneId {
        // Finish any active zone first
        if let Some(active_id) = self.active_zone_id.take() {
            if let Some(zone) = self.zones.get_mut(&active_id) {
                if zone.end_line.is_none() {
                    zone.end_line = Some(start_line);
                }
            }
        }

        let id = ZoneId::new(self.next_id);
        self.next_id += 1;

        let zone = CommandZone::new(id, state, start_line);
        self.zones.insert(id, zone);
        self.zones_by_line.push(id);
        self.active_zone_id = Some(id);

        id
    }

    /// Transition the active zone to a new state
    pub fn transition_state(&mut self, new_state: CommandState, line: usize) {
        if let Some(active_id) = self.active_zone_id {
            if let Some(zone) = self.zones.get_mut(&active_id) {
                zone.state = new_state;
                if new_state.is_finished() {
                    zone.end_line = Some(line);
                    zone.finished_at = Some(Instant::now());
                }
            }
        }
    }

    /// Finish the active zone with the given exit code
    pub fn finish_zone(&mut self, end_line: usize, exit_code: i32) {
        if let Some(active_id) = self.active_zone_id.take() {
            if let Some(zone) = self.zones.get_mut(&active_id) {
                zone.finish(end_line, exit_code);
            }
        }
    }

    /// Set the command text for the active zone
    pub fn set_command(&mut self, command: String) {
        if let Some(active_id) = self.active_zone_id {
            if let Some(zone) = self.zones.get_mut(&active_id) {
                zone.command = Some(command);
            }
        }
    }

    /// Set the working directory for the active zone
    pub fn set_working_directory(&mut self, dir: String) {
        if let Some(active_id) = self.active_zone_id {
            if let Some(zone) = self.zones.get_mut(&active_id) {
                zone.working_directory = Some(dir);
            }
        }
    }

    /// Get a zone by ID
    pub fn get(&self, id: ZoneId) -> Option<&CommandZone> {
        self.zones.get(&id)
    }

    /// Get a mutable zone by ID
    pub fn get_mut(&mut self, id: ZoneId) -> Option<&mut CommandZone> {
        self.zones.get_mut(&id)
    }

    /// Get the active zone
    pub fn active_zone(&self) -> Option<&CommandZone> {
        self.active_zone_id.and_then(|id| self.zones.get(&id))
    }

    /// Get the zone containing the given line
    pub fn zone_at_line(&self, line: usize) -> Option<&CommandZone> {
        // Search in reverse order (most recent first)
        for id in self.zones_by_line.iter().rev() {
            if let Some(zone) = self.zones.get(id) {
                if zone.contains_line(line) {
                    return Some(zone);
                }
            }
        }
        None
    }

    /// Find the previous prompt zone from the given line
    ///
    /// This returns the nearest zone that ends before or at the given line,
    /// skipping the zone that contains the current line.
    pub fn previous_prompt(&self, from_line: usize) -> Option<&CommandZone> {
        // First, find the zone containing the current line (if any)
        let current_zone_id = self.zone_at_line(from_line).map(|z| z.id);

        for id in self.zones_by_line.iter().rev() {
            if let Some(zone) = self.zones.get(id) {
                // Skip the zone containing the current line
                if Some(zone.id) == current_zone_id {
                    continue;
                }

                // Zone must end before or at the current line
                if zone.start_line < from_line {
                    return Some(zone);
                }
            }
        }
        None
    }

    /// Find the next prompt zone from the given line
    pub fn next_prompt(&self, from_line: usize) -> Option<&CommandZone> {
        for id in &self.zones_by_line {
            if let Some(zone) = self.zones.get(id) {
                if zone.start_line > from_line {
                    return Some(zone);
                }
            }
        }
        None
    }

    /// Get all zones
    pub fn zones(&self) -> impl Iterator<Item = &CommandZone> {
        self.zones.values()
    }

    /// Get the number of zones
    pub fn len(&self) -> usize {
        self.zones.len()
    }

    /// Check if there are no zones
    pub fn is_empty(&self) -> bool {
        self.zones.is_empty()
    }

    /// Clear all zones
    pub fn clear(&mut self) {
        self.zones.clear();
        self.zones_by_line.clear();
        self.active_zone_id = None;
        self.next_id = 0;
    }
}

impl CommandZone {
    /// Check if the zone has finished
    pub fn is_finished(&self) -> bool {
        self.state.is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ZoneId Tests =====

    #[test]
    fn test_zone_id_new() {
        let id = ZoneId::new(42);
        assert_eq!(id.inner(), 42);
    }

    #[test]
    fn test_zone_id_equality() {
        let id1 = ZoneId::new(1);
        let id2 = ZoneId::new(1);
        let id3 = ZoneId::new(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_zone_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ZoneId::new(1));
        set.insert(ZoneId::new(2));
        set.insert(ZoneId::new(1)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_zone_id_clone() {
        let id1 = ZoneId::new(5);
        let id2 = id1;
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_zone_id_debug() {
        let id = ZoneId::new(123);
        let debug = format!("{:?}", id);
        assert!(debug.contains("123"));
    }

    // ===== CommandState Tests =====

    #[test]
    fn test_command_state_prompt_start() {
        let state = CommandState::PromptStart;
        assert!(!state.is_running());
        assert!(!state.is_finished());
        assert!(state.exit_code().is_none());
        assert!(!state.is_success());
        assert!(!state.is_failure());
    }

    #[test]
    fn test_command_state_command_start() {
        let state = CommandState::CommandStart;
        assert!(!state.is_running());
        assert!(!state.is_finished());
    }

    #[test]
    fn test_command_state_executing() {
        let state = CommandState::CommandExecuting;
        assert!(state.is_running());
        assert!(!state.is_finished());
    }

    #[test]
    fn test_command_state_finished_success() {
        let state = CommandState::CommandFinished(0);
        assert!(!state.is_running());
        assert!(state.is_finished());
        assert_eq!(state.exit_code(), Some(0));
        assert!(state.is_success());
        assert!(!state.is_failure());
    }

    #[test]
    fn test_command_state_finished_failure() {
        let state = CommandState::CommandFinished(1);
        assert!(state.is_finished());
        assert_eq!(state.exit_code(), Some(1));
        assert!(!state.is_success());
        assert!(state.is_failure());
    }

    #[test]
    fn test_command_state_finished_negative_exit() {
        let state = CommandState::CommandFinished(-1);
        assert!(state.is_finished());
        assert_eq!(state.exit_code(), Some(-1));
        assert!(state.is_failure());
    }

    #[test]
    fn test_command_state_equality() {
        assert_eq!(CommandState::PromptStart, CommandState::PromptStart);
        assert_eq!(
            CommandState::CommandFinished(0),
            CommandState::CommandFinished(0)
        );
        assert_ne!(
            CommandState::CommandFinished(0),
            CommandState::CommandFinished(1)
        );
    }

    #[test]
    fn test_command_state_clone() {
        let state = CommandState::CommandFinished(42);
        let cloned = state;
        assert_eq!(state, cloned);
    }

    // ===== CommandZone Tests =====

    #[test]
    fn test_command_zone_new() {
        let zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 10);
        assert_eq!(zone.id, ZoneId::new(0));
        assert_eq!(zone.state, CommandState::PromptStart);
        assert_eq!(zone.start_line, 10);
        assert!(zone.end_line.is_none());
        assert!(zone.command.is_none());
    }

    #[test]
    fn test_command_zone_contains_line_open_ended() {
        let zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 5);
        assert!(!zone.contains_line(4));
        assert!(zone.contains_line(5));
        assert!(zone.contains_line(100));
    }

    #[test]
    fn test_command_zone_contains_line_closed() {
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 5);
        zone.end_line = Some(10);

        assert!(!zone.contains_line(4));
        assert!(zone.contains_line(5));
        assert!(zone.contains_line(9));
        assert!(!zone.contains_line(10)); // Exclusive end
    }

    #[test]
    fn test_command_zone_line_range() {
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 5);
        assert_eq!(zone.line_range(), (5, None));

        zone.end_line = Some(15);
        assert_eq!(zone.line_range(), (5, Some(15)));
    }

    #[test]
    fn test_command_zone_line_count() {
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 5);
        assert!(zone.line_count().is_none());

        zone.end_line = Some(15);
        assert_eq!(zone.line_count(), Some(10));
    }

    #[test]
    fn test_command_zone_duration() {
        let zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        assert!(zone.duration().is_none());
    }

    #[test]
    fn test_command_zone_finish() {
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::CommandExecuting, 5);
        zone.finish(20, 0);

        assert_eq!(zone.end_line, Some(20));
        assert_eq!(zone.state, CommandState::CommandFinished(0));
        assert!(zone.finished_at.is_some());
    }

    #[test]
    fn test_command_zone_is_finished() {
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        assert!(!zone.is_finished());

        zone.state = CommandState::CommandFinished(0);
        assert!(zone.is_finished());
    }

    #[test]
    fn test_command_zone_with_command() {
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::CommandStart, 0);
        zone.command = Some("ls -la".to_string());
        assert_eq!(zone.command.as_deref(), Some("ls -la"));
    }

    #[test]
    fn test_command_zone_with_working_directory() {
        let mut zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        zone.working_directory = Some("/home/user".to_string());
        assert_eq!(zone.working_directory.as_deref(), Some("/home/user"));
    }

    #[test]
    fn test_command_zone_edge_case_zero_line() {
        let zone = CommandZone::new(ZoneId::new(0), CommandState::PromptStart, 0);
        assert!(zone.contains_line(0));
    }

    // ===== ZoneManager Tests =====

    #[test]
    fn test_zone_manager_new() {
        let manager = ZoneManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert!(manager.active_zone().is_none());
    }

    #[test]
    fn test_zone_manager_default() {
        let manager = ZoneManager::default();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_zone_manager_start_zone() {
        let mut manager = ZoneManager::new();
        let id = manager.start_zone(CommandState::PromptStart, 0);

        assert_eq!(manager.len(), 1);
        assert!(manager.active_zone().is_some());
        assert_eq!(manager.active_zone().unwrap().id, id);
    }

    #[test]
    fn test_zone_manager_unique_ids() {
        let mut manager = ZoneManager::new();
        let id1 = manager.start_zone(CommandState::PromptStart, 0);
        let id2 = manager.start_zone(CommandState::PromptStart, 10);
        let id3 = manager.start_zone(CommandState::PromptStart, 20);

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_zone_manager_lifecycle_a_b_c_d() {
        let mut manager = ZoneManager::new();

        // A: Prompt start
        let id = manager.start_zone(CommandState::PromptStart, 0);
        assert_eq!(
            manager.active_zone().unwrap().state,
            CommandState::PromptStart
        );

        // B: Command start (user typing)
        manager.transition_state(CommandState::CommandStart, 1);
        assert_eq!(manager.get(id).unwrap().state, CommandState::CommandStart);

        // C: Command executing
        manager.transition_state(CommandState::CommandExecuting, 2);
        assert_eq!(
            manager.get(id).unwrap().state,
            CommandState::CommandExecuting
        );

        // D: Command finished
        manager.finish_zone(10, 0);
        let zone = manager.get(id).unwrap();
        assert!(zone.state.is_finished());
        assert_eq!(zone.end_line, Some(10));
    }

    #[test]
    fn test_zone_manager_set_command() {
        let mut manager = ZoneManager::new();
        manager.start_zone(CommandState::CommandStart, 0);
        manager.set_command("echo hello".to_string());

        assert_eq!(
            manager.active_zone().unwrap().command.as_deref(),
            Some("echo hello")
        );
    }

    #[test]
    fn test_zone_manager_set_working_directory() {
        let mut manager = ZoneManager::new();
        manager.start_zone(CommandState::PromptStart, 0);
        manager.set_working_directory("/home/user/project".to_string());

        assert_eq!(
            manager.active_zone().unwrap().working_directory.as_deref(),
            Some("/home/user/project")
        );
    }

    #[test]
    fn test_zone_manager_zone_at_line() {
        let mut manager = ZoneManager::new();

        manager.start_zone(CommandState::PromptStart, 0);
        manager.finish_zone(10, 0);

        manager.start_zone(CommandState::PromptStart, 10);
        manager.finish_zone(20, 0);

        assert!(manager.zone_at_line(0).is_some());
        assert_eq!(manager.zone_at_line(0).unwrap().start_line, 0);

        assert!(manager.zone_at_line(5).is_some());
        assert_eq!(manager.zone_at_line(5).unwrap().start_line, 0);

        assert!(manager.zone_at_line(10).is_some());
        assert_eq!(manager.zone_at_line(10).unwrap().start_line, 10);

        assert!(manager.zone_at_line(15).is_some());
        assert_eq!(manager.zone_at_line(15).unwrap().start_line, 10);
    }

    #[test]
    fn test_zone_manager_previous_prompt() {
        let mut manager = ZoneManager::new();

        manager.start_zone(CommandState::PromptStart, 0);
        manager.finish_zone(10, 0);

        manager.start_zone(CommandState::PromptStart, 10);
        manager.finish_zone(20, 0);

        let prev = manager.previous_prompt(25);
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().start_line, 10);

        let prev = manager.previous_prompt(15);
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().start_line, 0);
    }

    #[test]
    fn test_zone_manager_next_prompt() {
        let mut manager = ZoneManager::new();

        manager.start_zone(CommandState::PromptStart, 0);
        manager.finish_zone(10, 0);

        manager.start_zone(CommandState::PromptStart, 10);
        manager.finish_zone(20, 0);

        let next = manager.next_prompt(5);
        assert!(next.is_some());
        assert_eq!(next.unwrap().start_line, 10);

        let next = manager.next_prompt(15);
        assert!(next.is_none());
    }

    #[test]
    fn test_zone_manager_get_by_id() {
        let mut manager = ZoneManager::new();
        let id = manager.start_zone(CommandState::PromptStart, 0);

        assert!(manager.get(id).is_some());
        assert!(manager.get(ZoneId::new(999)).is_none());
    }

    #[test]
    fn test_zone_manager_get_mut() {
        let mut manager = ZoneManager::new();
        let id = manager.start_zone(CommandState::PromptStart, 0);

        if let Some(zone) = manager.get_mut(id) {
            zone.command = Some("test".to_string());
        }

        assert_eq!(manager.get(id).unwrap().command.as_deref(), Some("test"));
    }

    #[test]
    fn test_zone_manager_clear() {
        let mut manager = ZoneManager::new();
        manager.start_zone(CommandState::PromptStart, 0);
        manager.start_zone(CommandState::PromptStart, 10);

        assert_eq!(manager.len(), 2);

        manager.clear();

        assert!(manager.is_empty());
        assert!(manager.active_zone().is_none());
    }

    #[test]
    fn test_zone_manager_zones_iterator() {
        let mut manager = ZoneManager::new();
        manager.start_zone(CommandState::PromptStart, 0);
        manager.start_zone(CommandState::PromptStart, 10);
        manager.start_zone(CommandState::PromptStart, 20);

        let count = manager.zones().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_zone_manager_auto_close_previous() {
        let mut manager = ZoneManager::new();

        let id1 = manager.start_zone(CommandState::PromptStart, 0);
        let _id2 = manager.start_zone(CommandState::PromptStart, 10);

        // First zone should be auto-closed
        let zone1 = manager.get(id1).unwrap();
        assert_eq!(zone1.end_line, Some(10));
    }

    #[test]
    fn test_zone_manager_complex_lifecycle() {
        let mut manager = ZoneManager::new();

        // First command: ls
        manager.start_zone(CommandState::PromptStart, 0);
        manager.transition_state(CommandState::CommandStart, 1);
        manager.set_command("ls".to_string());
        manager.transition_state(CommandState::CommandExecuting, 1);
        manager.finish_zone(5, 0);

        // Second command: pwd (fails)
        manager.start_zone(CommandState::PromptStart, 5);
        manager.transition_state(CommandState::CommandStart, 6);
        manager.set_command("nonexistent".to_string());
        manager.transition_state(CommandState::CommandExecuting, 6);
        manager.finish_zone(7, 127);

        // Third command still running
        manager.start_zone(CommandState::PromptStart, 7);
        manager.transition_state(CommandState::CommandStart, 8);
        manager.set_command("sleep 100".to_string());
        manager.transition_state(CommandState::CommandExecuting, 8);

        assert_eq!(manager.len(), 3);

        let zones: Vec<_> = manager.zones().collect();
        let finished: Vec<_> = zones.iter().filter(|z| z.is_finished()).collect();
        assert_eq!(finished.len(), 2);

        assert!(manager.active_zone().is_some());
        assert!(manager.active_zone().unwrap().state.is_running());
    }
}
