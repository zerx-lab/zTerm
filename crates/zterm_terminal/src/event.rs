//! 终端事件系统

use std::sync::Arc;

/// 终端事件
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// 终端需要重绘
    Wakeup,

    /// PTY 输出数据（需要通过 VTE parser 解析）
    PtyOutput(Vec<u8>),

    /// 向 PTY 写入数据（用于终端响应,如 DSR）
    PtyWrite(Vec<u8>),

    /// 窗口标题改变
    TitleChanged(String),

    /// 响铃
    BellRing,

    /// 剪贴板操作
    ClipboardCopy(String),
    ClipboardPaste(String),

    /// 终端尺寸改变
    Resized {
        rows: u16,
        cols: u16,
    },

    /// Shell Integration 事件
    #[cfg(feature = "shell-integration")]
    ShellIntegration(ShellIntegrationEvent),

    /// PTY 进程退出
    ProcessExit {
        exit_code: Option<i32>,
    },

    /// 错误事件
    Error(Arc<anyhow::Error>),
}

/// Shell Integration 事件
#[cfg(feature = "shell-integration")]
#[derive(Debug, Clone)]
pub enum ShellIntegrationEvent {
    /// 提示符开始
    PromptStart { line: usize },

    /// 命令开始
    CommandStart { line: usize },

    /// 命令执行中
    CommandExecuting { line: usize },

    /// 命令结束
    CommandFinished {
        line: usize,
        exit_code: Option<i32>,
    },

    /// 工作目录改变
    WorkingDirectoryChanged(String),

    /// 原始 OSC 序列 (用于测试和调试)
    RawOscSequence(crate::shell_integration::OscSequence),
}

/// 终端事件监听器 trait
pub trait TerminalEventListener: Send + Sync {
    /// 处理终端事件
    fn on_event(&self, event: TerminalEvent);
}

/// 使用 flume channel 实现的事件监听器
#[derive(Clone)]
pub struct ChannelEventListener {
    tx: flume::Sender<TerminalEvent>,
}

impl ChannelEventListener {
    pub fn new(tx: flume::Sender<TerminalEvent>) -> Self {
        Self { tx }
    }
}

impl TerminalEventListener for ChannelEventListener {
    fn on_event(&self, event: TerminalEvent) {
        let _ = self.tx.send(event);
    }
}

/// 空事件监听器（用于测试）
pub struct NullEventListener;

impl TerminalEventListener for NullEventListener {
    fn on_event(&self, _event: TerminalEvent) {
        // 什么都不做
    }
}
