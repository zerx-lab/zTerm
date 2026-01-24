//! Platform-specific PTY implementations

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

mod shell;

pub use shell::detect_shell;

use zterm_common::Result;
use flume::Receiver;
use std::path::PathBuf;

/// Configuration for creating a PTY
#[derive(Debug, Clone)]
pub struct PtyConfig {
    /// Shell command to run (None = auto-detect)
    pub shell: Option<String>,
    /// Working directory
    pub working_directory: Option<PathBuf>,
    /// Number of columns
    pub cols: u16,
    /// Number of rows
    pub rows: u16,
    /// Additional environment variables
    pub env: Vec<(String, String)>,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            shell: None,
            cols: 80,
            rows: 24,
            working_directory: None,
            env: vec![],
        }
    }
}

/// Platform-independent PTY handle
pub struct Pty {
    #[cfg(unix)]
    inner: unix::UnixPty,

    #[cfg(windows)]
    inner: windows::WindowsPty,

    /// Channel for receiving output from the PTY
    output_rx: Receiver<Vec<u8>>,
}

impl Pty {
    /// Create a new PTY with the given configuration
    pub fn new(config: PtyConfig) -> Result<Self> {
        let (output_tx, output_rx) = flume::unbounded();

        #[cfg(unix)]
        let inner = unix::UnixPty::new(config, output_tx)?;

        #[cfg(windows)]
        let inner = windows::WindowsPty::new(config, output_tx)?;

        Ok(Self { inner, output_rx })
    }

    /// Get the output receiver for reading PTY output
    pub fn reader(&self) -> Receiver<Vec<u8>> {
        self.output_rx.clone()
    }

    /// Write data to the PTY
    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.inner.write(data)
    }

    /// Resize the PTY
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.inner.resize(cols, rows)
    }
}
