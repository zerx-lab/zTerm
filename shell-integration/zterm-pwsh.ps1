# zTerm Shell Integration for PowerShell
# Add this to your $PROFILE or source it: . path\to\zterm-pwsh.ps1

# Only enable if running in zTerm
if ($env:ZTERM_SHELL_INTEGRATION -ne "1") {
    return
}

function Send-OscSequence {
    param([string]$Code)
    Write-Host -NoNewline "$([char]0x1b)]$Code$([char]0x07)"
}

# Store original prompt function
$script:OriginalPrompt = $function:prompt

# Override prompt to send OSC sequences
function prompt {
    # Send command finished with last exit code
    $exitCode = if ($?) { 0 } else { 1 }
    Send-OscSequence "133;D;$exitCode"

    # Send working directory (OSC 7)
    $cwd = (Get-Location).Path -replace '\\', '/'
    Send-OscSequence "7;file://localhost/$cwd"

    # Send prompt start
    Send-OscSequence "133;A"

    # Call original prompt
    $result = & $script:OriginalPrompt

    # Send command start (end of prompt)
    Send-OscSequence "133;B"

    return $result
}

# Use PSReadLine to detect command execution
if (Get-Module -ListAvailable -Name PSReadLine) {
    Set-PSReadLineKeyHandler -Key Enter -ScriptBlock {
        # Send command executing
        Send-OscSequence "133;C"

        # Get the command text
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        # Send command text (URL encoded)
        $encoded = [System.Uri]::EscapeDataString($line)
        Send-OscSequence "633;E;$encoded"

        # Accept the line
        [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
    }
}

Write-Host "zTerm shell integration enabled" -ForegroundColor Green
