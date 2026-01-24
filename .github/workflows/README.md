# GitHub Workflows 说明

本项目包含多个 GitHub Actions workflow，用于自动化 CI/CD、安全扫描和代码审查。

## 📋 Workflows 概览

### 1. CI (ci.yml)
**触发条件**: Push 到 main/develop 分支，或 PR 提交

**功能**:
- ✅ **代码质量检查**
  - Rustfmt 格式化检查
  - Clippy 静态分析 (严格模式)

- 🔒 **安全漏洞扫描**
  - cargo-audit: 依赖安全漏洞检查
  - cargo-deny: 许可证和供应链安全检查

- 🧹 **未使用依赖检测**
  - cargo-machete: 识别并清理未使用的依赖

- 🏗️ **多平台构建**
  - Linux, Windows, macOS 三平台编译验证
  - Release 模式构建检查

- 🧪 **单元测试**
  - 全平台测试覆盖
  - 测试输出详细日志

- 📊 **代码覆盖率**
  - cargo-llvm-cov 生成覆盖率报告
  - 自动上传到 Codecov

- 📚 **文档生成**
  - 验证所有文档可正确生成
  - 检查文档警告

- 🔬 **内存安全检查**
  - Miri 内存安全分析

### 2. 定期安全扫描 (security-scan.yml)
**触发条件**: 每天 UTC 0:00 自动运行，或手动触发

**功能**:
- 🔍 **依赖安全审计**
  - 自动生成安全审计报告
  - 发现漏洞时自动创建 GitHub Issue

- 🔎 **静态代码分析**
  - Clippy 安全相关 lint 检查
  - 检测潜在的运行时错误

- ⛓️ **供应链安全**
  - 验证依赖来源
  - 许可证合规检查

- 📈 **代码复杂度分析**
  - cargo-geiger: 扫描 unsafe 代码使用
  - 生成不安全代码报告

### 3. Release (release.yml)
**触发条件**: 推送版本 tag (v*.*.*)，或手动触发

**功能**:
- 🚀 **自动化发布**
  - 创建 GitHub Release
  - 生成 Release Notes

- 📦 **多平台构建**
  - **Windows**: MSI 安装包 (使用 WiX Toolset)
    - 完整的安装程序
    - 自动添加到 PATH
    - 开始菜单和桌面快捷方式
    - 包含所有资源文件（字体、图标、主题）
  - **Linux x86_64**: 压缩的二进制文件 (.tar.gz)
  - **macOS x86_64**: 压缩的二进制文件 (.tar.gz)
  - **macOS ARM64**: 压缩的二进制文件 (.tar.gz, Apple Silicon)

- 📤 **资产上传**
  - Windows: MSI 安装包
  - Linux/macOS: tar.gz 压缩包
  - 自动上传到 GitHub Release

### 4. Claude Code Review (claude-code-review.yml)
**触发条件**: PR 创建或更新时

**功能**:
- 🤖 **AI 代码审查**
  - 使用 Claude 自动审查 PR
  - 识别潜在问题和改进建议

### 5. Claude Code (claude.yml)
**触发条件**: 评论中提及 @claude

**功能**:
- 💬 **AI 助手**
  - 回答代码相关问题
  - 执行指定任务

## 🔧 配置文件

### deny.toml
cargo-deny 的配置文件，用于：
- 定义允许的开源许可证
- 配置安全漏洞检查规则
- 设置依赖源验证

## 📊 安全扫描工具说明

### cargo-audit
- **用途**: 检查已知的安全漏洞
- **数据源**: RustSec Advisory Database
- **检查内容**: 依赖树中的所有包

### cargo-deny
- **用途**: 多维度的依赖检查
- **检查项**:
  - 安全漏洞 (advisories)
  - 许可证合规 (licenses)
  - 依赖来源 (sources)
  - 禁用包列表 (bans)

### cargo-machete
- **用途**: 检测未使用的依赖
- **效果**: 减少依赖树大小，提升编译速度

### cargo-geiger
- **用途**: 统计 unsafe 代码使用
- **输出**: 各 crate 的 unsafe 代码占比

### Clippy
- **用途**: Rust 官方 linter
- **检查级别**:
  - all: 所有基础检查
  - pedantic: 严格检查
  - nursery: 实验性检查
  - cargo: Cargo.toml 相关检查

### Miri
- **用途**: 运行时内存安全检查
- **检查内容**: 未定义行为、内存泄漏等

## 🚀 使用指南

### 本地运行安全检查

```bash
# 安装工具
cargo install cargo-audit cargo-deny cargo-machete cargo-geiger

# 依赖安全审计
cargo audit

# 完整安全检查
cargo deny check

# 查找未使用的依赖
cargo machete

# 扫描不安全代码
cargo geiger
```

### 创建 Release

```bash
# 1. 更新版本号
# 编辑 Cargo.toml 中的 [workspace.package] version 字段
# version = "1.0.0"

# 2. 提交版本更新
git add Cargo.toml
git commit -m "chore: bump version to v1.0.0"

# 3. 创建并推送 tag
git tag v1.0.0
git push origin v1.0.0

# 4. GitHub Actions 自动构建并发布
# - 构建所有平台的二进制文件
# - 生成 Windows MSI 安装包
# - 创建 GitHub Release
# - 上传所有资产
```

### 本地构建 Windows MSI 安装包

如果需要在本地构建 Windows 安装包：

```bash
# 1. 安装 WiX Toolset
# 下载: https://wixtoolset.org/releases/
# 或使用 Chocolatey: choco install wixtoolset

# 2. 安装 cargo-wix
cargo install cargo-wix

# 3. 构建 Release 版本
cargo build --release --target x86_64-pc-windows-msvc -p z_term

# 4. 生成 MSI 安装包
cd crates/z_term
cargo wix --target x86_64-pc-windows-msvc --nocapture

# 5. 安装包位于: target/wix/z_term-<version>-x86_64.msi
```

详细配置说明请参考 [wix/README.md](../../wix/README.md)

### 手动触发安全扫描

1. 进入 GitHub Actions 页面
2. 选择 "定期安全扫描" workflow
3. 点击 "Run workflow"

## 🔐 所需的 Secrets

如果要启用所有功能，需要配置以下 secrets:

- `CLAUDE_CODE_OAUTH_TOKEN`: Claude Code 的 OAuth token (用于 AI 审查)
- `CARGO_REGISTRY_TOKEN`: crates.io 的 API token (用于发布包)
- Codecov 会自动识别公开仓库，私有仓库需要配置 token

## 📈 持续改进建议

1. **定期更新依赖**: 每周检查并更新依赖
2. **监控安全报告**: 关注自动创建的 Issue
3. **提升代码覆盖率**: 目标 80% 以上
4. **减少 unsafe 代码**: 尽量使用安全抽象
5. **完善测试**: 增加集成测试和边界测试

## 🐛 常见问题

### Q: CI 失败怎么办?
A: 查看具体的失败步骤，常见原因：
- 格式化问题: 运行 `cargo fmt`
- Clippy 警告: 运行 `cargo clippy --fix`
- 测试失败: 本地运行 `cargo test`

### Q: 如何临时跳过某个检查?
A: 不建议跳过，但可以：
- 在代码中添加 `#[allow(clippy::xxx)]`
- 修改 deny.toml 调整规则

### Q: 安全扫描发现误报怎么办?
A: 在 deny.toml 中添加例外规则，并注释说明原因

## 📚 相关资源

- [cargo-audit 文档](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny 文档](https://embarkstudios.github.io/cargo-deny/)
- [Clippy Lints 列表](https://rust-lang.github.io/rust-clippy/master/)
- [RustSec Advisory DB](https://rustsec.org/)
