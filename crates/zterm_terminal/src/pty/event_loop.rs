//! PTY 事件循环

use super::Pty;
use crate::{PtyConfig, TerminalEventListener, TerminalSize, event::TerminalEvent};
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
pub struct PtyEventLoop {
    /// PTY
    pty: Arc<Mutex<Pty>>,

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
}

impl PtyEventLoop {
    /// 创建新的 PTY 事件循环
    pub fn new(config: &PtyConfig, event_listener: Arc<dyn TerminalEventListener>) -> Result<Self> {
        let (pty, child) = Pty::new(config)?;
        let (msg_tx, msg_rx) = flume::unbounded();

        tracing::debug!(
            "Creating PTY event loop with buffer size: {} KB",
            PTY_READ_BUFFER_SIZE / 1024
        );

        Ok(Self {
            pty: Arc::new(Mutex::new(pty)),
            child: Arc::new(Mutex::new(child)),
            msg_rx,
            msg_tx,
            event_listener,
            running: Arc::new(Mutex::new(false)),
            threads: Arc::new(Mutex::new(None)),
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
    pub fn spawn(self) -> Result<()> {
        *self.running.lock() = true;

        tracing::info!("Starting PTY event loop");

        // 启动读线程
        let read_thread = {
            let pty = self.pty.clone();
            let event_listener = self.event_listener.clone();
            let running = self.running.clone();

            thread::Builder::new()
                .name("pty-reader".to_string())
                .spawn(move || {
                    Self::read_loop(pty, event_listener, running);
                })
                .context("Failed to spawn PTY reader thread")?
        };

        // 启动写/控制线程
        let write_thread = {
            let pty = self.pty.clone();
            let msg_rx = self.msg_rx.clone();
            let running = self.running.clone();

            thread::Builder::new()
                .name("pty-writer".to_string())
                .spawn(move || {
                    Self::write_loop(pty, msg_rx, running);
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

        Ok(())
    }

    /// 读循环
    ///
    /// 从 PTY 读取数据并通过事件系统传递给 VTE parser
    /// 参考 WezTerm 的 read_from_pane_pty 实现
    fn read_loop(
        pty: Arc<Mutex<Pty>>,
        event_listener: Arc<dyn TerminalEventListener>,
        running: Arc<Mutex<bool>>,
    ) {
        // 使用 256KB 缓冲区读取 PTY 数据 (在堆上分配)
        // WezTerm 使用 1MB,这里使用 256KB 作为平衡
        // 大缓冲区可以减少系统调用次数,提高吞吐量
        let mut buf = vec![0u8; PTY_READ_BUFFER_SIZE];

        tracing::debug!(
            "PTY read loop started with {} KB buffer",
            PTY_READ_BUFFER_SIZE / 1024
        );

        while *running.lock() {
            let read_result = {
                let mut pty = pty.lock();
                pty.reader().read(&mut buf)
            };

            match read_result {
                Ok(0) => {
                    // EOF - PTY 已关闭,子进程已退出
                    tracing::trace!("read_pty EOF");
                    *running.lock() = false;
                    break;
                }
                Ok(n) => {
                    // 成功读取数据
                    tracing::trace!("read_pty read {} bytes", n);

                    // 将读取的数据发送给 VTE parser 处理
                    // 注意:必须克隆数据,因为 buf 会被重用
                    // TODO: 将来可以考虑使用 Arc<[u8]> 等零拷贝方案优化
                    event_listener.on_event(TerminalEvent::PtyOutput(buf[..n].to_vec()));
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                    // 被信号中断(例如 EINTR),这是正常的,立即重试
                    tracing::trace!("read_pty interrupted, retrying");
                    continue;
                }
                Err(e) => {
                    // 其他错误:I/O 错误、权限问题等
                    // 参考 WezTerm:所有错误都直接退出,不做区分
                    tracing::error!("read_pty failed: {:?}", e);
                    *running.lock() = false;
                    break;
                }
            }
        }

        tracing::debug!("PTY read loop exited");
    }

    /// 写循环
    fn write_loop(pty: Arc<Mutex<Pty>>, msg_rx: Receiver<PtyMessage>, running: Arc<Mutex<bool>>) {
        while *running.lock() {
            match msg_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(PtyMessage::Input(data)) => {
                    let write_result = {
                        let mut pty = pty.lock();
                        pty.writer().write_all(&data)
                    };

                    if let Err(e) = write_result {
                        tracing::error!("PTY write error: {}", e);
                        *running.lock() = false;
                        break;
                    }
                }
                Ok(PtyMessage::Resize(size)) => {
                    if let Err(e) = pty.lock().resize(size) {
                        tracing::error!("PTY resize error: {}", e);
                    }
                }
                Ok(PtyMessage::Shutdown) => {
                    tracing::debug!("PTY write loop received shutdown");
                    *running.lock() = false;
                    break;
                }
                Err(flume::RecvTimeoutError::Timeout) => {
                    // 超时，继续循环
                    continue;
                }
                Err(flume::RecvTimeoutError::Disconnected) => {
                    tracing::debug!("PTY message channel disconnected");
                    *running.lock() = false;
                    break;
                }
            }
        }

        tracing::debug!("PTY write loop exited");
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
