# zTerm Shell Integration for PowerShell
# 实现 Warp 风格的命令块系统
#
# 使用 OSC 133 标准协议标记块边界 + OSC 531 传输 JSON 元数据

# ============================================================================
# 防止重复加载
# ============================================================================

if ((Test-Path variable:global:__ZTermState) -and
    $null -ne $Global:__ZTermState.OriginalPrompt) {
    return
}

# 禁用受限语言模式
if ($ExecutionContext.SessionState.LanguageMode -ne "FullLanguage") {
    return
}

# ============================================================================
# 全局状态
# ============================================================================

$Global:__ZTermState = @{
    OriginalPrompt = $function:prompt
    OriginalPSConsoleHostReadLine = $null
    LastHistoryId = -1
    LastDirectory = $PWD.Path
    HasPSReadLine = $false
}

# ============================================================================
# OSC 序列定义
# ============================================================================

# OSC 133: 标准 Shell Integration 协议 (FinalTerm, iTerm2, Warp)
# A = 提示符开始
# B = 提示符结束 / 命令输入开始
# C = 命令执行前
# D = 命令执行完成 (可选 exit_code 参数)

# OSC 531: zTerm 自定义协议,传输 JSON 元数据
# 说明: VS Code 官方建议通用脚本使用 OSC 133 (FinalTerm 标准)
#       OSC 531 不与已知标准冲突,用于传输 zTerm 特定元数据
$Script:ZTERM_OSC_METADATA = 531

# ============================================================================
# 辅助函数
# ============================================================================

# 转义字符串 (防止控制字符破坏 OSC 序列)
# 基于 VS Code 实现,转义: 控制字符(0x00-0x1f)、反斜杠、换行、分号
function Global:__ZTerm-Escape-Value([string]$value) {
    if (-not $value) { return "" }

    # 使用正则替换特殊字符为 \xHH 格式
    [regex]::Replace($value, "[$([char]0x00)-$([char]0x1f)\\\n;]", {
        param($match)
        -Join ([System.Text.Encoding]::UTF8.GetBytes($match.Value) |
               ForEach-Object { '\x{0:x2}' -f $_ })
    })
}

# 发送 OSC 133 序列 (块边界标记)
function Send-OSC133 {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet('A', 'B', 'C', 'D')]
        [string]$Marker,
        [string]$Data = ""
    )

    if ($Data) {
        return "$([char]0x1b)]133;$Marker;$Data$([char]0x07)"
    } else {
        return "$([char]0x1b)]133;$Marker$([char]0x07)"
    }
}

# 发送 JSON 元数据 (OSC 531)
function Send-ZTermMetadata {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Data
    )

    $json = $Data | ConvertTo-Json -Compress
    # 转义 JSON 中的特殊字符
    $escaped = __ZTerm-Escape-Value $json
    return "$([char]0x1b)]$Script:ZTERM_OSC_METADATA;$escaped$([char]0x07)"
}

# 发送 OSC 7 (当前工作目录) - 标准协议
function Send-OSC7 {
    param([string]$Path)

    # 转换为 file:// URL
    $hostname = $env:COMPUTERNAME
    # 转义路径中的特殊字符
    $urlPath = $Path -replace '\\', '/' -replace ' ', '%20'
    $fileUrl = "file://$hostname/$urlPath"

    return "$([char]0x1b)]7;$fileUrl$([char]0x07)"
}

# 获取 Git 信息
function Get-GitInfo {
    try {
        $gitBranch = git rev-parse --abbrev-ref HEAD 2>$null
        if ($gitBranch) {
            $gitStatus = git status --porcelain 2>$null
            $hasChanges = $null -ne $gitStatus -and $gitStatus.Length -gt 0

            return @{
                branch = $gitBranch
                has_changes = $hasChanges
                repo_root = (git rev-parse --show-toplevel 2>$null)
            }
        }
    } catch {
        # Not in a git repo
    }
    return $null
}

# ============================================================================
# Prompt 函数 (核心)
# ============================================================================

function Global:prompt {
    # 获取退出码 (在任何命令执行前保存)
    $exitCode = $LASTEXITCODE
    if ($null -eq $exitCode) { $exitCode = 0 }

    # 使用 $? 检测上一个命令是否成功
    $success = $?
    if (-not $success -and $exitCode -eq 0) { $exitCode = 1 }

    # 获取最新的历史记录
    Set-StrictMode -Off
    $lastHistoryEntry = Get-History -Count 1

    $result = ""

    # 如果有命令执行完成,发送 OSC 133;D
    if ($Global:__ZTermState.LastHistoryId -ne -1) {
        if ($null -eq $lastHistoryEntry -or
            $lastHistoryEntry.Id -eq $Global:__ZTermState.LastHistoryId) {
            # 没有新命令 (Ctrl+C, 空回车等)
            $result += Send-OSC133 -Marker 'D'
        } else {
            # 有命令执行,包含退出码
            $result += Send-OSC133 -Marker 'D' -Data $exitCode

            # 发送命令完成元数据
            $duration = 0
            if ($lastHistoryEntry.EndExecutionTime -and $lastHistoryEntry.StartExecutionTime) {
                $duration = [long]($lastHistoryEntry.EndExecutionTime -
                                   $lastHistoryEntry.StartExecutionTime).TotalMilliseconds
            }

            $result += Send-ZTermMetadata -Data @{
                type = "command_end"
                exit_code = $exitCode
                duration_ms = $duration
                timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
            }
        }
    }

    # 检测目录变更
    $currentDir = $PWD.Path
    if ($currentDir -ne $Global:__ZTermState.LastDirectory) {
        $Global:__ZTermState.LastDirectory = $currentDir

        # 发送目录变更元数据
        $gitInfo = Get-GitInfo
        $dirData = @{
            type = "directory_changed"
            cwd = $currentDir
            timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        }
        if ($gitInfo) { $dirData.git = $gitInfo }

        $result += Send-ZTermMetadata -Data $dirData
    }

    # OSC 133;A - 提示符开始
    $result += Send-OSC133 -Marker 'A'

    # 发送 prompt_start 元数据
    $gitInfo = Get-GitInfo
    $promptData = @{
        type = "prompt_start"
        cwd = $PWD.Path
        user = $env:USERNAME
        hostname = $env:COMPUTERNAME
        timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    }
    if ($gitInfo) { $promptData.git = $gitInfo }
    $result += Send-ZTermMetadata -Data $promptData

    # OSC 7 - 当前目录
    $result += Send-OSC7 -Path $PWD.Path

    # 恢复 $? 状态 (调用原始 prompt 前)
    if ($exitCode -ne 0) {
        Write-Error "zterm-restore-exit-code" -ErrorAction SilentlyContinue
    }

    # 调用原始 prompt
    $originalPromptOutput = & $Global:__ZTermState.OriginalPrompt
    $result += $originalPromptOutput

    # OSC 133;B - 命令输入开始
    $result += Send-OSC133 -Marker 'B'

    # 更新历史 ID
    if ($null -ne $lastHistoryEntry) {
        $Global:__ZTermState.LastHistoryId = $lastHistoryEntry.Id
    }

    return $result
}

# ============================================================================
# PSReadLine 集成 (如果可用)
# ============================================================================

if (Get-Module -Name PSReadLine) {
    $Global:__ZTermState.HasPSReadLine = $true

    # 保存原始 PSConsoleHostReadLine
    if (Test-Path function:PSConsoleHostReadLine) {
        $Global:__ZTermState.OriginalPSConsoleHostReadLine = $function:PSConsoleHostReadLine
    }

    # Hook PSConsoleHostReadLine 获取命令行内容
    function Global:PSConsoleHostReadLine {
        # 获取用户输入的命令
        $commandLine = ""
        if ($Global:__ZTermState.OriginalPSConsoleHostReadLine) {
            $commandLine = & $Global:__ZTermState.OriginalPSConsoleHostReadLine
        } else {
            # 如果没有原始函数,使用 PSReadLine 默认行为
            $commandLine = [Microsoft.PowerShell.PSConsoleReadLine]::ReadLine(
                $host.Runspace,
                $ExecutionContext
            )
        }

        # 发送命令执行开始序列
        $result = ""

        # OSC 133;C - 命令执行前
        $result += Send-OSC133 -Marker 'C'

        # 发送命令开始元数据 (包含完整命令行)
        $result += Send-ZTermMetadata -Data @{
            type = "command_start"
            command = $commandLine
            cwd = $PWD.Path
            timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        }

        # 直接写入 Console (避免换行)
        [Console]::Write($result)

        return $commandLine
    }
}

# ============================================================================
# 初始化
# ============================================================================

# 发送 Shell 启动事件
$initData = @{
    type = "shell_started"
    shell = "PowerShell"
    version = $PSVersionTable.PSVersion.ToString()
    pid = $PID
    os = [System.Environment]::OSVersion.VersionString
    has_psreadline = $Global:__ZTermState.HasPSReadLine
    timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
}

[Console]::Write((Send-ZTermMetadata -Data $initData))

# 设置初始目录
[Console]::Write((Send-OSC7 -Path $PWD.Path))

# 输出加载信息
Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  zTerm Shell Integration 已启用                            ║" -ForegroundColor Cyan
Write-Host "╠════════════════════════════════════════════════════════════╣" -ForegroundColor Cyan
Write-Host "║  命令块协议: OSC 133 (FinalTerm 标准)                      ║" -ForegroundColor Gray
Write-Host "║  元数据协议: OSC 531 (zTerm 自定义 JSON)                   ║" -ForegroundColor Gray
Write-Host "║  工作目录:   OSC 7 (标准)                                   ║" -ForegroundColor Gray
if ($Global:__ZTermState.HasPSReadLine) {
    Write-Host "║  PSReadLine: 已启用 (完整命令捕获)                         ║" -ForegroundColor Green
} else {
    Write-Host "║  PSReadLine: 未加载 (基础功能)                             ║" -ForegroundColor Yellow
}
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ============================================================================
# 公共 API (可选)
# ============================================================================

# 用户可以手动发送自定义元数据
function Global:Send-ZTermCustomData {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Data
    )

    $Data.type = "custom"
    $Data.timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    [Console]::Write((Send-ZTermMetadata -Data $Data))
}

# 导出函数供用户使用
Export-ModuleMember -Function Send-ZTermCustomData -ErrorAction SilentlyContinue
