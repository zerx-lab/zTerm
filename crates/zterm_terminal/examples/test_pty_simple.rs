//! 简单的 PTY 测试 - 带 DSR 响应
//!
//! 运行: cargo run --example test_pty_simple -p zterm_terminal

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Simple PTY Test (with DSR response) ===\n");

    // 创建 PTY
    let pty_system = portable_pty::native_pty_system();

    let pair = pty_system
        .openpty(portable_pty::PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("Failed to open PTY");

    println!("PTY created successfully");

    // 启动 PowerShell
    let mut cmd = portable_pty::CommandBuilder::new("pwsh.exe");
    cmd.args(&["-NoLogo", "-NoProfile"]);
    cmd.env("TERM", "xterm-256color");

    println!("Spawning PowerShell...");
    let mut child = pair.slave.spawn_command(cmd).expect("Failed to spawn");
    println!("PowerShell spawned");

    // 关键：在 Windows 上必须 drop slave
    drop(pair.slave);
    println!("Slave dropped");

    // 获取 reader 和 writer
    let mut reader = pair.master.try_clone_reader().expect("Failed to clone reader");
    let writer = pair.master.take_writer().expect("Failed to take writer");
    let writer = Arc::new(std::sync::Mutex::new(writer));

    // 停止标志
    let running = Arc::new(AtomicBool::new(true));

    println!("\nStarting read thread...\n");

    // 读取线程
    let writer_clone = writer.clone();
    let running_clone = running.clone();
    let read_handle = thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut total_bytes = 0;
        let mut read_count = 0;

        while running_clone.load(Ordering::Relaxed) {
            println!("[Reader] Calling read()...");
            match reader.read(&mut buf) {
                Ok(0) => {
                    println!("[Reader] EOF");
                    break;
                }
                Ok(n) => {
                    read_count += 1;
                    total_bytes += n;
                    let data = String::from_utf8_lossy(&buf[..n]);
                    println!("[Reader] Read #{}: {} bytes", read_count, n);
                    println!("[Reader] Data: {:?}", data);

                    // 检查是否是 DSR 查询
                    if data.contains("\x1b[6n") {
                        println!("[Reader] >>> Detected DSR query, sending response...");
                        // 发送 DSR 响应: \x1b[row;colR (row=1, col=1)
                        let response = b"\x1b[1;1R";
                        if let Ok(mut w) = writer_clone.lock() {
                            if let Err(e) = w.write_all(response) {
                                println!("[Reader] Failed to send DSR response: {:?}", e);
                            } else {
                                let _ = w.flush();
                                println!("[Reader] DSR response sent: {:?}", String::from_utf8_lossy(response));
                            }
                        }
                    }

                    println!();
                }
                Err(e) => {
                    println!("[Reader] Error: {:?}", e);
                    break;
                }
            }

            // 读取足够后退出
            if read_count >= 15 || total_bytes > 2000 {
                println!("[Reader] Enough data received, exiting");
                break;
            }
        }

        println!("[Reader] Total: {} bytes in {} reads", total_bytes, read_count);
    });

    // 等待一会儿让 PowerShell 启动并输出提示符
    println!("[Main] Waiting for PowerShell to start (3 seconds)...");
    thread::sleep(Duration::from_secs(3));

    // 发送一个简单命令
    println!("[Writer] Sending 'echo hello'...");
    {
        let mut w = writer.lock().unwrap();
        w.write_all(b"echo hello\r").expect("Failed to write");
        w.flush().expect("Failed to flush");
    }

    // 等待输出
    thread::sleep(Duration::from_secs(2));

    // 停止读取线程
    running.store(false, Ordering::Relaxed);

    // 发送退出命令
    println!("[Writer] Sending 'exit'...");
    {
        let mut w = writer.lock().unwrap();
        w.write_all(b"exit\r").expect("Failed to write");
        w.flush().expect("Failed to flush");
    }

    // 等待子进程退出
    println!("\n[Main] Waiting for child to exit...");
    match child.wait() {
        Ok(status) => println!("[Main] Child exited with: {:?}", status),
        Err(e) => println!("[Main] Error waiting for child: {:?}", e),
    }

    // 等待读取线程
    let _ = read_handle.join();

    println!("\n=== Test Complete ===");
}
