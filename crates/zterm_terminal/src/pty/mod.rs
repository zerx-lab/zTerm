//! PTY 管理模块

mod event_loop;

pub use event_loop::{PtyEventLoop, PtyMessage};

use crate::{PtyConfig, TerminalSize};
use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, MasterPty};
use std::io::{Read, Write};

/// PTY 包装器
pub struct Pty {
    master: Box<dyn MasterPty + Send>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl Pty {
    /// 创建新的 PTY
    pub fn new(config: &PtyConfig) -> Result<(Self, Box<dyn portable_pty::Child + Send + Sync>)> {
        let pty_system = portable_pty::native_pty_system();

        // 创建 PTY pair
        let pair = pty_system
            .openpty(config.initial_size.into())
            .context("Failed to open PTY")?;

        // 构建命令
        let mut cmd = CommandBuilder::new(config.get_shell());
        cmd.args(&config.shell_args);

        if let Some(cwd) = &config.working_directory {
            cmd.cwd(cwd);
        }

        // 设置环境变量
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // 启动子进程
        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn shell process")?;

        // 创建 reader 和 writer
        let reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;

        let writer = pair
            .master
            .take_writer()
            .context("Failed to take PTY writer")?;

        Ok((
            Self {
                master: pair.master,
                reader,
                writer,
            },
            child,
        ))
    }

    /// 获取 reader
    pub fn reader(&mut self) -> &mut dyn Read {
        &mut self.reader
    }

    /// 获取 writer
    pub fn writer(&mut self) -> &mut dyn Write {
        &mut self.writer
    }

    /// 调整终端尺寸
    pub fn resize(&self, size: TerminalSize) -> Result<()> {
        self.master
            .resize(size.into())
            .context("Failed to resize PTY")
    }

    /// 获取当前尺寸
    pub fn get_size(&self) -> Result<TerminalSize> {
        self.master
            .get_size()
            .map(Into::into)
            .context("Failed to get PTY size")
    }
}
