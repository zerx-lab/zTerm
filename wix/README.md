# Windows MSI 安装包打包指南

本目录包含用于创建 zTerm Windows MSI 安装包的 WiX 配置文件。

## 📦 功能特性

生成的 MSI 安装包包含以下功能：

- ✅ **标准安装程序**: 符合 Windows 安装规范的 MSI 包
- ✅ **自动环境变量**: 自动添加 zTerm 到系统 PATH
- ✅ **开始菜单快捷方式**: 在开始菜单创建程序快捷方式
- ✅ **桌面快捷方式**: 可选的桌面快捷方式
- ✅ **完整资源打包**: 包含字体、图标、主题等资源文件
- ✅ **自动升级**: 支持覆盖安装和升级
- ✅ **卸载支持**: 完整的卸载功能

## 🛠️ 本地构建 MSI 安装包

### 前置要求

1. **安装 WiX Toolset**:
   - 下载并安装 [WiX Toolset v3.x](https://wixtoolset.org/releases/)
   - 或使用 Chocolatey: `choco install wixtoolset`

2. **安装 cargo-wix**:
   ```bash
   cargo install cargo-wix
   ```

### 构建步骤

1. **构建 Release 版本**:
   ```bash
   # 在项目根目录
   cargo build --release --target x86_64-pc-windows-msvc -p z_term
   ```

2. **生成 MSI 安装包**:
   ```bash
   # 进入主应用 crate 目录
   cd crates/z_term

   # 使用 cargo-wix 生成 MSI
   cargo wix --target x86_64-pc-windows-msvc --nocapture

   # 生成的 MSI 位于: target/wix/z_term-<version>-x86_64.msi
   ```

3. **自定义输出路径**:
   ```bash
   cargo wix --target x86_64-pc-windows-msvc --nocapture --output ../../release/zterm-installer.msi
   ```

## 🔧 WiX 配置说明

### main.wxs 文件结构

```xml
<?xml version='1.0'?>
<Wix xmlns='http://schemas.microsoft.com/wix/2006/wi'>
  <Product>
    <!-- 产品定义 -->
    <Package>
      <!-- 安装包元数据 -->
    </Package>

    <Directory>
      <!-- 安装目录结构 -->
    </Directory>

    <Feature>
      <!-- 功能组件 -->
    </Feature>
  </Product>
</Wix>
```

### 关键配置项

#### 1. UpgradeCode (升级 GUID)
```xml
UpgradeCode='6C8E4A2D-1F3B-4E9A-8D2C-9F7E6B5A4C3D'
```
- **重要**: 这个 GUID **永远不要改变**
- 用于识别同一产品的不同版本
- 改变会导致无法自动升级

#### 2. 版本号
```xml
Version='$(var.Version)'
```
- 从 Cargo.toml 自动读取
- 格式: `major.minor.patch`

#### 3. 安装路径
```xml
<Directory Id='APPLICATIONFOLDER' Name='zTerm'>
```
- 默认安装到: `C:\Program Files\zTerm`
- 用户可在安装时更改

#### 4. 资源文件
```xml
<Component Id='ThemesComponent'>
  <File Id='DarkTheme' Name='dark.toml' Source='assets\themes\dark.toml'/>
</Component>
```
- 自动打包 assets 目录下的资源
- 包括字体、图标、主题

## 📝 自定义安装包

### 修改安装包信息

编辑 `main.wxs`:

```xml
<!-- 修改产品名称 -->
<Product Name='zTerm'
         Manufacturer='zTerm Team'>

<!-- 修改描述 -->
<Package Description='A modern terminal emulator'>
```

### 添加额外文件

```xml
<Component Id='MyComponent' Guid='NEW-GUID-HERE'>
  <File Id='MyFile' Name='myfile.txt' Source='path\to\myfile.txt' KeyPath='yes'/>
</Component>

<!-- 在 Feature 中引用 -->
<Feature>
  <ComponentRef Id='MyComponent' />
</Feature>
```

### 添加许可协议

1. 创建 `wix/License.rtf` 文件（RTF 格式）
2. 在 `main.wxs` 中取消注释:
   ```xml
   <WixVariable Id='WixUILicenseRtf' Value='wix\License.rtf'/>
   ```

### 修改应用图标

1. 替换 `assets/icons/logo.svg`
2. 或直接使用 ICO 文件:
   ```xml
   <Icon Id='ProductIcon' SourceFile='path\to\icon.ico'/>
   ```

## 🚀 CI/CD 自动构建

Release workflow 会自动构建 MSI 安装包：

1. **触发方式**:
   - 推送版本标签: `git tag v1.0.0 && git push origin v1.0.0`
   - 或手动触发 workflow

2. **构建产物**:
   - MSI 安装包会自动上传到 GitHub Release
   - 文件名格式: `zterm-v1.0.0-x64.msi`

3. **构建流程**:
   ```yaml
   - 安装 Rust 工具链
   - 安装 cargo-wix
   - 构建 Release 二进制
   - 生成 MSI 安装包
   - 上传到 GitHub Release
   ```

## 🔍 常见问题

### Q: 构建失败，提示找不到 WiX 工具？
A: 确保已安装 WiX Toolset 并添加到 PATH。检查:
```bash
candle.exe -?
light.exe -?
```

### Q: 如何修改安装包的版本号？
A: 版本号自动从 `Cargo.toml` 读取，修改 workspace 的 `version` 字段即可。

### Q: 安装时提示"来自未知发行者"？
A: 这是因为 MSI 未签名。生产环境应该：
1. 购买代码签名证书
2. 使用 `signtool.exe` 对 MSI 签名
3. 或在 workflow 中集成 Azure Code Signing

### Q: 如何添加卸载时的清理逻辑？
A: 使用 WiX 的 `RemoveFile` 或 `RemoveFolder` 元素：
```xml
<Component>
  <RemoveFolder Id='RemoveConfigFolder' Directory='ConfigFolder' On='uninstall'/>
</Component>
```

### Q: 能否创建便携版（免安装）？
A: 可以，但需要单独的构建流程：
```bash
# 构建后直接打包二进制和资源
cargo build --release
# 然后手动打包 target/release/zterm.exe 和 assets 文件夹
```

## 📚 参考资源

- [WiX Toolset 官方文档](https://wixtoolset.org/documentation/)
- [cargo-wix GitHub](https://github.com/volks73/cargo-wix)
- [WiX 教程](https://www.firegiant.com/wix/tutorial/)
- [Windows Installer 最佳实践](https://docs.microsoft.com/en-us/windows/win32/msi/windows-installer-best-practices)

## 🔐 代码签名（可选）

为生产环境的 MSI 签名：

1. **获取证书**:
   - 购买代码签名证书（如 DigiCert, Sectigo）
   - 或使用 Azure Code Signing

2. **使用 signtool 签名**:
   ```bash
   signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com zterm-installer.msi
   ```

3. **在 CI/CD 中集成**:
   ```yaml
   - name: 签名 MSI
     run: |
       signtool sign /f ${{ secrets.CERTIFICATE_PATH }} `
                     /p ${{ secrets.CERTIFICATE_PASSWORD }} `
                     /t http://timestamp.digicert.com `
                     target/wix/zterm-*.msi
   ```

## 📊 安装包大小优化

1. **启用 LTO 和 strip**: 已在 `Cargo.toml` 中配置
2. **压缩二进制**: WiX 默认使用 CAB 压缩
3. **移除调试符号**: 使用 `strip = true`

当前配置下，预期安装包大小约 5-15 MB（取决于依赖）。
