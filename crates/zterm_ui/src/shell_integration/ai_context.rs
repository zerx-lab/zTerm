//! AI context extraction for shell integration
//!
//! This module provides utilities for extracting context from terminal
//! sessions for use with AI assistants.

use zterm_terminal::shell_integration::ContextSummary;

/// Context for a single command suitable for AI consumption
#[derive(Debug, Clone)]
pub struct AiCommandContext {
    /// The command that was executed
    pub command: Option<String>,
    /// Working directory when command ran
    pub working_directory: Option<String>,
    /// Exit code (if finished)
    pub exit_code: Option<i32>,
    /// Command output (possibly truncated)
    pub output: Option<String>,
    /// Whether output was truncated
    pub output_truncated: bool,
    /// Total output line count
    pub output_line_count: usize,
    /// Shell type (bash, zsh, powershell, etc.)
    pub shell_type: Option<String>,
}

impl Default for AiCommandContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AiCommandContext {
    /// Create a new empty AI command context
    pub fn new() -> Self {
        Self {
            command: None,
            working_directory: None,
            exit_code: None,
            output: None,
            output_truncated: false,
            output_line_count: 0,
            shell_type: None,
        }
    }

    /// Create from a ContextSummary
    pub fn from_summary(summary: &ContextSummary) -> Self {
        let (output, output_truncated, output_line_count) = match &summary.output {
            Some(output_summary) => (
                Some(output_summary.text.clone()),
                output_summary.truncated,
                output_summary.line_count,
            ),
            None => (None, false, 0),
        };

        Self {
            command: summary.command.clone(),
            working_directory: summary.working_dir.clone(),
            exit_code: summary.exit_code,
            output,
            output_truncated,
            output_line_count,
            shell_type: None,
        }
    }

    /// Set the command
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set the working directory
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Set the exit code
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Set the output
    pub fn with_output(
        mut self,
        output: impl Into<String>,
        line_count: usize,
        truncated: bool,
    ) -> Self {
        self.output = Some(output.into());
        self.output_line_count = line_count;
        self.output_truncated = truncated;
        self
    }

    /// Set the shell type
    pub fn with_shell_type(mut self, shell: impl Into<String>) -> Self {
        self.shell_type = Some(shell.into());
        self
    }

    /// Check if the command failed
    pub fn is_failure(&self) -> bool {
        matches!(self.exit_code, Some(code) if code != 0)
    }

    /// Check if the command succeeded
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Format as a structured context string for AI
    pub fn to_ai_prompt(&self, intent: AiIntent) -> String {
        let mut parts = Vec::new();

        // Intent-specific preamble
        parts.push(intent.preamble().to_string());
        parts.push(String::new());

        // Command context
        if let Some(cmd) = &self.command {
            parts.push(format!("**Command:** `{}`", cmd));
        }

        if let Some(dir) = &self.working_directory {
            parts.push(format!("**Working Directory:** `{}`", dir));
        }

        if let Some(shell) = &self.shell_type {
            parts.push(format!("**Shell:** {}", shell));
        }

        if let Some(code) = self.exit_code {
            let status = if code == 0 { "Success" } else { "Failed" };
            parts.push(format!("**Exit Code:** {} ({})", code, status));
        }

        // Output section
        if let Some(output) = &self.output {
            parts.push(String::new());
            if self.output_truncated {
                parts.push(format!(
                    "**Output** ({} lines, truncated):",
                    self.output_line_count
                ));
            } else {
                parts.push(format!("**Output** ({} lines):", self.output_line_count));
            }
            parts.push("```".to_string());
            parts.push(output.clone());
            parts.push("```".to_string());
        }

        parts.join("\n")
    }
}

/// Intent for AI context usage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiIntent {
    /// Explain what a command does
    ExplainCommand,
    /// Analyze command output
    AnalyzeOutput,
    /// Debug an error
    DebugError,
    /// Suggest improvements
    SuggestImprovements,
    /// General context
    General,
}

impl AiIntent {
    /// Get the preamble text for this intent
    pub fn preamble(&self) -> &str {
        match self {
            AiIntent::ExplainCommand => "Please explain what this command does and its options:",
            AiIntent::AnalyzeOutput => {
                "Please analyze this command output and summarize the key information:"
            }
            AiIntent::DebugError => {
                "This command failed. Please help me understand and fix the error:"
            }
            AiIntent::SuggestImprovements => {
                "Please suggest improvements or alternatives for this command:"
            }
            AiIntent::General => "Here is a terminal command and its output for context:",
        }
    }
}

/// Trait for terminal contexts that can provide AI context
pub trait AiTerminalContext {
    /// Get context for a specific zone
    fn get_zone_context(&self, zone_start_line: usize) -> Option<AiCommandContext>;

    /// Get context for the most recent command
    fn get_recent_context(&self) -> Option<AiCommandContext>;

    /// Get context for all visible commands
    fn get_visible_context(&self) -> Vec<AiCommandContext>;

    /// Get context for commands with errors
    fn get_error_context(&self) -> Vec<AiCommandContext>;
}

/// A collection of command contexts for multi-command AI analysis
#[derive(Debug, Clone, Default)]
pub struct AiSessionContext {
    /// Individual command contexts
    pub commands: Vec<AiCommandContext>,
    /// Current working directory
    pub current_directory: Option<String>,
    /// Session shell type
    pub shell_type: Option<String>,
    /// Session start time (as a formatted string)
    pub session_started: Option<String>,
}

impl AiSessionContext {
    /// Create a new empty session context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a command context
    pub fn add_command(&mut self, ctx: AiCommandContext) {
        self.commands.push(ctx);
    }

    /// Set the current directory
    pub fn with_current_directory(mut self, dir: impl Into<String>) -> Self {
        self.current_directory = Some(dir.into());
        self
    }

    /// Set the shell type
    pub fn with_shell_type(mut self, shell: impl Into<String>) -> Self {
        self.shell_type = Some(shell.into());
        self
    }

    /// Get the total number of commands
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Get the number of failed commands
    pub fn error_count(&self) -> usize {
        self.commands.iter().filter(|c| c.is_failure()).count()
    }

    /// Get the most recent command
    pub fn most_recent(&self) -> Option<&AiCommandContext> {
        self.commands.last()
    }

    /// Format as a session summary for AI
    pub fn to_session_summary(&self) -> String {
        let mut parts = Vec::new();

        parts.push("# Terminal Session Context".to_string());
        parts.push(String::new());

        if let Some(shell) = &self.shell_type {
            parts.push(format!("**Shell:** {}", shell));
        }

        if let Some(dir) = &self.current_directory {
            parts.push(format!("**Current Directory:** `{}`", dir));
        }

        parts.push(format!(
            "**Commands:** {} total, {} failed",
            self.command_count(),
            self.error_count()
        ));

        parts.push(String::new());
        parts.push("## Recent Commands".to_string());

        for (i, cmd) in self.commands.iter().rev().take(5).enumerate() {
            parts.push(String::new());
            parts.push(format!("### Command {}", i + 1));

            if let Some(command) = &cmd.command {
                parts.push(format!("`{}`", command));
            }

            if let Some(code) = cmd.exit_code {
                let status = if code == 0 { "success" } else { "failed" };
                parts.push(format!("Exit: {} ({})", code, status));
            }
        }

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zterm_terminal::shell_integration::OutputSummary;

    // ===== AiCommandContext Tests =====

    #[test]
    fn test_ai_command_context_default() {
        let ctx = AiCommandContext::default();
        assert!(ctx.command.is_none());
        assert!(ctx.exit_code.is_none());
        assert!(!ctx.output_truncated);
    }

    #[test]
    fn test_ai_command_context_builder() {
        let ctx = AiCommandContext::new()
            .with_command("ls -la")
            .with_working_directory("/home/user")
            .with_exit_code(0)
            .with_output("file1\nfile2", 2, false)
            .with_shell_type("bash");

        assert_eq!(ctx.command.as_deref(), Some("ls -la"));
        assert_eq!(ctx.working_directory.as_deref(), Some("/home/user"));
        assert_eq!(ctx.exit_code, Some(0));
        assert_eq!(ctx.output.as_deref(), Some("file1\nfile2"));
        assert_eq!(ctx.output_line_count, 2);
        assert!(!ctx.output_truncated);
        assert_eq!(ctx.shell_type.as_deref(), Some("bash"));
    }

    #[test]
    fn test_ai_command_context_from_summary() {
        let summary = ContextSummary {
            command: Some("ls".to_string()),
            working_dir: Some("/tmp".to_string()),
            exit_code: Some(0),
            output: Some(OutputSummary {
                text: "file1".to_string(),
                line_count: 1,
                truncated: false,
            }),
        };

        let ctx = AiCommandContext::from_summary(&summary);
        assert_eq!(ctx.command.as_deref(), Some("ls"));
        assert_eq!(ctx.exit_code, Some(0));
    }

    #[test]
    fn test_ai_command_context_is_failure() {
        let ctx = AiCommandContext::new().with_exit_code(1);
        assert!(ctx.is_failure());

        let ctx = AiCommandContext::new().with_exit_code(0);
        assert!(!ctx.is_failure());

        let ctx = AiCommandContext::new();
        assert!(!ctx.is_failure());
    }

    #[test]
    fn test_ai_command_context_is_success() {
        let ctx = AiCommandContext::new().with_exit_code(0);
        assert!(ctx.is_success());

        let ctx = AiCommandContext::new().with_exit_code(1);
        assert!(!ctx.is_success());
    }

    #[test]
    fn test_ai_command_context_to_ai_prompt() {
        let ctx = AiCommandContext::new()
            .with_command("cat file.txt")
            .with_exit_code(0)
            .with_output("content", 1, false);

        let prompt = ctx.to_ai_prompt(AiIntent::ExplainCommand);
        assert!(prompt.contains("explain"));
        assert!(prompt.contains("cat file.txt"));
    }

    #[test]
    fn test_ai_command_context_prompt_truncated() {
        let ctx = AiCommandContext::new()
            .with_command("ls")
            .with_output("...", 1000, true);

        let prompt = ctx.to_ai_prompt(AiIntent::General);
        assert!(prompt.contains("truncated"));
        assert!(prompt.contains("1000 lines"));
    }

    // ===== AiIntent Tests =====

    #[test]
    fn test_ai_intent_preambles() {
        assert!(AiIntent::ExplainCommand.preamble().contains("explain"));
        assert!(AiIntent::AnalyzeOutput.preamble().contains("analyze"));
        assert!(AiIntent::DebugError.preamble().contains("failed"));
        assert!(
            AiIntent::SuggestImprovements
                .preamble()
                .contains("improvements")
        );
    }

    #[test]
    fn test_ai_intent_equality() {
        assert_eq!(AiIntent::ExplainCommand, AiIntent::ExplainCommand);
        assert_ne!(AiIntent::ExplainCommand, AiIntent::AnalyzeOutput);
    }

    // ===== AiSessionContext Tests =====

    #[test]
    fn test_ai_session_context_default() {
        let ctx = AiSessionContext::default();
        assert!(ctx.commands.is_empty());
        assert_eq!(ctx.command_count(), 0);
    }

    #[test]
    fn test_ai_session_context_add_command() {
        let mut session = AiSessionContext::new();
        session.add_command(AiCommandContext::new().with_command("ls"));
        session.add_command(AiCommandContext::new().with_command("pwd"));

        assert_eq!(session.command_count(), 2);
    }

    #[test]
    fn test_ai_session_context_error_count() {
        let mut session = AiSessionContext::new();
        session.add_command(AiCommandContext::new().with_exit_code(0));
        session.add_command(AiCommandContext::new().with_exit_code(1));
        session.add_command(AiCommandContext::new().with_exit_code(127));

        assert_eq!(session.error_count(), 2);
    }

    #[test]
    fn test_ai_session_context_most_recent() {
        let mut session = AiSessionContext::new();
        session.add_command(AiCommandContext::new().with_command("first"));
        session.add_command(AiCommandContext::new().with_command("last"));

        let recent = session.most_recent();
        assert!(recent.is_some());
        assert_eq!(recent.unwrap().command.as_deref(), Some("last"));
    }

    #[test]
    fn test_ai_session_context_most_recent_empty() {
        let session = AiSessionContext::new();
        assert!(session.most_recent().is_none());
    }

    #[test]
    fn test_ai_session_context_to_summary() {
        let mut session = AiSessionContext::new()
            .with_shell_type("bash")
            .with_current_directory("/home");

        session.add_command(AiCommandContext::new().with_command("ls").with_exit_code(0));

        let summary = session.to_session_summary();
        assert!(summary.contains("bash"));
        assert!(summary.contains("/home"));
        assert!(summary.contains("ls"));
    }

    #[test]
    fn test_ai_session_context_builder() {
        let session = AiSessionContext::new()
            .with_shell_type("zsh")
            .with_current_directory("/tmp");

        assert_eq!(session.shell_type.as_deref(), Some("zsh"));
        assert_eq!(session.current_directory.as_deref(), Some("/tmp"));
    }
}
