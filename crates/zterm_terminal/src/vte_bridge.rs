//! VTE 解析器桥接

use crate::event::{TerminalEvent, TerminalEventListener};
use std::sync::Arc;
use vte::{Params, Perform};

/// VTE Performer - 处理 VTE 解析的输出
pub struct VtePerformer {
    /// 事件监听器
    event_listener: Arc<dyn TerminalEventListener>,

    /// 当前标题（用于 OSC 0/2）
    current_title: String,
}

impl VtePerformer {
    pub fn new(event_listener: Arc<dyn TerminalEventListener>) -> Self {
        Self {
            event_listener,
            current_title: String::new(),
        }
    }
}

impl Perform for VtePerformer {
    /// 打印字符
    fn print(&mut self, _c: char) {
        // TODO: 将字符发送到终端 grid
        // 目前只触发重绘
        self.event_listener.on_event(TerminalEvent::Wakeup);
    }

    /// 执行 C0 或 C1 控制字符
    fn execute(&mut self, byte: u8) {
        match byte {
            0x07 => {
                // BEL (Bell)
                self.event_listener.on_event(TerminalEvent::BellRing);
            }
            0x08 => {
                // BS (Backspace)
                // TODO: 移动光标
            }
            0x09 => {
                // HT (Tab)
                // TODO: 移动到下一个 tab stop
            }
            0x0A | 0x0B | 0x0C => {
                // LF, VT, FF (Line feed)
                // TODO: 换行
            }
            0x0D => {
                // CR (Carriage return)
                // TODO: 光标移动到行首
            }
            _ => {
                // 其他控制字符
                tracing::trace!("Execute control: 0x{:02x}", byte);
            }
        }
    }

    /// 处理 CSI 序列
    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        if ignore {
            return;
        }

        tracing::trace!(
            "CSI dispatch: action={}, params={:?}, intermediates={:?}",
            action,
            params,
            intermediates
        );

        match action {
            'm' => {
                // SGR (Select Graphic Rendition)
                // TODO: 处理文本属性
            }
            'A' => {
                // CUU (Cursor Up)
                // TODO: 移动光标
            }
            'B' => {
                // CUD (Cursor Down)
                // TODO: 移动光标
            }
            'C' => {
                // CUF (Cursor Forward)
                // TODO: 移动光标
            }
            'D' => {
                // CUB (Cursor Back)
                // TODO: 移动光标
            }
            'H' | 'f' => {
                // CUP (Cursor Position)
                // TODO: 移动光标到指定位置
            }
            'J' => {
                // ED (Erase Display)
                // TODO: 清除显示
            }
            'K' => {
                // EL (Erase Line)
                // TODO: 清除行
            }
            _ => {
                tracing::debug!("Unhandled CSI: {}", action);
            }
        }

        self.event_listener.on_event(TerminalEvent::Wakeup);
    }

    /// 处理 ESC 序列
    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        if ignore {
            return;
        }

        tracing::trace!(
            "ESC dispatch: byte=0x{:02x}, intermediates={:?}",
            byte,
            intermediates
        );

        match byte {
            b'D' => {
                // IND (Index)
                // TODO: 向下滚动
            }
            b'E' => {
                // NEL (Next Line)
                // TODO: 移动到下一行
            }
            b'M' => {
                // RI (Reverse Index)
                // TODO: 向上滚动
            }
            b'c' => {
                // RIS (Reset to Initial State)
                // TODO: 重置终端
            }
            _ => {
                tracing::debug!("Unhandled ESC: 0x{:02x}", byte);
            }
        }

        self.event_listener.on_event(TerminalEvent::Wakeup);
    }

    /// 处理 OSC (Operating System Command)
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.is_empty() {
            return;
        }

        // 解析 OSC 命令号
        let cmd = String::from_utf8_lossy(params[0]);

        tracing::trace!("OSC dispatch: cmd={}, params={:?}", cmd, params);

        match cmd.as_ref() {
            "0" | "2" => {
                // Set window title
                if params.len() > 1 {
                    let title = String::from_utf8_lossy(params[1]).to_string();
                    if title != self.current_title {
                        self.current_title = title.clone();
                        self.event_listener
                            .on_event(TerminalEvent::TitleChanged(title));
                    }
                }
            }
            "52" => {
                // Clipboard operations
                // TODO: 处理剪贴板
            }
            "133" => {
                // Shell Integration (FinalTerm/VSCode)
                // TODO: 处理 shell integration
                #[cfg(feature = "shell-integration")]
                {
                    self.handle_osc_133(params);
                }
            }
            "633" => {
                // Shell Integration (VSCode extended)
                // TODO: 处理 shell integration
                #[cfg(feature = "shell-integration")]
                {
                    self.handle_osc_633(params);
                }
            }
            _ => {
                tracing::debug!("Unhandled OSC: {}", cmd);
            }
        }
    }

    /// Hook - DCS 开始
    fn hook(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        tracing::trace!(
            "Hook: action={}, params={:?}, intermediates={:?}",
            action,
            params,
            intermediates
        );
    }

    /// Put - DCS 数据
    fn put(&mut self, byte: u8) {
        tracing::trace!("Put: 0x{:02x}", byte);
    }

    /// Unhook - DCS 结束
    fn unhook(&mut self) {
        tracing::trace!("Unhook");
    }
}

#[cfg(feature = "shell-integration")]
impl VtePerformer {
    fn handle_osc_133(&mut self, params: &[&[u8]]) {
        use crate::event::ShellIntegrationEvent;

        if params.len() < 2 {
            return;
        }

        let subcommand = String::from_utf8_lossy(params[1]);

        match subcommand.as_ref() {
            "A" => {
                // Prompt start
                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::PromptStart { line: 0 }, // TODO: 获取实际行号
                ));
            }
            "B" => {
                // Command start
                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::CommandStart { line: 0 },
                ));
            }
            "C" => {
                // Command executing
                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::CommandExecuting { line: 0 },
                ));
            }
            "D" => {
                // Command finished
                let exit_code = if params.len() > 2 {
                    String::from_utf8_lossy(params[2])
                        .split(';')
                        .next()
                        .and_then(|s| s.parse::<i32>().ok())
                } else {
                    None
                };

                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::CommandFinished {
                        line: 0,
                        exit_code,
                    },
                ));
            }
            _ => {
                tracing::debug!("Unhandled OSC 133 subcommand: {}", subcommand);
            }
        }
    }

    fn handle_osc_633(&mut self, params: &[&[u8]]) {
        use crate::event::ShellIntegrationEvent;

        if params.len() < 2 {
            return;
        }

        let subcommand = String::from_utf8_lossy(params[1]);

        match subcommand.as_ref() {
            cmd if cmd.starts_with("P;Cwd=") => {
                // Working directory
                let cwd = cmd.trim_start_matches("P;Cwd=").to_string();
                self.event_listener.on_event(TerminalEvent::ShellIntegration(
                    ShellIntegrationEvent::WorkingDirectoryChanged(cwd),
                ));
            }
            _ => {
                tracing::debug!("Unhandled OSC 633 subcommand: {}", subcommand);
            }
        }
    }
}
