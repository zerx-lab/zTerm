//! Shell detection utilities

use std::env;

/// Detect the default shell for the current platform
pub fn detect_shell() -> String {
    #[cfg(windows)]
    {
        detect_windows_shell()
    }

    #[cfg(unix)]
    {
        detect_unix_shell()
    }
}

#[cfg(windows)]
fn detect_windows_shell() -> String {
    // Check for SHELL environment variable first
    if let Ok(shell) = env::var("SHELL") {
        return shell;
    }

    // Check for COMSPEC
    if let Ok(shell) = env::var("COMSPEC") {
        return shell;
    }

    // Check for PowerShell
    if which::which("pwsh").is_ok() {
        return "pwsh".to_string();
    }

    if which::which("powershell").is_ok() {
        return "powershell".to_string();
    }

    // Fallback to cmd
    "cmd.exe".to_string()
}

#[cfg(unix)]
fn detect_unix_shell() -> String {
    // Try SHELL environment variable first
    if let Ok(shell) = env::var("SHELL") {
        return shell;
    }

    // Check for common shells
    let shell_candidates = [
        "/bin/zsh",
        "/bin/bash",
        "/bin/fish",
        "/bin/sh",
    ];

    for shell in shell_candidates {
        if std::path::Path::new(shell).exists() {
            return shell.to_string();
        }
    }

    // Last resort
    "/bin/sh".to_string()
}

/// Get shell arguments for login shell behavior
#[allow(dead_code)]
pub fn get_login_shell_args(shell: &str) -> Vec<String> {
    let shell_name = std::path::Path::new(shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(shell);

    match shell_name {
        "bash" | "zsh" | "fish" | "sh" => vec!["-l".to_string()],
        "pwsh" | "powershell" => vec!["-NoLogo".to_string()],
        "cmd" | "cmd.exe" => vec![],
        _ => vec![],
    }
}
