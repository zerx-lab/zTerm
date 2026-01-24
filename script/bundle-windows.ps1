<#
.SYNOPSIS
    zTerm Windows 打包脚本

.DESCRIPTION
    构建并打包 zTerm 为 Windows 安装程序。

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
    [string]$PackageName = "z_term",

    # 输出的可执行文件名 (与 Cargo.toml 中 [[bin]] name 一致)
    [string]$OutputName = "zterm"
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

    # 检查主程序 (二进制名由 Cargo.toml 中的 [[bin]] name 决定)
    $targetExePath = Join-Path $ReleaseDir "$OutputName.exe"

    Write-Step "查找主程序: $targetExePath"

    if (Test-Path $targetExePath) {
        Write-Step "主程序已存在: $targetExePath"
    } else {
        throw "未找到主程序: $targetExePath`n请先运行构建: cargo build --release --package $PackageName"
    }

    Write-Step "文件准备完成"
}

function Build-Installer {
    Write-Header "生成安装程序"

    if (-not (Test-Path $InnoSetupPath)) {
        Write-Warning "跳过安装程序生成 (未安装 Inno Setup)"
        return
    }

    $issFile = Join-Path $ResourcesDir "zterm.iss"
    if (-not (Test-Path $issFile)) {
        throw "未找到 Inno Setup 配置文件: $issFile"
    }

    # 确保图标文件存在
    $iconPath = Join-Path $ResourcesDir "app-icon.ico"
    if (-not (Test-Path $iconPath)) {
        Write-Step "生成安装程序图标..."
        Generate-Icon
    }

    Write-Step "运行 Inno Setup 编译器..."
    Write-Host "    Target Dir: $TargetDir"
    Write-Host "    Project Root: $ProjectRoot"

    # 设置环境变量供 ISS 文件使用
    $env:ZTERM_VERSION = $Version
    $env:ZTERM_TARGET_DIR = $TargetDir
    $env:ZTERM_PROJECT_ROOT = $ProjectRoot

    & $InnoSetupPath $issFile
    if ($LASTEXITCODE -ne 0) {
        throw "Inno Setup 编译失败"
    }

    $installerPath = Join-Path $InstallerDir "zTerm-$Version-x64-setup.exe"
    if (Test-Path $installerPath) {
        Write-Step "安装程序已生成: $installerPath"
        $size = (Get-Item $installerPath).Length / 1MB
        Write-Host "    大小: $([math]::Round($size, 2)) MB"
    }
}

function Generate-Icon {
    # 尝试从编译输出中复制图标，或使用 ImageMagick 生成
    $svgPath = Join-Path $AssetsDir "icons\logo.svg"
    $iconPath = Join-Path $ResourcesDir "app-icon.ico"

    # 方法 1: 检查是否有 ImageMagick
    if (Get-Command magick -ErrorAction SilentlyContinue) {
        Write-Step "使用 ImageMagick 生成图标..."
        & magick convert $svgPath -define icon:auto-resize=256,128,64,48,32,16 $iconPath
        if ($LASTEXITCODE -eq 0 -and (Test-Path $iconPath)) {
            Write-Host "    图标已生成: $iconPath"
            return
        }
    }

    # 方法 2: 检查是否有 Inkscape + ImageMagick
    if ((Get-Command inkscape -ErrorAction SilentlyContinue) -and (Get-Command magick -ErrorAction SilentlyContinue)) {
        Write-Step "使用 Inkscape + ImageMagick 生成图标..."
        $tempPng = Join-Path $env:TEMP "zterm-logo.png"
        & inkscape $svgPath --export-filename=$tempPng -w 256 -h 256
        if ($LASTEXITCODE -eq 0) {
            & magick convert $tempPng -define icon:auto-resize=256,128,64,48,32,16 $iconPath
            Remove-Item $tempPng -ErrorAction SilentlyContinue
            if (Test-Path $iconPath) {
                Write-Host "    图标已生成: $iconPath"
                return
            }
        }
    }

    # 方法 3: 检查编译输出目录中是否有生成的图标
    $buildIconDir = Join-Path $TargetDir "release\build"
    if (Test-Path $buildIconDir) {
        $buildIco = Get-ChildItem -Path $buildIconDir -Recurse -Filter "app-icon.ico" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($buildIco) {
            Write-Step "从构建输出复制图标..."
            Copy-Item $buildIco.FullName $iconPath -Force
            Write-Host "    图标已复制: $iconPath"
            return
        }
    }

    Write-Warning "无法生成图标文件，请手动创建 $iconPath"
    Write-Warning "可以使用以下方法之一:"
    Write-Warning "  1. 安装 ImageMagick: winget install ImageMagick.ImageMagick"
    Write-Warning "  2. 使用在线工具转换 SVG 到 ICO"
}

function Install-Application {
    Write-Header "安装应用程序"

    $installerPath = Join-Path $InstallerDir "zTerm-$Version-x64-setup.exe"
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

    $installerPath = Join-Path $InstallerDir "zTerm-$Version-x64-setup.exe"
    if (Test-Path $installerPath) {
        Write-Host "  - $installerPath"
    }

    $exePath = Join-Path $ReleaseDir "zterm.exe"
    if (Test-Path $exePath) {
        Write-Host "  - $exePath"
    }
}

# 主流程
try {
    Write-Host ""
    Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Magenta
    Write-Host "║        zTerm Windows 打包工具          ║" -ForegroundColor Magenta
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
