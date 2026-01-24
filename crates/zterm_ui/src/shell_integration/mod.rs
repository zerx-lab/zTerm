//! Shell integration UI module for zTerm
//!
//! This module provides UI components and utilities for shell integration,
//! including:
//! - Mouse interaction handling for command zones
//! - Gutter rendering for command status marks
//! - Zone highlighting
//! - Context menu building
//! - AI context extraction
//!
//! # Overview
//!
//! The shell integration UI builds on top of the core shell integration
//! in `zterm_terminal` to provide visual feedback and interaction
//! capabilities.
//!
//! # Usage
//!
//! ```ignore
//! use zterm_ui::shell_integration::{
//!     MouseHandler, MouseConfig, HoverState,
//!     GutterConfig, GutterVisual,
//!     HighlightConfig, HighlightRegion, HighlightType,
//!     build_context_menu, MenuContext,
//!     AiCommandContext, AiIntent,
//! };
//!
//! // Mouse handling
//! let handler = MouseHandler::new(MouseConfig::default());
//! let hover_state = HoverState::from_position(&handler, pos, |line| {
//!     zone_manager.zone_at_line(line).map(|z| (z.start_line, line > z.start_line))
//! });
//!
//! // Gutter rendering
//! let visual = gutter::command_state_to_visual(is_prompt, is_running, exit_code);
//!
//! // Context menu
//! let context = MenuContext::new()
//!     .with_command("ls -la")
//!     .with_output(true)
//!     .with_ai(true);
//! let menu = build_context_menu(&context);
//!
//! // AI context
//! let ai_ctx = AiCommandContext::new()
//!     .with_command("ls -la")
//!     .with_exit_code(0);
//! let prompt = ai_ctx.to_ai_prompt(AiIntent::ExplainCommand);
//! ```

pub mod ai_context;
pub mod context_menu;
pub mod gutter;
pub mod highlight;
pub mod mouse;

pub use ai_context::{AiCommandContext, AiIntent, AiSessionContext, AiTerminalContext};
pub use context_menu::{
    build_context_menu, count_enabled_items, ContextMenuAction, MenuContext, MenuEntry, MenuItem,
};
pub use gutter::{command_state_to_visual, GutterConfig, GutterIcon, GutterMark, GutterVisual};
pub use highlight::{HighlightConfig, HighlightRect, HighlightRegion, HighlightType};
pub use mouse::{HoverState, MouseConfig, MouseHandler};
