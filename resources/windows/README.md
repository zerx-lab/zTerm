# Windows 资源文件

此目录包含 Windows 安装程序所需的资源文件。

## 文件说明

- `zterm.iss` - Inno Setup 安装程序配置文件
- `app-icon.ico` - 应用程序图标 (由打包脚本自动生成)
- `messages/` - 多语言消息文件

## 构建安装程序

### 前提条件

1. 安装 [Inno Setup 6](https://jrsoftware.org/isdl.php)
2. 安装 Rust 工具链
3. (可选) 安装 [ImageMagick](https://imagemagick.org/) 用于生成图标

### 构建命令

```powershell
# 在项目根目录运行
.\script\bundle-windows.ps1

# 或者指定架构
.\script\bundle-windows.ps1 -Architecture x86_64

# 构建后自动安装
.\script\bundle-windows.ps1 -Install

# 跳过构建，仅打包
.\script\bundle-windows.ps1 -SkipBuild
```

### 输出

安装程序将生成在 `target/installer/` 目录下。

## 图标生成

打包脚本会自动尝试生成图标。如果自动生成失败，可以手动生成：

### 方法 1: 使用 ImageMagick

```bash
magick convert assets/icons/logo.svg -define icon:auto-resize=256,128,64,48,32,16 resources/windows/app-icon.ico
```

### 方法 2: 使用在线工具

1. 访问 https://convertio.co/svg-ico/
2. 上传 `assets/icons/logo.svg`
3. 下载生成的 `.ico` 文件
4. 重命名为 `app-icon.ico` 并放置在此目录

### 方法 3: 使用 Inkscape + ImageMagick

```bash
inkscape assets/icons/logo.svg --export-filename=logo.png -w 256 -h 256
magick convert logo.png -define icon:auto-resize=256,128,64,48,32,16 resources/windows/app-icon.ico
```
