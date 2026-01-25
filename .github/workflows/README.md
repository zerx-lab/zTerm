# GitHub Workflows 说明

本项目包含多个 GitHub Actions workflow，用于自动化发布和代码审查。

## 📋 Workflows 概览

### 1. Release (release.yml)
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

### 2. Claude Code Review (claude-code-review.yml)
**触发条件**: PR 创建或更新时

**功能**:
- 🤖 **AI 代码审查**
  - 使用 Claude 自动审查 PR
  - 识别潜在问题和改进建议

### 3. Claude Code (claude.yml)
**触发条件**: 评论中提及 @claude

**功能**:
- 💬 **AI 助手**
  - 回答代码相关问题
  - 执行指定任务
  - 自动修复代码问题

## 🔧 配置文件

### deny.toml
cargo-deny 的配置文件，用于：
- 定义允许的开源许可证
- 配置安全漏洞检查规则
- 设置依赖源验证

## 🛠️ 推荐开发工具

### Clippy
- **用途**: Rust 官方 linter
- **安装**: 随 Rust 工具链自动安装
- **检查级别**:
  - all: 所有基础检查
  - pedantic: 严格检查
  - nursery: 实验性检查
  - cargo: Cargo.toml 相关检查

### cargo-deny
- **用途**: 多维度的依赖检查
- **安装**: `cargo install cargo-deny`
- **检查项**:
  - 安全漏洞 (advisories)
  - 许可证合规 (licenses)
  - 依赖来源 (sources)
  - 禁用包列表 (bans)

### cargo-machete
- **用途**: 检测未使用的依赖
- **安装**: `cargo install cargo-machete`
- **效果**: 减少依赖树大小，提升编译速度

## 🚀 使用指南

### 本地开发检查

推荐在提交代码前运行以下命令：

```bash
# 代码格式化
cargo fmt --all

# 格式检查
cargo fmt --all -- --check

# 静态分析
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 运行测试
cargo test --workspace --all-features

# 快速编译检查
cargo check --workspace --all-targets --all-features
```

### 可选的安全检查工具

```bash
# 安装工具
cargo install cargo-audit cargo-deny cargo-machete

# 依赖安全审计
cargo audit

# 完整安全检查
cargo deny check

# 查找未使用的依赖
cargo machete
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

## 🔐 所需的 Secrets

Claude 集成需要配置以下 secret:

- `CLAUDE_CODE_OAUTH_TOKEN`: Claude Code 的 OAuth token (用于 AI 审查和助手功能)

## 📈 持续改进建议

1. **定期更新依赖**: 每周检查并更新依赖
2. **运行本地检查**: 提交前运行格式化、Clippy 和测试
3. **完善测试**: 增加集成测试和边界测试
4. **减少 unsafe 代码**: 尽量使用安全抽象
5. **使用 Claude 助手**: 在 PR 中通过 `@claude` 获取代码审查建议

## 🐛 常见问题

### Q: 如何使用 Claude 代码审查?
A: 创建或更新 PR 时，claude-code-review workflow 会自动运行并提供审查意见

### Q: 如何手动触发 Claude 助手?
A: 在 Issue 或 PR 评论中提及 `@claude`，并描述需要执行的任务

### Q: 如何临时跳过 Clippy 警告?
A: 不建议跳过，但可以在代码中添加 `#[allow(clippy::xxx)]`

## 📚 相关资源

- [Clippy Lints 列表](https://rust-lang.github.io/rust-clippy/master/)
- [cargo-deny 文档](https://embarkstudios.github.io/cargo-deny/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Claude Code 文档](https://docs.anthropic.com/claude/docs)
