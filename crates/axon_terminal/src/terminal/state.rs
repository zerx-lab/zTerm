//! Terminal state and entity management

use crate::buffer::Grid;
use crate::parser::AnsiParser;
use crate::platform::{Pty, PtyConfig};
use crate::TerminalEvent;
use axon_common::Result;
use gpui::{AsyncApp, Context, EventEmitter, Task, WeakEntity};
use std::path::PathBuf;
use tracing::{debug, error, info};

/// Terminal size in characters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self { cols: 80, rows: 24 }
    }
}

/// The main Terminal entity
pub struct Terminal {
    /// PTY handle
    pty: Option<Pty>,

    /// Screen buffer
    grid: Grid,

    /// ANSI parser
    parser: AnsiParser,

    /// Current size
    size: TerminalSize,

    /// Working directory
    working_directory: PathBuf,

    /// Terminal title
    title: String,

    /// Whether the process has exited
    exited: bool,

    /// Reader task handle
    _reader_task: Option<Task<()>>,
}

impl EventEmitter<TerminalEvent> for Terminal {}

impl Terminal {
    /// Create a new terminal with the given configuration
    pub fn new(
        shell: Option<String>,
        working_directory: Option<PathBuf>,
        size: TerminalSize,
        cx: &mut Context<Self>,
    ) -> Self {
        let working_directory = working_directory
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        let grid = Grid::new(size.cols as usize, size.rows as usize);
        let parser = AnsiParser::new();

        let mut terminal = Self {
            pty: None,
            grid,
            parser,
            size,
            working_directory: working_directory.clone(),
            title: "Axon Terminal".to_string(),
            exited: false,
            _reader_task: None,
        };

        // Spawn the PTY
        if let Err(e) = terminal.spawn_pty(shell, working_directory, cx) {
            error!("Failed to spawn PTY: {}", e);
            cx.emit(TerminalEvent::Error(format!("Failed to spawn PTY: {}", e)));
        }

        terminal
    }

    /// Spawn the PTY process
    fn spawn_pty(
        &mut self,
        shell: Option<String>,
        working_directory: PathBuf,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let config = PtyConfig {
            shell,
            working_directory: Some(working_directory),
            cols: self.size.cols,
            rows: self.size.rows,
            env: vec![],
        };

        let pty = Pty::new(config)?;

        // Set up the reader task
        let reader = pty.reader();
        let reader_task = cx.spawn(async move |this: WeakEntity<Terminal>, cx: &mut AsyncApp| {
            loop {
                match reader.recv_async().await {
                    Ok(data) => {
                        let result = this.update(cx, |terminal, cx| {
                            terminal.process_output(&data, cx);
                        });
                        if result.is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        debug!("PTY reader channel closed");
                        let _ = this.update(cx, |terminal, cx| {
                            terminal.exited = true;
                            cx.emit(TerminalEvent::ProcessExited { exit_code: None });
                        });
                        break;
                    }
                }
            }
        });

        self.pty = Some(pty);
        self._reader_task = Some(reader_task);

        info!("PTY spawned successfully");
        Ok(())
    }

    /// Process output from the PTY
    fn process_output(&mut self, data: &[u8], cx: &mut Context<Self>) {
        debug!("Processing PTY output: {} bytes", data.len());

        // Parse VT sequences and update grid
        self.parser.process(data, &mut self.grid);

        // Emit the output event
        cx.emit(TerminalEvent::Output(data.to_vec()));
        cx.notify();
    }

    /// Write data to the terminal (keyboard input)
    pub fn write(&mut self, data: &[u8]) {
        if let Some(pty) = &self.pty {
            if let Err(e) = pty.write(data) {
                error!("Failed to write to PTY: {}", e);
            }
        }
    }

    /// Write a string to the terminal
    pub fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
    }

    /// Resize the terminal
    pub fn resize(&mut self, size: TerminalSize, cx: &mut Context<Self>) {
        if self.size == size {
            return;
        }

        self.size = size;
        self.grid.resize(size.cols as usize, size.rows as usize);

        if let Some(pty) = &self.pty {
            if let Err(e) = pty.resize(size.cols, size.rows) {
                error!("Failed to resize PTY: {}", e);
            }
        }

        cx.emit(TerminalEvent::Resized {
            cols: size.cols as usize,
            rows: size.rows as usize,
        });
        cx.notify();
    }

    /// Get the current grid
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Get the current size
    pub fn size(&self) -> TerminalSize {
        self.size
    }

    /// Get the terminal title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the terminal title
    pub fn set_title(&mut self, title: String, cx: &mut Context<Self>) {
        self.title = title.clone();
        cx.emit(TerminalEvent::TitleChanged(title));
        cx.notify();
    }

    /// Check if the process has exited
    pub fn has_exited(&self) -> bool {
        self.exited
    }

    /// Get the working directory
    pub fn working_directory(&self) -> &PathBuf {
        &self.working_directory
    }
}
