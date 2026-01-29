//! # zterm_terminal
//!
//! 终端后端核心库，提供：
//! - PTY 管理（基于 portable-pty）
//! - VTE 解析（基于 vte）
//! - Terminal State 管理
//! - Shell Integration（OSC 133/633）
//! - Event 系统

// 模块声明
pub mod config;
pub mod event;
pub mod grid;
pub mod pty;
pub mod terminal;
pub mod vte_performer;

#[cfg(feature = "shell-integration")]
pub mod shell_integration;

// 公开 API
pub use config::{PtyConfig, TerminalConfig};
pub use event::{TerminalEvent, TerminalEventListener};
pub use grid::{Cell, CellAttributes, Color, CursorShape, PhysRowIndex, VisibleRowIndex};
pub use terminal::{CursorInfo, Terminal};

/// 终端尺寸
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TerminalSize {
    /// 行数
    pub rows: u16,
    /// 列数
    pub cols: u16,
    /// 像素宽度（可选）
    pub pixel_width: u16,
    /// 像素高度（可选）
    pub pixel_height: u16,
}

impl TerminalSize {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

impl From<TerminalSize> for portable_pty::PtySize {
    fn from(size: TerminalSize) -> Self {
        portable_pty::PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: size.pixel_width,
            pixel_height: size.pixel_height,
        }
    }
}

impl From<portable_pty::PtySize> for TerminalSize {
    fn from(size: portable_pty::PtySize) -> Self {
        Self {
            rows: size.rows,
            cols: size.cols,
            pixel_width: size.pixel_width,
            pixel_height: size.pixel_height,
        }
    }
}

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
