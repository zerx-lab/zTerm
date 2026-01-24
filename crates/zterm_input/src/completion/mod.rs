//! Auto-completion functionality

use std::path::PathBuf;

/// Completion suggestion
#[derive(Debug, Clone)]
pub struct Completion {
    /// The completion text
    pub text: String,
    /// Display text (may include additional info)
    pub display: String,
    /// Type of completion
    pub kind: CompletionKind,
}

/// Type of completion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionKind {
    /// File or directory path
    Path,
    /// Command/executable
    Command,
    /// Environment variable
    Environment,
    /// History entry
    History,
    /// Custom completion
    Custom(String),
}

/// Completer trait for providing completions
pub trait Completer {
    /// Get completions for the given input at the cursor position
    fn complete(&self, input: &str, cursor_pos: usize) -> Vec<Completion>;
}

/// Path completer
pub struct PathCompleter {
    /// Base directory for relative paths
    base_dir: PathBuf,
}

impl PathCompleter {
    /// Create a new path completer
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Set the base directory
    pub fn set_base_dir(&mut self, base_dir: PathBuf) {
        self.base_dir = base_dir;
    }
}

impl Completer for PathCompleter {
    fn complete(&self, input: &str, _cursor_pos: usize) -> Vec<Completion> {
        let mut completions = Vec::new();

        // Extract the path portion to complete
        let path_to_complete = if input.contains(' ') {
            input.rsplit(' ').next().unwrap_or("")
        } else {
            input
        };

        let (dir_path, prefix): (PathBuf, String) =
            if path_to_complete.contains('/') || path_to_complete.contains('\\') {
                let path = PathBuf::from(path_to_complete);
                if path_to_complete.ends_with('/') || path_to_complete.ends_with('\\') {
                    (path, String::new())
                } else {
                    let parent = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    (parent, file_name)
                }
            } else {
                (PathBuf::new(), path_to_complete.to_string())
            };

        let search_dir = if dir_path.is_absolute() {
            dir_path
        } else {
            self.base_dir.join(&dir_path)
        };

        if let Ok(entries) = std::fs::read_dir(&search_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();

                if name.starts_with(&prefix) {
                    let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let display = if is_dir {
                        format!("{}/", name)
                    } else {
                        name.to_string()
                    };

                    completions.push(Completion {
                        text: name.to_string(),
                        display,
                        kind: CompletionKind::Path,
                    });
                }
            }
        }

        // Sort completions: directories first, then alphabetically
        completions.sort_by(|a, b| {
            let a_is_dir = a.display.ends_with('/');
            let b_is_dir = b.display.ends_with('/');
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.text.cmp(&b.text),
            }
        });

        completions
    }
}

/// Command completer (searches PATH for executables)
pub struct CommandCompleter {
    /// Cached list of commands
    commands: Vec<String>,
}

impl CommandCompleter {
    /// Create a new command completer
    pub fn new() -> Self {
        Self {
            commands: Self::scan_path(),
        }
    }

    /// Scan PATH for executables
    fn scan_path() -> Vec<String> {
        let mut commands = Vec::new();

        if let Ok(path_var) = std::env::var("PATH") {
            #[cfg(windows)]
            let separator = ';';
            #[cfg(unix)]
            let separator = ':';

            for dir in path_var.split(separator) {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        if let Some(name) = entry.file_name().to_str() {
                            #[cfg(windows)]
                            {
                                // On Windows, check for common executable extensions
                                let lower = name.to_lowercase();
                                if lower.ends_with(".exe")
                                    || lower.ends_with(".cmd")
                                    || lower.ends_with(".bat")
                                {
                                    commands.push(name.to_string());
                                }
                            }
                            #[cfg(unix)]
                            {
                                // On Unix, check if file is executable
                                use std::os::unix::fs::PermissionsExt;
                                if let Ok(metadata) = entry.metadata() {
                                    if metadata.permissions().mode() & 0o111 != 0 {
                                        commands.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        commands.sort();
        commands.dedup();
        commands
    }

    /// Refresh the command cache
    pub fn refresh(&mut self) {
        self.commands = Self::scan_path();
    }
}

impl Default for CommandCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for CommandCompleter {
    fn complete(&self, input: &str, _cursor_pos: usize) -> Vec<Completion> {
        // Only complete the first word (command name)
        if input.contains(' ') {
            return vec![];
        }

        self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(input))
            .map(|cmd| Completion {
                text: cmd.clone(),
                display: cmd.clone(),
                kind: CompletionKind::Command,
            })
            .collect()
    }
}

/// Combined completer that uses multiple completers
pub struct CombinedCompleter {
    path_completer: PathCompleter,
    command_completer: CommandCompleter,
}

impl CombinedCompleter {
    /// Create a new combined completer
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            path_completer: PathCompleter::new(base_dir),
            command_completer: CommandCompleter::new(),
        }
    }

    /// Update the base directory for path completion
    pub fn set_base_dir(&mut self, base_dir: PathBuf) {
        self.path_completer.set_base_dir(base_dir);
    }
}

impl Completer for CombinedCompleter {
    fn complete(&self, input: &str, cursor_pos: usize) -> Vec<Completion> {
        let mut completions = Vec::new();

        // If input contains space, complete paths
        if input.contains(' ') {
            completions.extend(self.path_completer.complete(input, cursor_pos));
        } else {
            // Otherwise, complete commands first, then paths
            completions.extend(self.command_completer.complete(input, cursor_pos));
            completions.extend(self.path_completer.complete(input, cursor_pos));
        }

        completions
    }
}
