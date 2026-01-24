//! Shell integration script injection
//!
//! This module provides scripts and utilities for automatically injecting
//! shell integration support into various shells.

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
        __zterm_osc "133;D;$exit_code"
        __zterm_osc "633;E;$(__zterm_urlencode $global:__zterm_last_cmd)"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_integration_script_not_empty() {
        assert!(!POWERSHELL_INTEGRATION.is_empty());
        assert!(POWERSHELL_INTEGRATION.contains("OSC 133"));
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
}
