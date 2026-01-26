//! 终端状态管理

use crate::{
    event::{ChannelEventListener, TerminalEvent},
    pty::{PtyEventLoop, PtyMessage},
    vte_bridge::VtePerformer,
    PtyConfig, TerminalConfig, TerminalSize,
};
use anyhow::{Context, Result};
use flume::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;
use vte::Parser;

/// 终端实例
pub struct Terminal {
    /// PTY 事件循环消息发送器
    pty_tx: Sender<PtyMessage>,

    /// 终端事件接收器
    event_rx: Receiver<TerminalEvent>,

    /// VTE 解析器
    parser: Arc<Mutex<Parser>>,

    /// VTE Performer
    performer: Arc<Mutex<VtePerformer>>,

    /// 配置
    config: TerminalConfig,

    /// 当前尺寸
    size: Arc<Mutex<TerminalSize>>,
}

impl Terminal {
    /// 创建新的终端实例
    pub fn new(pty_config: PtyConfig, term_config: TerminalConfig) -> Result<Self> {
        // 创建事件通道
        let (event_tx, event_rx) = flume::unbounded();
        let event_listener = Arc::new(ChannelEventListener::new(event_tx));

        // 创建 PTY 事件循环
        let pty_loop =
            PtyEventLoop::new(&pty_config, event_listener.clone()).context("创建 PTY 失败")?;

        let pty_tx = pty_loop.sender();

        // 启动 PTY 事件循环
        pty_loop.spawn().context("启动 PTY 事件循环失败")?;

        // 创建 VTE 解析器
        let parser = Arc::new(Mutex::new(Parser::new()));
        let performer = Arc::new(Mutex::new(VtePerformer::new(event_listener)));

        Ok(Self {
            pty_tx,
            event_rx,
            parser,
            performer,
            config: term_config,
            size: Arc::new(Mutex::new(pty_config.initial_size)),
        })
    }

    /// 写入数据到 PTY
    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.pty_tx
            .send(PtyMessage::Input(data.to_vec()))
            .context("发送输入失败")
    }

    /// 调整终端尺寸
    pub fn resize(&self, size: TerminalSize) -> Result<()> {
        *self.size.lock() = size;
        self.pty_tx
            .send(PtyMessage::Resize(size))
            .context("发送 resize 消息失败")
    }

    /// 获取当前尺寸
    pub fn size(&self) -> TerminalSize {
        *self.size.lock()
    }

    /// 获取事件接收器
    pub fn event_receiver(&self) -> Receiver<TerminalEvent> {
        self.event_rx.clone()
    }

    /// 处理从 PTY 读取的数据
    pub fn process_pty_data(&self, data: &[u8]) {
        let mut parser = self.parser.lock();
        let mut performer = self.performer.lock();

        parser.advance(&mut *performer, data);
    }

    /// 关闭终端
    pub fn shutdown(&self) -> Result<()> {
        self.pty_tx
            .send(PtyMessage::Shutdown)
            .context("发送关闭消息失败")
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let pty_config = PtyConfig::default();
        let term_config = TerminalConfig::default();

        let terminal = Terminal::new(pty_config, term_config);
        assert!(terminal.is_ok());
    }

    #[test]
    fn test_terminal_resize() {
        let pty_config = PtyConfig::default();
        let term_config = TerminalConfig::default();

        let terminal = Terminal::new(pty_config, term_config).unwrap();

        let new_size = TerminalSize::new(40, 120);
        assert!(terminal.resize(new_size).is_ok());
        assert_eq!(terminal.size(), new_size);
    }
}
