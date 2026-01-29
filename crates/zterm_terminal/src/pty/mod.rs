//! PTY 管理模块

mod event_loop;

pub use event_loop::{PtyEventLoop, PtyMessage};

use crate::{PtyConfig, TerminalSize};
use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, MasterPty};
use std::io::{Read, Write};

/// PTY Reader (分离出来避免死锁)
pub struct PtyReader {
    reader: Box<dyn Read + Send>,
}

impl PtyReader {
    /// 读取数据
    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

/// PTY Writer (分离出来避免死锁)
pub struct PtyWriter {
    writer: Box<dyn Write + Send>,
}

impl PtyWriter {
    /// 写入数据
    pub fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)
    }

    /// 刷新缓冲区
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

/// PTY Master (用于 resize 等操作)
pub struct PtyMaster {
    master: Box<dyn MasterPty + Send>,
}

impl PtyMaster {
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

/// PTY 包装器 (旧接口，保持兼容)
pub struct Pty {
    master: Box<dyn MasterPty + Send>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl Pty {
    /// 创建新的 PTY
    pub fn new(config: &PtyConfig) -> Result<(Self, Box<dyn portable_pty::Child + Send + Sync>)> {
        tracing::info!("Creating PTY with shell: {:?}", config.get_shell());

        let pty_system = portable_pty::native_pty_system();

        // 创建 PTY pair
        let pair = pty_system
            .openpty(config.initial_size.into())
            .context("Failed to open PTY")?;
        tracing::debug!("PTY pair created successfully");

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

        tracing::info!("Spawning shell: {:?} with args: {:?}", config.get_shell(), config.shell_args);

        // 启动子进程
        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn shell process")?;

        tracing::info!("Shell process spawned successfully");

        // 关键步骤: 在Windows上必须drop slave,否则ConPTY的管道读取会阻塞
        // 参考: WezTerm的pty/examples/bash.rs
        // slave端持有的管道句柄需要被关闭,master端的读取才能正常工作
        drop(pair.slave);

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

/// 创建分离的 PTY 组件 (避免死锁)
///
/// 返回 (PtyReader, PtyWriter, PtyMaster, Child)
/// Reader 和 Writer 分别在各自的线程中使用，不共享锁
pub fn create_pty_components(
    config: &PtyConfig,
) -> Result<(
    PtyReader,
    PtyWriter,
    PtyMaster,
    Box<dyn portable_pty::Child + Send + Sync>,
)> {
    tracing::info!("Creating PTY components with shell: {:?}", config.get_shell());

    let pty_system = portable_pty::native_pty_system();

    // 创建 PTY pair
    let pair = pty_system
        .openpty(config.initial_size.into())
        .context("Failed to open PTY")?;
    tracing::debug!("PTY pair created successfully");

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

    tracing::info!(
        "Spawning shell: {:?} with args: {:?}",
        config.get_shell(),
        config.shell_args
    );

    // 启动子进程
    let child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn shell process")?;

    tracing::info!("Shell process spawned successfully");

    // 关键步骤: 在Windows上必须drop slave,否则ConPTY的管道读取会阻塞
    drop(pair.slave);

    // 创建分离的 reader 和 writer
    let reader = pair
        .master
        .try_clone_reader()
        .context("Failed to clone PTY reader")?;

    let writer = pair
        .master
        .take_writer()
        .context("Failed to take PTY writer")?;

    Ok((
        PtyReader { reader },
        PtyWriter { writer },
        PtyMaster {
            master: pair.master,
        },
        child,
    ))
}
