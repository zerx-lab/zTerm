# 测试 OSC 序列发送
#
# 此脚本直接发送各种 OSC 序列,用于手动测试终端是否正确接收

Write-Host "=== OSC 序列测试脚本 ===" -ForegroundColor Cyan
Write-Host ""

# 加载 shell integration
$integrationScript = Join-Path $PSScriptRoot "zterm-integration.ps1"
if (Test-Path $integrationScript) {
    Write-Host "✓ 加载 shell integration..." -ForegroundColor Green
    . $integrationScript
} else {
    Write-Host "✗ 找不到 shell integration 脚本" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "--- 测试 1: 基础 OSC 133 序列 ---" -ForegroundColor Yellow

# 手动发送 OSC 133;A (Prompt Start)
Write-Host "`e]133;A`a" -NoNewline
Write-Host "已发送: OSC 133;A (Prompt Start)"

Start-Sleep -Milliseconds 200

# 手动发送 OSC 133;B (Command Start)
Write-Host "`e]133;B`a" -NoNewline
Write-Host "已发送: OSC 133;B (Command Start)"

Start-Sleep -Milliseconds 200

# 手动发送 OSC 133;C (Command Executing)
Write-Host "`e]133;C`a" -NoNewline
Write-Host "已发送: OSC 133;C (Command Executing)"

Start-Sleep -Milliseconds 200

# 手动发送 OSC 133;D;0 (Command Finished, exit code 0)
Write-Host "`e]133;D;0`a" -NoNewline
Write-Host "已发送: OSC 133;D;0 (Command Finished)"

Write-Host ""
Write-Host "--- 测试 2: OSC 531 JSON 数据 ---" -ForegroundColor Yellow

# 测试 shell_started
$testData1 = @{
    type = "shell_started"
    shell = "PowerShell"
    version = $PSVersionTable.PSVersion.ToString()
    test = $true
} | ConvertTo-Json -Compress

$escaped1 = __ZTerm-Escape-Value $testData1
Write-Host "`e]531;$escaped1`a" -NoNewline
Write-Host "已发送: OSC 531 (shell_started)"

Start-Sleep -Milliseconds 200

# 测试 command_start
$testData2 = @{
    type = "command_start"
    command = "Get-Date"
    cwd = $PWD.Path
    test = $true
} | ConvertTo-Json -Compress

$escaped2 = __ZTerm-Escape-Value $testData2
Write-Host "`e]531;$escaped2`a" -NoNewline
Write-Host "已发送: OSC 531 (command_start)"

Start-Sleep -Milliseconds 200

# 测试带换行符的 JSON (测试转义)
$testData3 = @{
    type = "custom"
    text = "line1`nline2`nline3"
    test = $true
} | ConvertTo-Json -Compress

$escaped3 = __ZTerm-Escape-Value $testData3
Write-Host "`e]531;$escaped3`a" -NoNewline
Write-Host "已发送: OSC 531 (custom with newlines)"

Write-Host ""
Write-Host "--- 测试 3: OSC 7 (Working Directory) ---" -ForegroundColor Yellow

$hostname = $env:COMPUTERNAME
$urlPath = $PWD.Path -replace '\\', '/' -replace ' ', '%20'
$fileUrl = "file://$hostname/$urlPath"
Write-Host "`e]7;$fileUrl`a" -NoNewline
Write-Host "已发送: OSC 7 ($fileUrl)"

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "如果终端正确解析 OSC 序列,你应该能在日志或调试输出中看到这些序列。"
Write-Host ""
