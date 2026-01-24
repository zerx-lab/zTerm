<#
.SYNOPSIS
    Axon Term Windows 打包脚本

.DESCRIPTION
    构建并打包 Axon Term 为 Windows 安装程序。

.PARAMETER Architecture
    目标架构: x86_64 (默认) 或 aarch64

.PARAMETER Release
    构建 release 版本 (默认: true)

.PARAMETER Install
    构建后自动安装

.PARAMETER SkipBuild
    跳过构建步骤，仅打包

.EXAMPLE
    .\bundle-windows.ps1

.EXAMPLE
    .\bundle-windows.ps1 -Architecture aarch64 -Install
#>

[CmdletBinding()]
param(
    [ValidateSet("x86_64", "aarch64")]
    [string]$Architecture = "x86_64",

    [switch]$Release = $true,

    [switch]$Install,

    [switch]$SkipBuild,

    # 包名 (Cargo.toml 中的 package name)
    [string]$PackageName = "axon_app",

    # 输出的可执行文件名 (用于安装程序)
    [string]$OutputName = "axon_term"
)

# 严格模式
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 项目根目录
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not (Test-Path (Join-Path $ProjectRoot "Cargo.toml"))) {
    $ProjectRoot = Split-Path -Parent $PSScriptRoot
}

# 版本信息
$CargoToml = Get-Content (Join-Path $ProjectRoot "Cargo.toml") -Raw
if ($CargoToml -match 'version\s*=\s*"([^"]+)"') {
    $Version = $Matches[1]
} else {
    $Version = "0.1.0"
}

# 动态获取 target 目录 (从 .cargo/config.toml 读取)
function Get-TargetDir {
    $cargoConfigPath = Join-Path $ProjectRoot ".cargo\config.toml"
    if (Test-Path $cargoConfigPath) {
        $cargoConfig = Get-Content $cargoConfigPath -Raw
        if ($cargoConfig -match 'target-dir\s*=\s*"([^"]+)"') {
            $targetDir = $Matches[1]
            # 处理相对路径和绝对路径
            if ([System.IO.Path]::IsPathRooted($targetDir)) {
                return $targetDir
            } else {
                return Join-Path $ProjectRoot $targetDir
            }
        }
    }
    # 默认使用项目根目录下的 target
    return Join-Path $ProjectRoot "target"
}

# 路径配置
$TargetDir = Get-TargetDir
$ReleaseDir = Join-Path $TargetDir "release"
$InstallerDir = Join-Path $TargetDir "installer"
$ResourcesDir = Join-Path $ProjectRoot "resources\windows"
$AssetsDir = Join-Path $ProjectRoot "assets"

# 工具路径
$InnoSetupPath = "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe"
if (-not (Test-Path $InnoSetupPath)) {
    $InnoSetupPath = "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
}

# Rust target
$RustTarget = switch ($Architecture) {
    "x86_64"  { "x86_64-pc-windows-msvc" }
    "aarch64" { "aarch64-pc-windows-msvc" }
}

function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host " $Message" -ForegroundColor Cyan
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Step {
    param([string]$Message)
    Write-Host "[*] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[!] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[X] $Message" -ForegroundColor Red
}

function Test-Prerequisites {
    Write-Header "检查构建环境"

    # Rust
    Write-Step "检查 Rust..."
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "未找到 Cargo。请安装 Rust: https://rustup.rs"
    }
    $rustVersion = rustc --version
    Write-Host "    $rustVersion"

    # Inno Setup
    Write-Step "检查 Inno Setup..."
    if (-not (Test-Path $InnoSetupPath)) {
        Write-Warning "未找到 Inno Setup 6。安装程序生成将被跳过。"
        Write-Warning "下载: https://jrsoftware.org/isdl.php"
        return $false
    }
    Write-Host "    找到 Inno Setup: $InnoSetupPath"

    return $true
}

function Build-Project {
    Write-Header "构建项目"

    # 不指定 --target，使用默认 host target，输出到 release/ 而不是 release/<target>/
    $buildArgs = @(
        "build"
        "--release"
        "--package", $PackageName
    )

    Write-Step "运行 cargo build..."

    Push-Location $ProjectRoot
    try {
        & cargo @buildArgs
        if ($LASTEXITCODE -ne 0) {
            throw "构建失败"
        }
    }
    finally {
        Pop-Location
    }

    Write-Step "构建完成"
}

function Prepare-Files {
    Write-Header "准备打包文件"

    # 创建安装程序输出目录
    if (-not (Test-Path $InstallerDir)) {
        New-Item -ItemType Directory -Path $InstallerDir -Force | Out-Null
    }

    # 检查主程序 (包名可能与可执行文件名不同，例如 axon_app -> axon_app.exe)
    $exePath = Join-Path $ReleaseDir "$PackageName.exe"
    $targetExePath = Join-Path $ReleaseDir "$OutputName.exe"

    Write-Step "查找主程序: $exePath"

    if (Test-Path $exePath) {
        if ($exePath -ne $targetExePath) {
            Write-Step "复制主程序为 $OutputName.exe..."
            Copy-Item $exePath $targetExePath -Force
        }
    } elseif (Test-Path $targetExePath) {
        Write-Step "主程序已存在: $targetExePath"
    } else {
        throw "未找到主程序: $exePath`n请先运行构建: cargo build --release --package $PackageName"
    }

    # 检查图标文件
    $iconPath = Join-Path $ResourcesDir "app-icon.ico"
    if (-not (Test-Path $iconPath)) {
        Write-Warning "未找到图标文件: $iconPath"
        Write-Warning "请从 assets/icons/logo.svg 生成 .ico 文件"
    }

    Write-Step "文件准备完成"
}

function Build-Installer {
    Write-Header "生成安装程序"

    if (-not (Test-Path $InnoSetupPath)) {
        Write-Warning "跳过安装程序生成 (未安装 Inno Setup)"
        return
    }

    $issFile = Join-Path $ResourcesDir "axon_term.iss"
    if (-not (Test-Path $issFile)) {
        throw "未找到 Inno Setup 配置文件: $issFile"
    }

    Write-Step "运行 Inno Setup 编译器..."
    Write-Host "    Target Dir: $TargetDir"
    Write-Host "    Project Root: $ProjectRoot"

    # 设置环境变量供 ISS 文件使用
    $env:AXON_VERSION = $Version
    $env:AXON_TARGET_DIR = $TargetDir
    $env:AXON_PROJECT_ROOT = $ProjectRoot

    & $InnoSetupPath $issFile
    if ($LASTEXITCODE -ne 0) {
        throw "Inno Setup 编译失败"
    }

    $installerPath = Join-Path $InstallerDir "AxonTerm-$Version-x64-setup.exe"
    if (Test-Path $installerPath) {
        Write-Step "安装程序已生成: $installerPath"
        $size = (Get-Item $installerPath).Length / 1MB
        Write-Host "    大小: $([math]::Round($size, 2)) MB"
    }
}

function Install-Application {
    Write-Header "安装应用程序"

    $installerPath = Join-Path $InstallerDir "AxonTerm-$Version-x64-setup.exe"
    if (-not (Test-Path $installerPath)) {
        Write-Warning "未找到安装程序，跳过安装"
        return
    }

    Write-Step "运行安装程序..."
    Start-Process -FilePath $installerPath -Wait

    Write-Step "安装完成"
}

function Show-Summary {
    Write-Header "打包完成"

    Write-Host "版本: $Version"
    Write-Host "架构: $Architecture"
    Write-Host ""
    Write-Host "输出文件:"

    $installerPath = Join-Path $InstallerDir "AxonTerm-$Version-x64-setup.exe"
    if (Test-Path $installerPath) {
        Write-Host "  - $installerPath"
    }

    $exePath = Join-Path $ReleaseDir "axon_term.exe"
    if (Test-Path $exePath) {
        Write-Host "  - $exePath"
    }
}

# 主流程
try {
    Write-Host ""
    Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Magenta
    Write-Host "║      Axon Term Windows 打包工具        ║" -ForegroundColor Magenta
    Write-Host "║              v$Version                    ║" -ForegroundColor Magenta
    Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Magenta

    $hasInnoSetup = Test-Prerequisites

    if (-not $SkipBuild) {
        Build-Project
    }

    Prepare-Files

    if ($hasInnoSetup) {
        Build-Installer
    }

    if ($Install) {
        Install-Application
    }

    Show-Summary
}
catch {
    Write-Error $_.Exception.Message
    exit 1
}
