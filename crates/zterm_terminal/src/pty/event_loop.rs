//! PTY 事件循环

use super::{create_pty_components, PtyMaster, PtyReader, PtyWriter};
use crate::{event::TerminalEvent, PtyConfig, TerminalEventListener, TerminalSize};
use anyhow::{Context, Result};
use flume::{Receiver, Sender};
use parking_lot::Mutex;
use std::io;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// PTY 读取缓冲区大小
/// 参考 WezTerm 使用 1MB,这里使用 512KB 作为平衡选择
const PTY_READ_BUFFER_SIZE: usize = 512 * 1024;

/// PTY 事件循环消息
#[derive(Debug)]
pub enum PtyMessage {
    /// 写入数据到 PTY
    Input(Vec<u8>),

    /// 调整终端尺寸
    Resize(TerminalSize),

    /// 关闭 PTY
    Shutdown,
}

/// PTY 事件循环
///
/// 关键设计：Reader 和 Writer 分离，避免死锁
/// - Reader 在读取线程中独占使用，不需要锁
/// - Writer 有自己的 Mutex，只在写入线程中使用
/// - Master 有自己的 Mutex，用于 resize 操作
pub struct PtyEventLoop {
    /// PTY Writer (用 Mutex 包装因为需要在写入线程中使用)
    writer: Arc<Mutex<PtyWriter>>,

    /// PTY Master (用于 resize)
    master: Arc<Mutex<PtyMaster>>,

    /// 子进程
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,

    /// 消息接收器
    msg_rx: Receiver<PtyMessage>,

    /// 消息发送器
    msg_tx: Sender<PtyMessage>,

    /// 事件监听器
    event_listener: Arc<dyn TerminalEventListener>,

    /// 是否运行中
    running: Arc<Mutex<bool>>,

    /// 线程句柄 (保存以便优雅关闭)
    threads: Arc<Mutex<Option<(JoinHandle<()>, JoinHandle<()>, JoinHandle<()>)>>>,

    /// Reader (Option 因为会被移动到读取线程)
    reader: Option<PtyReader>,
}

impl PtyEventLoop {
    /// 创建新的 PTY 事件循环
    pub fn new(config: &PtyConfig, event_listener: Arc<dyn TerminalEventListener>) -> Result<Self> {
        // 使用分离的组件创建 PTY，避免死锁
        let (reader, writer, master, child) = create_pty_components(config)?;
        let (msg_tx, msg_rx) = flume::unbounded();

        tracing::info!(
            "Creating PTY event loop with buffer size: {} KB (using separated reader/writer)",
            PTY_READ_BUFFER_SIZE / 1024
        );

        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
            master: Arc::new(Mutex::new(master)),
            child: Arc::new(Mutex::new(child)),
            msg_rx,
            msg_tx,
            event_listener,
            running: Arc::new(Mutex::new(false)),
            threads: Arc::new(Mutex::new(None)),
            reader: Some(reader),
        })
    }

    /// 获取消息发送器
    pub fn sender(&self) -> Sender<PtyMessage> {
        self.msg_tx.clone()
    }

    /// 优雅关闭 PTY 事件循环
    ///
    /// 设置 running 标志为 false,并等待所有线程完成
    pub fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down PTY event loop");

        // 设置运行标志为 false
        *self.running.lock() = false;

        // 发送 shutdown 消息以唤醒 write_loop
        let _ = self.msg_tx.send(PtyMessage::Shutdown);

        // 等待所有线程完成
        if let Some((read_thread, write_thread, child_monitor)) = self.threads.lock().take() {
            tracing::debug!("Waiting for PTY threads to finish");

            if let Err(e) = read_thread.join() {
                tracing::error!("PTY reader thread panicked: {:?}", e);
            }

            if let Err(e) = write_thread.join() {
                tracing::error!("PTY writer thread panicked: {:?}", e);
            }

            if let Err(e) = child_monitor.join() {
                tracing::error!("PTY child monitor thread panicked: {:?}", e);
            }

            tracing::info!("All PTY threads stopped");
        }

        Ok(())
    }

    /// 启动事件循环
    /// 返回 self 以保持生命周期,防止被 drop 导致线程退出
    pub fn spawn(mut self) -> Result<Self> {
        *self.running.lock() = true;

        tracing::info!("Starting PTY event loop (with separated reader/writer)");

        // 取出 reader (移动到读取线程)
        let reader = self
            .reader
            .take()
            .expect("Reader should exist before spawn");

        // 启动读线程 (reader 被移动到这个线程，不需要锁)
        let read_thread = {
            let event_listener = self.event_listener.clone();
            let running = self.running.clone();

            thread::Builder::new()
                .name("pty-reader".to_string())
                .spawn(move || {
                    Self::read_loop(reader, event_listener, running);
                })
                .context("Failed to spawn PTY reader thread")?
        };

        // 启动写/控制线程
        let write_thread = {
            let writer = self.writer.clone();
            let master = self.master.clone();
            let msg_rx = self.msg_rx.clone();
            let running = self.running.clone();

            thread::Builder::new()
                .name("pty-writer".to_string())
                .spawn(move || {
                    Self::write_loop(writer, master, msg_rx, running);
                })
                .context("Failed to spawn PTY writer thread")?
        };

        // 启动子进程监控线程
        let child_monitor = {
            let child = self.child.clone();
            let event_listener = self.event_listener.clone();
            let running = self.running.clone();

            thread::Builder::new()
                .name("pty-child-monitor".to_string())
                .spawn(move || {
                    Self::child_monitor_loop(child, event_listener, running);
                })
                .context("Failed to spawn child monitor thread")?
        };

        // 保存线程句柄以便后续 join
        *self.threads.lock() = Some((read_thread, write_thread, child_monitor));

        tracing::info!("PTY event loop started successfully");

        Ok(self)
    }

    /// 读循环
    ///
    /// 从 PTY 读取数据并通过事件系统传递给 VTE parser
    /// 关键：reader 被移动到此线程，不与 writer 共享锁
    fn read_loop(
        mut reader: PtyReader,
        event_listener: Arc<dyn TerminalEventListener>,
        running: Arc<Mutex<bool>>,
    ) {
        // 使用 256KB 缓冲区读取 PTY 数据 (在堆上分配)
        let mut buf = vec![0u8; PTY_READ_BUFFER_SIZE];

        tracing::info!(
            "[ReadLoop] Started with {} KB buffer (no lock contention with writer)",
            PTY_READ_BUFFER_SIZE / 1024
        );

        let mut read_count = 0;

        loop {
            // 检查运行状态
            let is_running = *running.lock();
            if !is_running {
                tracing::info!("[ReadLoop] running flag is false, exiting");
                break;
            }

            tracing::debug!(
                "[ReadLoop] Iteration {}: calling read() (no lock needed)...",
                read_count + 1
            );

            // 直接调用 read，不需要获取任何锁！
            let read_result = reader.read(&mut buf);

            match read_result {
                Ok(0) => {
                    // EOF - PTY 已关闭,子进程已退出
                    tracing::warn!("[ReadLoop] EOF after {} successful reads", read_count);
                    *running.lock() = false;
                    break;
                }
                Ok(n) => {
                    // 成功读取数据
                    read_count += 1;
                    tracing::info!("[ReadLoop] Read #{}: received {} bytes", read_count, n);

                    // 显示前200字节的内容（用于调试）
                    let preview = String::from_utf8_lossy(&buf[..n.min(200)]);
                    tracing::debug!("[ReadLoop] Data preview: {:?}", preview);

                    // 将读取的数据发送给 VTE parser 处理
                    event_listener.on_event(TerminalEvent::PtyOutput(buf[..n].to_vec()));
                    tracing::debug!("[ReadLoop] Event sent");
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                    // 被信号中断(例如 EINTR),这是正常的,立即重试
                    tracing::debug!("[ReadLoop] Interrupted, retrying");
                    continue;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // 非阻塞模式下没有数据可读
                    tracing::trace!("[ReadLoop] WouldBlock, sleeping briefly");
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    // 其他错误:I/O 错误、权限问题等
                    tracing::error!("[ReadLoop] Error: {:?} (kind: {:?})", e, e.kind());
                    *running.lock() = false;
                    break;
                }
            }
        }

        tracing::info!("[ReadLoop] Exited after {} reads", read_count);
    }

    /// 写循环
    ///
    /// 关键：writer 有自己的锁，不与 reader 冲突
    fn write_loop(
        writer: Arc<Mutex<PtyWriter>>,
        master: Arc<Mutex<PtyMaster>>,
        msg_rx: Receiver<PtyMessage>,
        running: Arc<Mutex<bool>>,
    ) {
        tracing::info!("[WriteLoop] Started (has dedicated writer lock)");

        loop {
            let is_running = *running.lock();
            if !is_running {
                tracing::info!("[WriteLoop] running flag is false, exiting");
                break;
            }

            match msg_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(PtyMessage::Input(data)) => {
                    tracing::info!("[WriteLoop] Received Input message: {} bytes", data.len());
                    tracing::debug!("[WriteLoop] Data: {:?}", String::from_utf8_lossy(&data));

                    // 获取 writer 锁（不会与 reader 冲突）
                    let write_result = {
                        let mut w = writer.lock();
                        tracing::debug!("[WriteLoop] Writer lock acquired, writing...");
                        let result = w.write_all(&data);
                        if result.is_ok() {
                            let _ = w.flush();
                        }
                        result
                    };

                    match write_result {
                        Ok(()) => {
                            tracing::info!("[WriteLoop] Write successful: {} bytes", data.len());
                        }
                        Err(e) => {
                            tracing::error!("[WriteLoop] Write error: {}", e);
                            *running.lock() = false;
                            break;
                        }
                    }
                }
                Ok(PtyMessage::Resize(size)) => {
                    tracing::info!(
                        "[WriteLoop] Received Resize message: {}x{}",
                        size.cols,
                        size.rows
                    );
                    if let Err(e) = master.lock().resize(size) {
                        tracing::error!("[WriteLoop] Resize error: {}", e);
                    }
                }
                Ok(PtyMessage::Shutdown) => {
                    tracing::info!("[WriteLoop] Received Shutdown message");
                    *running.lock() = false;
                    break;
                }
                Err(flume::RecvTimeoutError::Timeout) => {
                    // 超时，继续循环
                    continue;
                }
                Err(flume::RecvTimeoutError::Disconnected) => {
                    tracing::info!("[WriteLoop] Channel disconnected");
                    *running.lock() = false;
                    break;
                }
            }
        }

        tracing::info!("[WriteLoop] Exited");
    }

    /// 子进程监控循环
    fn child_monitor_loop(
        child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
        event_listener: Arc<dyn TerminalEventListener>,
        running: Arc<Mutex<bool>>,
    ) {
        while *running.lock() {
            let try_wait_result = {
                let mut child = child.lock();
                child.try_wait()
            };

            match try_wait_result {
                Ok(Some(status)) => {
                    tracing::debug!("Child process exited with status: {:?}", status);
                    event_listener.on_event(TerminalEvent::ProcessExit {
                        exit_code: Some(status.exit_code() as i32),
                    });
                    *running.lock() = false;
                    break;
                }
                Ok(None) => {
                    // 子进程仍在运行
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    tracing::error!("Failed to wait for child: {}", e);
                    *running.lock() = false;
                    break;
                }
            }
        }

        tracing::debug!("Child monitor loop exited");
    }
}

impl Drop for PtyEventLoop {
    fn drop(&mut self) {
        // 确保在 drop 时优雅关闭所有线程
        tracing::debug!("PtyEventLoop drop: ensuring threads are stopped");

        *self.running.lock() = false;

        // 发送 shutdown 消息
        let _ = self.msg_tx.send(PtyMessage::Shutdown);

        // 等待线程完成
        // 注意:这里使用较短的超时,避免阻塞太久
        // 如果线程没有响应,它们会在进程退出时被强制终止
        if let Some((read_thread, write_thread, child_monitor)) = self.threads.lock().take() {
            // 给每个线程 100ms 来完成
            // 这对于大多数情况足够了
            std::mem::drop(read_thread);
            std::mem::drop(write_thread);
            std::mem::drop(child_monitor);
        }
    }
}
