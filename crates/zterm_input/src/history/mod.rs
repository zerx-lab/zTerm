//! Command history management

use zterm_common::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

/// Command history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The command text
    pub command: String,
    /// Timestamp when the command was executed
    pub timestamp: u64,
    /// Working directory when the command was executed
    pub working_directory: Option<String>,
    /// Exit code of the command (if known)
    pub exit_code: Option<i32>,
}

/// Command history manager
#[derive(Debug)]
pub struct History {
    /// History entries
    entries: VecDeque<HistoryEntry>,
    /// Maximum number of entries to keep
    max_entries: usize,
    /// Current position when navigating history
    position: Option<usize>,
    /// Path to history file
    file_path: Option<PathBuf>,
}

impl Default for History {
    fn default() -> Self {
        Self::new(10000)
    }
}

impl History {
    /// Create a new history with the given maximum size
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
            position: None,
            file_path: Self::default_history_path(),
        }
    }

    /// Get the default history file path
    fn default_history_path() -> Option<PathBuf> {
        dirs::data_dir().map(|p| p.join("zterm").join("history.json"))
    }

    /// Load history from file
    pub fn load(&mut self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                self.entries = serde_json::from_str(&content)?;
            }
        }
        Ok(())
    }

    /// Save history to file
    pub fn save(&self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(&self.entries)?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }

    /// Add a command to history
    pub fn add(
        &mut self,
        command: String,
        working_directory: Option<String>,
        exit_code: Option<i32>,
    ) {
        // Don't add empty commands or duplicates of the last command
        if command.trim().is_empty() {
            return;
        }

        if let Some(last) = self.entries.back() {
            if last.command == command {
                return;
            }
        }

        let entry = HistoryEntry {
            command,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            working_directory,
            exit_code,
        };

        self.entries.push_back(entry);

        // Trim if over limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        // Reset navigation position
        self.position = None;
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get a command at a specific index
    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    /// Navigate to the previous command in history
    pub fn previous(&mut self) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }

        let new_pos = match self.position {
            None => self.entries.len().saturating_sub(1),
            Some(0) => 0,
            Some(pos) => pos - 1,
        };

        self.position = Some(new_pos);
        self.entries.get(new_pos).map(|e| e.command.as_str())
    }

    /// Navigate to the next command in history
    pub fn next(&mut self) -> Option<&str> {
        match self.position {
            None => None,
            Some(pos) => {
                if pos + 1 >= self.entries.len() {
                    self.position = None;
                    None
                } else {
                    self.position = Some(pos + 1);
                    self.entries.get(pos + 1).map(|e| e.command.as_str())
                }
            }
        }
    }

    /// Reset navigation position
    pub fn reset_navigation(&mut self) {
        self.position = None;
    }

    /// Search history for commands containing the given query
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.command.contains(query))
            .collect()
    }

    /// Search history backwards for commands starting with the given prefix
    pub fn search_prefix(&self, prefix: &str) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.command.starts_with(prefix))
            .collect()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.position = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_add() {
        let mut history = History::new(100);
        history.add("ls".to_string(), None, None);
        history.add("cd".to_string(), None, None);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_history_navigation() {
        let mut history = History::new(100);
        history.add("cmd1".to_string(), None, None);
        history.add("cmd2".to_string(), None, None);
        history.add("cmd3".to_string(), None, None);

        assert_eq!(history.previous(), Some("cmd3"));
        assert_eq!(history.previous(), Some("cmd2"));
        assert_eq!(history.previous(), Some("cmd1"));
        assert_eq!(history.next(), Some("cmd2"));
    }

    #[test]
    fn test_history_no_duplicates() {
        let mut history = History::new(100);
        history.add("ls".to_string(), None, None);
        history.add("ls".to_string(), None, None);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_history_default() {
        let history = History::default();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_history_empty_command_ignored() {
        let mut history = History::new(100);
        history.add("".to_string(), None, None);
        history.add("   ".to_string(), None, None);
        assert!(history.is_empty());
    }

    #[test]
    fn test_history_max_entries() {
        let mut history = History::new(3);
        history.add("cmd1".to_string(), None, None);
        history.add("cmd2".to_string(), None, None);
        history.add("cmd3".to_string(), None, None);
        history.add("cmd4".to_string(), None, None);
        assert_eq!(history.len(), 3);
        // First command should be removed
        assert_eq!(history.get(0).unwrap().command, "cmd2");
    }

    #[test]
    fn test_history_get() {
        let mut history = History::new(100);
        history.add("test".to_string(), Some("/home".to_string()), Some(0));

        let entry = history.get(0).unwrap();
        assert_eq!(entry.command, "test");
        assert_eq!(entry.working_directory, Some("/home".to_string()));
        assert_eq!(entry.exit_code, Some(0));
    }

    #[test]
    fn test_history_get_out_of_bounds() {
        let history = History::new(100);
        assert!(history.get(0).is_none());
        assert!(history.get(100).is_none());
    }

    #[test]
    fn test_history_previous_empty() {
        let mut history = History::new(100);
        assert!(history.previous().is_none());
    }

    #[test]
    fn test_history_next_no_navigation() {
        let mut history = History::new(100);
        history.add("cmd".to_string(), None, None);
        // Without calling previous first, next should return None
        assert!(history.next().is_none());
    }

    #[test]
    fn test_history_navigation_boundaries() {
        let mut history = History::new(100);
        history.add("cmd1".to_string(), None, None);
        history.add("cmd2".to_string(), None, None);

        // Navigate to the oldest entry
        assert_eq!(history.previous(), Some("cmd2"));
        assert_eq!(history.previous(), Some("cmd1"));
        // At the oldest, previous should stay at oldest
        assert_eq!(history.previous(), Some("cmd1"));

        // Navigate back
        assert_eq!(history.next(), Some("cmd2"));
        // Past the newest, should return None and reset
        assert!(history.next().is_none());
    }

    #[test]
    fn test_history_reset_navigation() {
        let mut history = History::new(100);
        history.add("cmd1".to_string(), None, None);
        history.add("cmd2".to_string(), None, None);

        history.previous();
        history.reset_navigation();
        // After reset, previous starts from the end again
        assert_eq!(history.previous(), Some("cmd2"));
    }

    #[test]
    fn test_history_search() {
        let mut history = History::new(100);
        history.add("git status".to_string(), None, None);
        history.add("git commit".to_string(), None, None);
        history.add("ls -la".to_string(), None, None);

        let results = history.search("git");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_history_search_no_results() {
        let mut history = History::new(100);
        history.add("ls".to_string(), None, None);

        let results = history.search("git");
        assert!(results.is_empty());
    }

    #[test]
    fn test_history_search_prefix() {
        let mut history = History::new(100);
        history.add("git status".to_string(), None, None);
        history.add("git commit".to_string(), None, None);
        history.add("gzip file".to_string(), None, None);

        let results = history.search_prefix("git");
        assert_eq!(results.len(), 2);
        // Results are in reverse order
        assert_eq!(results[0].command, "git commit");
        assert_eq!(results[1].command, "git status");
    }

    #[test]
    fn test_history_clear() {
        let mut history = History::new(100);
        history.add("cmd1".to_string(), None, None);
        history.add("cmd2".to_string(), None, None);

        history.clear();
        assert!(history.is_empty());
        assert!(history.previous().is_none());
    }

    #[test]
    fn test_history_add_resets_navigation() {
        let mut history = History::new(100);
        history.add("cmd1".to_string(), None, None);
        history.add("cmd2".to_string(), None, None);

        history.previous(); // Navigate to cmd2
        history.add("cmd3".to_string(), None, None);

        // After adding, navigation should reset
        assert_eq!(history.previous(), Some("cmd3"));
    }

    #[test]
    fn test_history_entry_fields() {
        let entry = HistoryEntry {
            command: "test command".to_string(),
            timestamp: 1234567890,
            working_directory: Some("/home/user".to_string()),
            exit_code: Some(0),
        };

        assert_eq!(entry.command, "test command");
        assert_eq!(entry.timestamp, 1234567890);
        assert_eq!(entry.working_directory, Some("/home/user".to_string()));
        assert_eq!(entry.exit_code, Some(0));
    }

    #[test]
    fn test_history_entry_clone() {
        let entry = HistoryEntry {
            command: "test".to_string(),
            timestamp: 100,
            working_directory: None,
            exit_code: None,
        };

        let cloned = entry.clone();
        assert_eq!(entry.command, cloned.command);
        assert_eq!(entry.timestamp, cloned.timestamp);
    }

    #[test]
    fn test_history_non_consecutive_duplicates_allowed() {
        let mut history = History::new(100);
        history.add("ls".to_string(), None, None);
        history.add("cd".to_string(), None, None);
        history.add("ls".to_string(), None, None); // Not consecutive duplicate
        assert_eq!(history.len(), 3);
    }
}
