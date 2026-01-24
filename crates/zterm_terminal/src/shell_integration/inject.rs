//! Shell integration script injection
//!
//! This module provides scripts and utilities for automatically injecting
//! shell integration support into various shells.
//!
//! Implementation follows VS Code's approach:
//! - PowerShell: Uses `-NoExit -Command "try { . 'script.ps1' } catch {}"` to load script
//! - Script is saved to a temporary file and sourced via dot sourcing

use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// PowerShell Shell Integration script
///
/// This script hooks into PowerShell's prompt to emit OSC 133/633 sequences
/// for shell integration support.
pub const POWERSHELL_INTEGRATION: &str = r#"
# zTerm Shell Integration for PowerShell
function __zterm_osc { param([string]$code) Write-Host -NoNewline "`e]$code`a" }
function __zterm_urlencode { param([string]$text) [uri]::EscapeDataString($text) }

# Save original prompt if exists
if (Test-Path Function:\Prompt) { Copy-Item Function:\Prompt Function:\__zterm_original_prompt }

# Track command state
$global:__zterm_last_cmd = $null

function Prompt {
    $exit_code = if ($null -eq $LASTEXITCODE) { 0 } else { $LASTEXITCODE }

    # Send command finished if we had a command
    if ($global:__zterm_last_cmd) {
        # IMPORTANT: Send command text BEFORE finished, so handler can associate it with the zone
        __zterm_osc "633;E;$(__zterm_urlencode $global:__zterm_last_cmd)"
        __zterm_osc "133;D;$exit_code"
        $global:__zterm_last_cmd = $null
    }

    # Send working directory
    try {
        $loc = Get-Location
        if ($loc.Provider.Name -eq 'FileSystem') {
            $uri = [Uri]::new($loc.ProviderPath).AbsoluteUri
            __zterm_osc "7;$uri"
        }
    } catch {}

    # Send prompt start
    __zterm_osc "133;A"

    # Get original prompt
    $p = if (Test-Path Function:\__zterm_original_prompt) {
        & __zterm_original_prompt
    } else {
        "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
    }

    # Send command input start
    __zterm_osc "133;B"
    return $p
}

# Hook PSReadLine if available
try {
    if (Get-Module -ListAvailable -Name PSReadLine) {
        Import-Module PSReadLine -ErrorAction SilentlyContinue
        Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {
            $line = $null; $cursor = $null
            [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
            if ($line) {
                $global:__zterm_last_cmd = $line
                __zterm_osc "133;C"
            }
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
        }
    }
} catch {}
"#;

/// Get shell integration script for the given shell
pub fn get_integration_script(shell: &str) -> Option<&'static str> {
    let shell_name = std::path::Path::new(shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(shell)
        .to_lowercase();

    match shell_name.as_str() {
        "pwsh" | "pwsh.exe" | "powershell" | "powershell.exe" => Some(POWERSHELL_INTEGRATION),
        _ => None,
    }
}

/// Check if a shell supports automatic integration
pub fn supports_integration(shell: &str) -> bool {
    get_integration_script(shell).is_some()
}

/// Get the shell integration script file path (creates temp file if needed)
pub fn get_integration_script_path(shell: &str) -> Option<PathBuf> {
    let script = get_integration_script(shell)?;

    // Create temp directory for shell integration scripts
    let temp_dir = std::env::temp_dir().join("zterm_shell_integration");
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        tracing::error!("Failed to create shell integration temp directory: {}", e);
        return None;
    }

    // Determine script filename based on shell
    let shell_name = std::path::Path::new(shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(shell)
        .to_lowercase();

    let script_path = temp_dir.join(format!("{}.ps1", shell_name));

    // Write script to file
    if let Ok(mut file) = fs::File::create(&script_path) {
        if let Err(e) = file.write_all(script.as_bytes()) {
            tracing::error!("Failed to write shell integration script: {}", e);
            return None;
        }
        tracing::info!("Created shell integration script at: {:?}", script_path);
        Some(script_path)
    } else {
        tracing::error!("Failed to create shell integration script file");
        None
    }
}

/// Get shell arguments for loading integration (VS Code style)
///
/// For PowerShell: `["-NoLogo", "-NoExit", "-Command", "try { . 'script.ps1' } catch {}"]`
pub fn get_shell_args_with_integration(shell: &str) -> Option<Vec<String>> {
    let script_path = get_integration_script_path(shell)?;
    let script_path_str = script_path.to_string_lossy();

    let shell_name = std::path::Path::new(shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(shell)
        .to_lowercase();

    match shell_name.as_str() {
        "pwsh" | "pwsh.exe" | "powershell" | "powershell.exe" => {
            // VS Code style: powershell.exe -noexit -command 'try { . "script.ps1" } catch {}'
            Some(vec![
                "-NoLogo".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                format!("try {{ . '{}' }} catch {{}}", script_path_str),
            ])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_integration_script_not_empty() {
        assert!(!POWERSHELL_INTEGRATION.is_empty());
        assert!(POWERSHELL_INTEGRATION.contains("133;"));
        assert!(POWERSHELL_INTEGRATION.contains("__zterm_osc"));
    }

    #[test]
    fn test_get_integration_script_powershell() {
        assert!(get_integration_script("pwsh").is_some());
        assert!(get_integration_script("powershell").is_some());
        assert!(get_integration_script("pwsh.exe").is_some());
        assert!(get_integration_script("PowerShell.exe").is_some());
    }

    #[test]
    fn test_get_integration_script_unsupported() {
        assert!(get_integration_script("bash").is_none());
        assert!(get_integration_script("zsh").is_none());
        assert!(get_integration_script("cmd").is_none());
    }

    #[test]
    fn test_supports_integration() {
        assert!(supports_integration("pwsh"));
        assert!(supports_integration("powershell"));
        assert!(!supports_integration("bash"));
        assert!(!supports_integration("cmd"));
    }

    #[test]
    fn test_integration_script_with_path() {
        assert!(get_integration_script("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe").is_some());
        assert!(get_integration_script("/usr/bin/pwsh").is_some());
    }

    #[test]
    fn test_get_shell_args_with_integration() {
        // Test PowerShell
        let args = get_shell_args_with_integration("pwsh");
        assert!(args.is_some());
        let args = args.unwrap();
        assert_eq!(args[0], "-NoLogo");
        assert_eq!(args[1], "-NoExit");
        assert_eq!(args[2], "-Command");
        assert!(args[3].contains("try"));
        assert!(args[3].contains("catch"));

        // Test unsupported shell
        assert!(get_shell_args_with_integration("bash").is_none());
    }

    #[test]
    fn test_get_integration_script_path() {
        // This test actually creates a file, so we test that it returns Some
        let path = get_integration_script_path("pwsh");
        assert!(path.is_some());

        // Verify the file was created
        if let Some(p) = path {
            assert!(p.exists());
            assert!(p.to_string_lossy().contains("zterm_shell_integration"));
            assert!(p.to_string_lossy().ends_with(".ps1"));
        }
    }
}
