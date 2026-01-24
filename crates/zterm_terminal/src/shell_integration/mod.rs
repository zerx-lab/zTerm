//! Shell integration module for zTerm
//!
//! This module provides shell integration features including:
//! - OSC 133 (FinalTerm) and OSC 633 (VS Code) sequence handling
//! - Command zone tracking and management
//! - Text extraction for AI context
//!
//! # Overview
//!
//! Shell integration allows the terminal to understand the structure of
//! shell output, including:
//! - Where prompts begin and end
//! - What commands were executed
//! - The output of each command
//! - Exit codes of commands
//!
//! This enables features like:
//! - Clicking on command output to select it
//! - Jumping between prompts
//! - Copying just the command or just the output
//! - Providing context to AI assistants
//!
//! # Usage
//!
//! ```ignore
//! use zterm_terminal::shell_integration::{ShellIntegrationHandler, ShellEvent};
//!
//! let mut handler = ShellIntegrationHandler::new();
//!
//! // Process OSC sequences
//! handler.handle_osc(b"133;A");  // Prompt started
//! handler.handle_osc(b"133;B");  // Command input
//! handler.handle_osc(b"133;C");  // Command executing
//! handler.handle_osc(b"133;D;0"); // Command finished
//!
//! // Get events
//! for event in handler.take_events() {
//!     match event {
//!         ShellEvent::CommandFinished { exit_code, .. } => {
//!             println!("Command finished with exit code: {}", exit_code);
//!         }
//!         _ => {}
//!     }
//! }
//! ```

mod event;
mod extractor;
mod handler;
mod inject;
mod scanner;
mod zone;

pub use event::ShellEvent;
pub use extractor::{ContextSummary, OutputSummary, TextBuffer, TextExtractor};
pub use handler::ShellIntegrationHandler;
pub use inject::{
    get_integration_script, get_integration_script_path, get_shell_args_with_integration,
    supports_integration, POWERSHELL_INTEGRATION,
};
pub use scanner::{OscScanner, OscSequence};
pub use zone::{CommandState, CommandZone, ZoneId, ZoneManager};
