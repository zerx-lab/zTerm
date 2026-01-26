//! 终端配置

pub mod env_validation;

use crate::TerminalSize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use env_validation::{get_safe_default_env, validate_env_vars, EnvValidationError};

/// PTY 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConfig {
    /// Shell 程序路径（None 则使用系统默认 shell）
    pub shell: Option<PathBuf>,

    /// Shell 参数
    pub shell_args: Vec<String>,

    /// 工作目录（None 则使用当前目录）
    pub working_directory: Option<PathBuf>,

    /// 环境变量
    #[serde(default)]
    pub env: Vec<(String, String)>,

    /// 初始终端尺寸
    pub initial_size: TerminalSize,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            shell: None,
            shell_args: vec![],
            working_directory: None,
            env: vec![],
            initial_size: TerminalSize::new(24, 80),
        }
    }
}

impl PtyConfig {
    /// 获取 shell 程序路径（如果未指定则使用系统默认）
    pub fn get_shell(&self) -> PathBuf {
        if let Some(shell) = &self.shell {
            return shell.clone();
        }

        #[cfg(windows)]
        {
            std::env::var("COMSPEC")
                .unwrap_or_else(|_| "cmd.exe".to_string())
                .into()
        }

        #[cfg(unix)]
        {
            std::env::var("SHELL")
                .unwrap_or_else(|_| "/bin/sh".to_string())
                .into()
        }
    }
}

/// 终端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// 最大滚动回溯行数（0 表示无限制）
    #[serde(default = "default_scrollback_lines")]
    pub scrollback_lines: usize,

    /// 是否启用 shell integration
    #[serde(default = "default_true")]
    pub enable_shell_integration: bool,

    /// 输入批处理间隔（毫秒）
    #[serde(default = "default_input_batch_ms")]
    pub input_batch_ms: u64,

    /// 事件批处理间隔（毫秒）
    #[serde(default = "default_event_batch_ms")]
    pub event_batch_ms: u64,
}

fn default_scrollback_lines() -> usize {
    10000
}

fn default_true() -> bool {
    true
}

fn default_input_batch_ms() -> u64 {
    4
}

fn default_event_batch_ms() -> u64 {
    4
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: default_scrollback_lines(),
            enable_shell_integration: default_true(),
            input_batch_ms: default_input_batch_ms(),
            event_batch_ms: default_event_batch_ms(),
        }
    }
}
