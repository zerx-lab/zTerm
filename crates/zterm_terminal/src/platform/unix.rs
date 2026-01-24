//! Unix PTY implementation

use super::{PtyConfig, detect_shell};
use flume::Sender;
use parking_lot::Mutex;
use portable_pty::{CommandBuilder, PtyPair, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;
use tracing::{debug, error};
use zterm_common::{Error, Result};

/// Unix PTY handle
pub struct UnixPty {
    pair: Arc<Mutex<PtyPair>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl UnixPty {
    /// Create a new Unix PTY
    pub fn new(config: PtyConfig, output_tx: Sender<Vec<u8>>) -> Result<Self> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows: config.rows,
            cols: config.cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system
            .openpty(size)
            .map_err(|e| Error::pty(format!("Failed to open PTY: {}", e)))?;

        // Determine shell to use
        let shell = config.shell.unwrap_or_else(detect_shell);
        debug!("Using shell: {}", shell);

        // Build command
        let mut cmd = CommandBuilder::new(&shell);

        // Set working directory
        if let Some(ref cwd) = config.working_directory {
            cmd.cwd(cwd);
        }

        // Add environment variables
        for (key, value) in config.env {
            cmd.env(key, value);
        }

        // Spawn the shell
        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| Error::pty(format!("Failed to spawn shell: {}", e)))?;

        // Get writer
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| Error::pty(format!("Failed to get PTY writer: {}", e)))?;

        // Get reader and spawn reader thread
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| Error::pty(format!("Failed to get PTY reader: {}", e)))?;

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        debug!("PTY EOF");
                        break;
                    }
                    Ok(n) => {
                        if output_tx.send(buf[..n].to_vec()).is_err() {
                            debug!("Output channel closed");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("PTY read error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            pair: Arc::new(Mutex::new(pair)),
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    /// Write data to the PTY
    pub fn write(&self, data: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock();
        writer
            .write_all(data)
            .map_err(|e| Error::pty(format!("Failed to write to PTY: {}", e)))?;
        writer
            .flush()
            .map_err(|e| Error::pty(format!("Failed to flush PTY: {}", e)))?;
        Ok(())
    }

    /// Resize the PTY
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let pair = self.pair.lock();
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        pair.master
            .resize(size)
            .map_err(|e| Error::pty(format!("Failed to resize PTY: {}", e)))?;
        Ok(())
    }
}
