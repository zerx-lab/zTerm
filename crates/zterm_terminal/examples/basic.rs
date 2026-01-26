//! 基本使用示例

use anyhow::Result;
use std::time::Duration;
use zterm_terminal::{PtyConfig, Terminal, TerminalConfig, TerminalEvent, TerminalSize};

fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("zterm_terminal=debug")
        .init();

    println!("创建终端...");

    // 创建 PTY 配置
    let pty_config = PtyConfig {
        shell: None, // 使用系统默认 shell
        shell_args: vec![],
        working_directory: None,
        env: vec![
            ("TERM".to_string(), "xterm-256color".to_string()),
            ("COLORTERM".to_string(), "truecolor".to_string()),
        ],
        initial_size: TerminalSize::new(24, 80),
    };

    // 创建终端配置
    let term_config = TerminalConfig::default();

    // 创建终端实例
    let terminal = Terminal::new(pty_config, term_config)?;

    println!("终端创建成功！");
    println!("当前尺寸: {:?}", terminal.size());

    // 获取事件接收器
    let event_rx = terminal.event_receiver();

    // 写入测试命令
    println!("\n发送命令: echo 'Hello from zterm_terminal!'");
    terminal.write(b"echo 'Hello from zterm_terminal!'\n")?;

    // 监听事件（10 秒超时）
    println!("\n监听终端事件（10 秒）...\n");
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(10) {
        match event_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => match event {
                TerminalEvent::Wakeup => {
                    println!("[Event] 终端需要重绘");
                }
                TerminalEvent::TitleChanged(title) => {
                    println!("[Event] 标题改变: {}", title);
                }
                TerminalEvent::BellRing => {
                    println!("[Event] 响铃");
                }
                TerminalEvent::Resized { rows, cols } => {
                    println!("[Event] 尺寸改变: {}x{}", cols, rows);
                }
                #[cfg(feature = "shell-integration")]
                TerminalEvent::ShellIntegration(shell_event) => {
                    println!("[Event] Shell Integration: {:?}", shell_event);
                }
                TerminalEvent::ProcessExit { exit_code } => {
                    println!("[Event] 进程退出: {:?}", exit_code);
                    break;
                }
                TerminalEvent::Error(err) => {
                    eprintln!("[Event] 错误: {}", err);
                    break;
                }
                _ => {}
            },
            Err(flume::RecvTimeoutError::Timeout) => {
                // 超时，继续
            }
            Err(flume::RecvTimeoutError::Disconnected) => {
                println!("事件通道关闭");
                break;
            }
        }
    }

    println!("\n关闭终端...");
    terminal.shutdown()?;

    println!("示例完成！");
    Ok(())
}
